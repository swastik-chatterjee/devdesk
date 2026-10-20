# Devdesk

Devdesk is a desktop GitHub workspace built with Vue, TypeScript, and Tauri. GitHub access uses fine-grained personal access tokens; there is no interactive GitHub login flow.

## Development

Requirements: Node.js 20+, npm, Rust stable, and the native WebView/Tauri prerequisites for your operating system. See the [Tauri prerequisites](https://tauri.app/start/prerequisites/) before building the desktop app.

```sh
npm install
npm run dev
```

To run the desktop app:

```sh
npm run tauri dev
```

To build and check the frontend:

```sh
npm run build
```

## Releases

Every push to `main` that passes the frontend and Rust checks creates a unique release tag using the app version and workflow run number (for example, `v0.1.0-build.42`). GitHub Actions then builds the release bundles for Linux, macOS, and Windows and publishes the installers to a GitHub release.

## GitHub access

On first launch, create a fine-grained PAT using [GitHub's token settings](https://github.com/settings/personal-access-tokens/new), select only the repositories Devdesk should access, and grant **Metadata: read-only** for repository discovery. Contents: read-only is only needed if repository file browsing is implemented. The current GitHub feature set is read-only repository discovery; creating issues, managing pull requests, and reading workflow information are not currently supported and do not require broader permissions yet.

The Rust backend validates the token with GitHub, reads the account and accessible repositories, and stores the secret in the operating system credential store. Profile metadata (username, avatar URL, repository count, and connection time) is stored separately in the app config directory. Tokens are not written to SQLite, settings files, source code, or logs, and are never returned to the Vue frontend after submission. Multiple GitHub accounts can be connected independently. Disconnecting removes the associated credential.

GitHub's API does not provide a definitive endpoint for inspecting every fine-grained token permission. When an API request is forbidden, Devdesk explains likely repository selection, permission, and rate-limit causes; it cannot always distinguish them. Revoked, expired, and invalid credentials are reported as token rejection by GitHub.

## Supported functionality

| Capability | Token permissions | Status |
| --- | --- | --- |
| Discover accessible repositories | Metadata: read-only | Supported |
| Read repository contents | Contents: read-only | Not implemented |
| Create issues | Issues: read and write | Not implemented |
| Manage pull requests | Pull requests: read and write; Contents: read-only for branch data | Not implemented |
| Read Actions/workflow information | Actions: read-only; Metadata: read-only | Not implemented |

Fine-grained tokens can have endpoint-specific limitations. See [GitHub's fine-grained PAT permissions](https://docs.github.com/rest/authentication/permissions-required-for-fine-grained-personal-access-tokens) documentation before expanding access. Devdesk does not silently ask for broad permissions.

## Offline behavior

GitHub-backed repository discovery requires a live connection and a valid credential. The desktop shell and local workspace remain available when GitHub is unreachable; live repository requests show an actionable error. Cached repository persistence and personal task management are not implemented yet.

## Security notes

Never commit PATs or paste them into issues, logs, or terminal arguments. Devdesk submits a token only to the narrowly scoped Rust `connect_github` command. Authenticated GitHub requests originate in Rust. Credentials are stored using the platform credential service through the `keyring` crate. On Linux, a Secret Service-compatible keyring must be available.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md), [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md), and [THIRD_PARTY.md](THIRD_PARTY.md).