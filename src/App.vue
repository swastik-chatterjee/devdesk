<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  Activity, ArrowUpRight, Check, ChevronDown, CircleHelp, Command,
  GitFork, KeyRound, LayoutDashboard, LockKeyhole, Menu, Plus,
  RefreshCw, Search, Settings2, ShieldCheck, Sparkles, Trash2, X,
} from "@lucide/vue";

type Profile = {
  profile_id: string;
  username: string;
  avatar_url: string;
  repository_count: number;
  connected_at: string;
};
type Repository = { name: string; full_name: string; private: boolean; html_url: string; description: string | null; updated_at: string };

const profiles = ref<Profile[]>([]);
const repositories = ref<Repository[]>([]);
const activeProfileId = ref("");
const token = ref("");
const revealToken = ref(false);
const busy = ref(false);
const loadingRepos = ref(false);
const notice = ref("");
const error = ref("");
const query = ref("");
const mobileNavOpen = ref(false);
const activeNav = ref("Settings");
const isDesktop = Boolean((window as Window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__);

const activeProfile = computed(() => profiles.value.find((profile) => profile.profile_id === activeProfileId.value) ?? profiles.value[0]);
const visibleRepositories = computed(() => repositories.value.filter((repository) =>
  `${repository.full_name} ${repository.description ?? ""}`.toLowerCase().includes(query.value.toLowerCase()),
));
const shortDate = (value: string) => new Intl.DateTimeFormat(undefined, { month: "short", day: "numeric" }).format(new Date(value));

async function loadProfiles() {
  if (!isDesktop) return;
  try {
    profiles.value = await invoke<Profile[]>("list_profiles");
    if (profiles.value.length) activeNav.value = "Overview";
    if (!profiles.value.some((profile) => profile.profile_id === activeProfileId.value)) {
      activeProfileId.value = profiles.value[0]?.profile_id ?? "";
    }
    if (activeProfileId.value) await loadRepositories();
  } catch {
    error.value = "Could not read connected profiles. Check that the system credential store is available.";
  }
}

async function loadRepositories() {
  if (!activeProfileId.value) return;
  loadingRepos.value = true;
  try {
    repositories.value = await invoke<Repository[]>("list_repositories", { profileId: activeProfileId.value });
    error.value = "";
  } catch (reason) {
    error.value = String(reason);
  } finally {
    loadingRepos.value = false;
  }
}

async function connectProfile() {
  if (!token.value.trim() || busy.value) return;
  busy.value = true;
  error.value = "";
  notice.value = "";
  try {
    const profile = await invoke<Profile>("connect_github", { token: token.value.trim() });
    token.value = "";
    profiles.value = [...profiles.value.filter((item) => item.username !== profile.username), profile];
    activeProfileId.value = profile.profile_id;
    activeNav.value = "Overview";
    notice.value = `@${profile.username} connected securely`;
    await loadRepositories();
  } catch (reason) {
    error.value = String(reason);
  } finally {
    token.value = "";
    busy.value = false;
  }
}

async function removeProfile(profile: Profile) {
  if (!window.confirm(`Remove @${profile.username} from Devdesk? Its saved credential will be deleted.`)) return;
  try {
    await invoke("disconnect_github", { profileId: profile.profile_id });
    profiles.value = profiles.value.filter((item) => item.profile_id !== profile.profile_id);
    repositories.value = [];
    if (activeProfileId.value === profile.profile_id) activeProfileId.value = profiles.value[0]?.profile_id ?? "";
    if (activeProfileId.value) await loadRepositories();
    notice.value = `@${profile.username} disconnected`;
  } catch (reason) {
    error.value = String(reason);
  }
}

function selectProfile(profile: Profile) {
  activeProfileId.value = profile.profile_id;
  void loadRepositories();
}

onMounted(loadProfiles);
</script>

<template>
  <div class="shell">
    <aside class="sidebar" :class="{ 'sidebar-open': mobileNavOpen }">
      <div class="brand"><div class="brand-mark"><Command :size="18" /></div><span>devdesk</span><span class="brand-build">DESKTOP</span></div>
      <div class="workspace-label">WORKSPACE</div>
      <button class="workspace-picker"><span class="workspace-icon">D</span><span class="workspace-copy"><strong>Developer space</strong><small>Personal workspace</small></span><ChevronDown :size="15" /></button>
      <nav class="primary-nav" aria-label="Main navigation">
        <button :class="{ selected: activeNav === 'Overview' }" @click="activeNav = 'Overview'; mobileNavOpen = false"><LayoutDashboard :size="17" />Overview</button>
        <button :class="{ selected: activeNav === 'Repositories' }" @click="activeNav = 'Repositories'; mobileNavOpen = false"><GitFork :size="17" />Repositories<span class="nav-count">{{ activeProfile?.repository_count ?? 0 }}</span></button>
        <button :class="{ selected: activeNav === 'Activity' }" @click="activeNav = 'Activity'; mobileNavOpen = false"><Activity :size="17" />Activity</button>
      </nav>
      <div class="sidebar-spacer"></div>
      <div class="sidebar-bottom">
        <div class="sidebar-status"><span class="status-dot"></span><span>{{ profiles.length }} account{{ profiles.length === 1 ? '' : 's' }} connected</span><button title="Refresh profiles" aria-label="Refresh profiles" @click="loadProfiles"><RefreshCw :size="14" /></button></div>
        <button class="nav-settings" :class="{ selected: activeNav === 'Settings' }" @click="activeNav = 'Settings'; mobileNavOpen = false"><Settings2 :size="17" />Settings</button>
        <div class="user-row"><div class="user-avatar">DS</div><div><strong>Devdesk</strong><small>Local workspace</small></div><button class="icon-button" title="Help" aria-label="Help"><CircleHelp :size="17" /></button></div>
      </div>
    </aside>

    <main class="main-panel">
      <header class="topbar">
        <button class="icon-button mobile-menu" aria-label="Open navigation" @click="mobileNavOpen = !mobileNavOpen"><Menu :size="19" /></button>
        <div class="breadcrumb">Workspace <span>/</span> <strong>{{ activeNav }}</strong></div>
        <div class="topbar-right"><div class="connection-pill"><span class="status-dot"></span>{{ activeProfile ? 'GitHub connected' : 'No account connected' }}</div><button class="top-avatar" title="Current workspace">D</button></div>
      </header>

      <div class="page-content">
        <div v-if="error && activeNav !== 'Settings'" class="form-message error-message" role="alert"><X :size="15" /><span>{{ error }}</span><button class="text-action" @click="activeNav = 'Settings'">Review connection <ArrowUpRight :size="13" /></button></div>
        <section v-if="activeNav === 'Overview' || activeNav === 'Repositories'" class="welcome-row">
          <div><div class="eyebrow"><span class="eyebrow-line"></span>YOUR DEVELOPER SPACE</div><h1>{{ activeNav === 'Repositories' ? 'Repositories' : 'Good to see you.' }}</h1><p>{{ activeNav === 'Repositories' ? 'Your GitHub repositories, all in one quiet place.' : 'A clear view of your work across GitHub.' }}</p></div>
          <button class="date-chip"><span class="date-indicator"></span>{{ new Intl.DateTimeFormat(undefined, { weekday: 'long', month: 'long', day: 'numeric' }).format(new Date()) }}</button>
        </section>

        <section v-if="activeNav === 'Overview' || activeNav === 'Repositories'" class="metrics-grid" aria-label="Workspace summary">
          <article class="metric"><div class="metric-top"><span>CONNECTED PROFILES</span><div class="metric-icon green"><GitFork :size="16" /></div></div><div class="metric-value">{{ profiles.length.toString().padStart(2, '0') }}<span class="metric-caption">github account{{ profiles.length === 1 ? '' : 's' }}</span></div><div class="metric-foot"><span class="small-live"><i></i>All systems nominal</span></div></article>
          <article class="metric"><div class="metric-top"><span>REPOSITORIES</span><div class="metric-icon blue"><GitFork :size="16" /></div></div><div class="metric-value">{{ repositories.length.toString().padStart(2, '0') }}<span class="metric-caption">visible to active profile</span></div><div class="metric-foot">{{ activeProfile ? `@${activeProfile.username}` : 'Connect an account to begin' }}</div></article>
          <article class="metric"><div class="metric-top"><span>CONNECTION SECURITY</span><div class="metric-icon neutral"><ShieldCheck :size="16" /></div></div><div class="metric-value metric-word">{{ activeProfile ? 'Protected' : 'Ready' }}</div><div class="metric-foot"><LockKeyhole :size="12" /> Credentials stored in system keychain</div></article>
        </section>

        <section v-if="activeNav === 'Overview' || activeNav === 'Repositories'" class="content-grid">
          <div class="panel repo-panel">
            <div class="panel-heading"><div><div class="section-kicker">YOUR CODE</div><h2>Recent repositories</h2></div><button class="text-action" @click="activeNav = 'Repositories'">View all <ArrowUpRight :size="14" /></button></div>
            <div v-if="repositories.length" class="repo-toolbar"><label class="search-field"><Search :size="15" /><input v-model="query" placeholder="Find a repository..." aria-label="Find a repository" /></label><button class="icon-button" title="Refresh repositories" aria-label="Refresh repositories" @click="loadRepositories"><RefreshCw :size="15" /></button></div>
            <div v-if="loadingRepos" class="empty-state"><span class="loading-ring"></span><strong>Loading repositories</strong><small>Fetching access from GitHub</small></div>
            <div v-else-if="!repositories.length" class="empty-state"><div class="empty-icon"><GitFork :size="20" /></div><strong>No repositories yet</strong><small>Connect a profile with repository access to see your projects here.</small><button class="subtle-action" @click="activeNav = 'Settings'">Manage GitHub profiles <ArrowUpRight :size="13" /></button></div>
            <div v-else class="repo-list"><a v-for="repo in visibleRepositories.slice(0, 6)" :key="repo.full_name" class="repo-row" :href="repo.html_url" target="_blank" rel="noreferrer"><div class="repo-symbol"><GitFork :size="15" /></div><div class="repo-copy"><strong>{{ repo.name }}</strong><small>{{ repo.description || repo.full_name }}</small></div><span v-if="repo.private" class="visibility-tag">PRIVATE</span><span v-else class="visibility-tag public">PUBLIC</span><span class="repo-updated">{{ shortDate(repo.updated_at) }}</span><ArrowUpRight class="repo-open" :size="14" /></a></div>
            <div v-if="repositories.length" class="repo-footer">Showing {{ Math.min(visibleRepositories.length, 6) }} of {{ visibleRepositories.length }} accessible repositories <span>{{ activeProfile ? `via @${activeProfile.username}` : '' }}</span></div>
          </div>

          <aside class="right-column">
            <div class="panel profile-panel"><div class="panel-heading compact"><div><div class="section-kicker">ACCOUNTS</div><h2>GitHub profiles</h2></div><span class="profile-count">{{ profiles.length }}</span></div>
              <div v-if="profiles.length" class="profile-list"><button v-for="profile in profiles" :key="profile.profile_id" class="profile-row" :class="{ active: activeProfileId === profile.profile_id }" @click="selectProfile(profile)"><img class="profile-avatar" :src="profile.avatar_url" alt="" /><span class="profile-copy"><strong>{{ profile.username }}</strong><small>{{ profile.repository_count }} repositories</small></span><span class="profile-state"><i></i>Active</span></button></div>
              <button class="add-profile" @click="activeNav = 'Settings'"><Plus :size="15" />Add another profile</button>
            </div>
            <div class="permission-note"><div class="permission-icon"><Sparkles :size="16" /></div><div><strong>Access, on your terms.</strong><p>Tokens stay in your system keychain. Choose exactly which repositories to share.</p><button @click="activeNav = 'Settings'">Review security <ArrowUpRight :size="12" /></button></div></div>
          </aside>
        </section>

        <section v-if="activeNav === 'Settings'" class="settings-view">
          <div class="settings-intro"><div class="eyebrow"><span class="eyebrow-line"></span>ACCOUNT MANAGEMENT</div><h1>GitHub connections</h1><p>Connect multiple profiles and keep each credential protected in your system keychain.</p></div>
          <div class="settings-layout">
            <div class="panel connections-panel"><div class="panel-heading"><div><div class="section-kicker">CONNECTED ACCOUNTS</div><h2>Your profiles</h2></div><span class="profile-count">{{ profiles.length }} connected</span></div>
              <div v-if="profiles.length" class="connection-list"><article v-for="profile in profiles" :key="profile.profile_id" class="connection-card"><img class="connection-avatar" :src="profile.avatar_url" alt="" /><div class="connection-info"><strong>{{ profile.username }}</strong><small><span class="inline-state"><i></i>Connected</span><span class="connection-separator">·</span>{{ profile.repository_count }} repositories</small></div><button class="icon-button danger-button" :title="`Remove ${profile.username}`" :aria-label="`Remove ${profile.username}`" @click="removeProfile(profile)"><Trash2 :size="16" /></button></article></div>
              <div v-else class="no-profiles"><div class="empty-icon"><GitFork :size="20" /></div><strong>No GitHub profiles connected</strong><small>Add a profile below to enable live GitHub data.</small></div>
            </div>

            <div class="panel connect-panel"><div class="panel-heading"><div><div class="section-kicker">NEW CONNECTION</div><h2>Connect a profile</h2></div><div class="metric-icon green"><GitFork :size="16" /></div></div>
              <div class="connect-explainer"><span class="step-number">01</span><span>Create a fine-grained token on GitHub, then paste it below.</span><a href="https://github.com/settings/personal-access-tokens/new" target="_blank" rel="noreferrer" aria-label="Create fine-grained token on GitHub"><ArrowUpRight :size="14" /></a></div>
              <form class="token-form" @submit.prevent="connectProfile">
                <label class="field-label" for="github-token">Personal access token</label>
                <div class="token-input-wrap"><KeyRound :size="16" /><input id="github-token" v-model="token" :type="revealToken ? 'text' : 'password'" autocomplete="off" spellcheck="false" placeholder="github_pat_••••••••••••••••" :disabled="busy || !isDesktop" /><button class="icon-button reveal-button" type="button" :title="revealToken ? 'Hide token' : 'Reveal token'" :aria-label="revealToken ? 'Hide token' : 'Reveal token'" :disabled="!isDesktop" @click="revealToken = !revealToken">{{ revealToken ? 'Hide' : 'Show' }}</button></div>
                <button class="connect-button" type="submit" :disabled="busy || !token.trim() || !isDesktop"><span v-if="busy" class="loading-ring small-ring"></span><LockKeyhole v-else :size="15" />{{ busy ? 'Verifying securely…' : 'Connect to GitHub' }}<ArrowUpRight v-if="!busy" :size="15" /></button>
              </form>
              <div v-if="!isDesktop" class="runtime-warning"><CircleHelp :size="14" />Credential submission works in the Devdesk desktop app.</div>
              <div v-if="error" class="form-message error-message" role="alert"><X :size="15" />{{ error }}</div>
              <div v-if="notice" class="form-message success-message" role="status"><Check :size="15" />{{ notice }}</div>
              <div class="permission-box"><div class="permission-title"><ShieldCheck :size="15" /><strong>Recommended access</strong></div><ul><li><Check :size="12" />Metadata: read-only, required for repository discovery</li><li><Check :size="12" />Select only the repositories Devdesk should access</li><li><Check :size="12" />Contents: read-only only if you later enable file browsing</li></ul><p>Issue creation, pull request management, and workflow data need extra permissions. Devdesk currently uses read-only repository discovery.</p></div>
              <div class="keychain-foot"><LockKeyhole :size="13" /><span>Validated and stored in your operating system credential store. Never saved in app settings.</span></div>
            </div>
          </div>
        </section>

        <section v-if="activeNav === 'Activity'" class="placeholder-view"><div class="empty-icon"><Activity :size="20" /></div><div class="section-kicker">ACTIVITY</div><h1>Your activity stream is quiet.</h1><p>GitHub activity will appear here as those features are added.</p><button class="subtle-action" @click="activeNav = 'Overview'">Back to overview</button></section>

        <footer class="page-footer"><span><span class="status-dot"></span>LOCAL DATA · PRIVATE BY DESIGN</span><span>Devdesk <span class="footer-divider">/</span> GitHub workspace</span><a href="https://docs.github.com/authentication/keeping-your-account-and-data-secure/creating-a-personal-access-token" target="_blank" rel="noreferrer">Token documentation <ArrowUpRight :size="11" /></a></footer>
      </div>
    </main>
    <button v-if="mobileNavOpen" class="mobile-scrim" aria-label="Close navigation" @click="mobileNavOpen = false"></button>
  </div>
</template>