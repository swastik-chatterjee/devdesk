use keyring::Entry;
use reqwest::{header, Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tauri::Manager;
use uuid::Uuid;

const SERVICE: &str = "devdesk.github";
const GITHUB_API: &str = "https://api.github.com";

#[derive(Clone, Serialize, Deserialize)]
struct Profile {
    profile_id: String,
    username: String,
    avatar_url: String,
    repository_count: usize,
    connected_at: String,
}

#[derive(Serialize, Deserialize)]
struct Repository {
    name: String,
    full_name: String,
    private: bool,
    html_url: String,
    description: Option<String>,
    updated_at: String,
}

#[derive(Deserialize)]
struct GithubUser {
    login: String,
    avatar_url: String,
}

#[derive(Deserialize)]
struct GithubRepository {
    name: String,
    full_name: String,
    private: bool,
    html_url: String,
    description: Option<String>,
    updated_at: String,
}

async fn fetch_repositories(client: &Client) -> Result<Vec<GithubRepository>, String> {
    let mut repositories = Vec::new();
    let mut page = 1;
    loop {
        let response = client.get(format!("{GITHUB_API}/user/repos?per_page=100&type=all&sort=updated&page={page}"))
            .send().await.map_err(|_| "Could not retrieve repository access from GitHub.".to_string())?;
        if !response.status().is_success() { return Err(explain_status(response.status())); }
        let mut page_repositories: Vec<GithubRepository> = response.json().await
            .map_err(|_| "GitHub returned an unreadable repository response.".to_string())?;
        let page_size = page_repositories.len();
        repositories.append(&mut page_repositories);
        if page_size < 100 { break; }
        page += 1;
    }
    Ok(repositories)
}

fn profile_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|_| "Could not access Devdesk settings.".to_string())?;
    fs::create_dir_all(&dir).map_err(|_| "Could not create Devdesk settings directory.".to_string())?;
    Ok(dir.join("github-profiles.json"))
}

fn read_profiles(app: &tauri::AppHandle) -> Result<Vec<Profile>, String> {
    let path = profile_path(app)?;
    if !path.exists() { return Ok(Vec::new()); }
    let contents = fs::read_to_string(path).map_err(|_| "Could not read GitHub profile settings.".to_string())?;
    serde_json::from_str(&contents).map_err(|_| "GitHub profile settings are invalid.".to_string())
}

fn write_profiles(app: &tauri::AppHandle, profiles: &[Profile]) -> Result<(), String> {
    let path = profile_path(app)?;
    let contents = serde_json::to_vec_pretty(profiles).map_err(|_| "Could not serialize profile settings.".to_string())?;
    fs::write(path, contents).map_err(|_| "Could not save GitHub profile settings.".to_string())
}

fn credential(profile_id: &str) -> Result<Entry, String> {
    Entry::new(SERVICE, profile_id).map_err(|_| "Could not access the operating system credential store.".to_string())
}

fn github_client(token: &str) -> Result<Client, String> {
    let mut auth = header::HeaderValue::from_str(&format!("Bearer {token}"))
        .map_err(|_| "The token contains invalid characters.".to_string())?;
    auth.set_sensitive(true);
    let mut headers = header::HeaderMap::new();
    headers.insert(header::AUTHORIZATION, auth);
    headers.insert(header::ACCEPT, header::HeaderValue::from_static("application/vnd.github+json"));
    headers.insert("x-github-api-version", header::HeaderValue::from_static("2022-11-28"));
    Client::builder().default_headers(headers).user_agent("Devdesk/0.1.0")
        .build().map_err(|_| "Could not prepare a secure GitHub connection.".to_string())
}

fn explain_status(status: StatusCode) -> String {
    match status {
        StatusCode::UNAUTHORIZED => "GitHub rejected this token. It may be invalid, expired, or revoked. Create a replacement token and try again.".into(),
        StatusCode::FORBIDDEN => "GitHub denied this request. Check that the token has repository Metadata (read-only) access, includes the intended repositories, and is not rate-limited.".into(),
        StatusCode::NOT_FOUND => "GitHub could not find the requested account or repository access. Confirm the token can access at least one selected repository.".into(),
        _ => format!("GitHub returned HTTP {status}. Check your connection and token access, then try again."),
    }
}

#[tauri::command]
async fn connect_github(app: tauri::AppHandle, token: String) -> Result<Profile, String> {
    let token = token.trim();
    if token.is_empty() { return Err("Enter a GitHub personal access token.".into()); }
    let client = github_client(token)?;
    let user_response = client.get(format!("{GITHUB_API}/user")).send().await
        .map_err(|_| "Could not reach GitHub. Check your connection and try again.".to_string())?;
    if !user_response.status().is_success() { return Err(explain_status(user_response.status())); }
    let user: GithubUser = user_response.json().await
        .map_err(|_| "GitHub returned an unreadable account response.".to_string())?;

    let repos = fetch_repositories(&client).await?;

    let mut profiles = read_profiles(&app)?;
    let existing = profiles.iter().find(|profile| profile.username.eq_ignore_ascii_case(&user.login)).cloned();
    let previous_token = if existing.is_some() {
        credential(&existing.as_ref().unwrap().profile_id)?.get_password().ok()
    } else {
        None
    };
    let profile = Profile {
        profile_id: existing.as_ref().map(|profile| profile.profile_id.clone()).unwrap_or_else(|| Uuid::new_v4().to_string()),
        username: user.login,
        avatar_url: user.avatar_url,
        repository_count: repos.len(),
        connected_at: existing.map(|profile| profile.connected_at).unwrap_or_else(|| timestamp()),
    };

    credential(&profile.profile_id)?.set_password(token)
        .map_err(|_| "Could not save the token in your operating system credential store.".to_string())?;
    profiles.retain(|item| item.profile_id != profile.profile_id);
    profiles.push(profile.clone());
    if let Err(error) = write_profiles(&app, &profiles) {
        let entry = credential(&profile.profile_id)?;
        if let Some(previous_token) = previous_token {
            let _ = entry.set_password(&previous_token);
        } else {
            let _ = entry.delete_credential();
        }
        return Err(error);
    }
    Ok(profile)
}

fn timestamp() -> String {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .map(|duration| format!("{}", duration.as_secs()))
        .unwrap_or_else(|_| "0".into())
}

#[tauri::command]
fn list_profiles(app: tauri::AppHandle) -> Result<Vec<Profile>, String> {
    read_profiles(&app)
}

#[tauri::command]
async fn list_repositories(app: tauri::AppHandle, profile_id: String) -> Result<Vec<Repository>, String> {
    let profiles = read_profiles(&app)?;
    if !profiles.iter().any(|profile| profile.profile_id == profile_id) {
        return Err("This GitHub profile is no longer connected. Add it again in Settings.".into());
    }
    let token = credential(&profile_id)?.get_password()
        .map_err(|_| "The saved GitHub token is unavailable. Replace this profile's token in Settings.".to_string())?;
    let client = github_client(&token)?;
    drop(token);
    let repos = fetch_repositories(&client).await?;
    Ok(repos.into_iter().map(|repo| Repository {
        name: repo.name,
        full_name: repo.full_name,
        private: repo.private,
        html_url: repo.html_url,
        description: repo.description,
        updated_at: repo.updated_at,
    }).collect())
}

#[tauri::command]
fn disconnect_github(app: tauri::AppHandle, profile_id: String) -> Result<(), String> {
    let mut profiles = read_profiles(&app)?;
    if !profiles.iter().any(|profile| profile.profile_id == profile_id) { return Ok(()); }
    credential(&profile_id)?.delete_credential()
        .map_err(|_| "Could not remove the token from your operating system credential store.".to_string())?;
    profiles.retain(|profile| profile.profile_id != profile_id);
    write_profiles(&app, &profiles)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![connect_github, list_profiles, list_repositories, disconnect_github])
        .run(tauri::generate_context!())
        .expect("error while running Devdesk");
}