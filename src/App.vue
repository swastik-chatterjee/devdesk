<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  Activity, ArrowUpRight, Check, CheckSquare, ChevronDown, CircleHelp, Command,
  GitFork, GitPullRequest, KeyRound, LayoutDashboard, ListTodo, LockKeyhole,
  Menu, Plus, RefreshCw, Search, Settings2, ShieldCheck, Sparkles, Trash2,
  Workflow, X,
} from "@lucide/vue";

type Profile = {
  profile_id: string;
  username: string;
  avatar_url: string;
  repository_count: number;
  connected_at: string;
};
type Repository = { name: string; full_name: string; private: boolean; html_url: string; description: string | null; updated_at: string };
type WorkflowRun = {
  name: string; status: string; conclusion: string | null; html_url: string;
  run_number: number; updated_at: string; repository: string;
};
type PullRequest = {
  title: string; number: number; state: string; html_url: string; author: string;
  updated_at: string; repository: string; draft: boolean; merged: boolean;
};
type ActivityEvent = {
  id: string; event_type: string; repository: string; repository_url: string; created_at: string;
};
type Todo = { id: string; title: string; done: boolean; created_at: string };
type Workspace = { id: string; name: string; todos: Todo[] };
type WorkspacePage = "Overview" | "Repositories" | "Actions" | "Pull requests" | "Todos" | "Activity" | "Settings";

const WORKSPACE_STORAGE_KEY = "devdesk.workspace-state";
const profiles = ref<Profile[]>([]);
const repositories = ref<Repository[]>([]);
const workflowRuns = ref<WorkflowRun[]>([]);
const pullRequests = ref<PullRequest[]>([]);
const activityEvents = ref<ActivityEvent[]>([]);
const activeProfileId = ref("");
const token = ref("");
const todoTitle = ref("");
const revealToken = ref(false);
const busy = ref(false);
const loadingRepos = ref(false);
const loadingFeature = ref(false);
const notice = ref("");
const error = ref("");
const query = ref("");
const mobileNavOpen = ref(false);
const workspaceMenuOpen = ref(false);
const workspaceDialogOpen = ref(false);
const workspaceDialogMode = ref<"create" | "rename">("create");
const workspaceNameDraft = ref("");
const activeNav = ref<WorkspacePage>("Settings");
const workspaceStorageEnabled = ref(true);
const isDesktop = Boolean((window as Window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__);
const defaultWorkspace: Workspace = { id: "personal", name: "Developer space", todos: [] };
let featureRequestId = 0;

function readWorkspaceState(): { workspaces: Workspace[]; activeWorkspaceId: string } {
  const saved = window.localStorage.getItem(WORKSPACE_STORAGE_KEY);
  if (!saved) return { workspaces: [defaultWorkspace], activeWorkspaceId: defaultWorkspace.id };
  const parsed: unknown = JSON.parse(saved);
  if (!parsed || typeof parsed !== "object") throw new Error("Saved workspace data is invalid.");
  const state = parsed as { workspaces?: unknown; activeWorkspaceId?: unknown };
  if (!Array.isArray(state.workspaces) || !state.workspaces.length) throw new Error("Saved workspace data is invalid.");
  const valid = state.workspaces.every((workspace) =>
    workspace && typeof workspace.id === "string" && typeof workspace.name === "string" &&
    Array.isArray(workspace.todos) && workspace.todos.every((todo: unknown) =>
      todo && typeof todo === "object" &&
      typeof (todo as Todo).id === "string" &&
      typeof (todo as Todo).title === "string" &&
      typeof (todo as Todo).done === "boolean" &&
      typeof (todo as Todo).created_at === "string",
    ),
  );
  if (!valid) throw new Error("Saved workspace data is invalid.");
  const workspaces = state.workspaces as Workspace[];
  const activeWorkspaceId = typeof state.activeWorkspaceId === "string" &&
    workspaces.some((workspace) => workspace.id === state.activeWorkspaceId)
    ? state.activeWorkspaceId
    : workspaces[0].id;
  return { workspaces, activeWorkspaceId };
}

let initialWorkspaceState = { workspaces: [defaultWorkspace], activeWorkspaceId: defaultWorkspace.id };
try {
  initialWorkspaceState = readWorkspaceState();
} catch {
  workspaceStorageEnabled.value = false;
  error.value = "Saved workspace data could not be read. Your existing data was left untouched.";
}
const workspaces = ref<Workspace[]>(initialWorkspaceState.workspaces);
const activeWorkspaceId = ref(initialWorkspaceState.activeWorkspaceId);
const activeWorkspace = computed(() =>
  workspaces.value.find((workspace) => workspace.id === activeWorkspaceId.value) ?? workspaces.value[0],
);
const activeProfile = computed(() => profiles.value.find((profile) => profile.profile_id === activeProfileId.value) ?? profiles.value[0]);
const visibleRepositories = computed(() => repositories.value.filter((repository) =>
  `${repository.full_name} ${repository.description ?? ""}`.toLowerCase().includes(query.value.toLowerCase()),
));
const completedTodoCount = computed(() => activeWorkspace.value?.todos.filter((todo) => todo.done).length ?? 0);
const remainingTodoCount = computed(() => (activeWorkspace.value?.todos.length ?? 0) - completedTodoCount.value);
const shortDate = (value: string) => new Intl.DateTimeFormat(undefined, { month: "short", day: "numeric" }).format(new Date(value));
const fullDate = (value: string) => new Intl.DateTimeFormat(undefined, { month: "short", day: "numeric", hour: "numeric", minute: "2-digit" }).format(new Date(value));

watch([workspaces, activeWorkspaceId], () => {
  if (!workspaceStorageEnabled.value) return;
  try {
    window.localStorage.setItem(WORKSPACE_STORAGE_KEY, JSON.stringify({
      workspaces: workspaces.value,
      activeWorkspaceId: activeWorkspaceId.value,
    }));
  } catch {
    error.value = "Could not save workspace changes to local storage.";
  }
}, { deep: true });

async function loadProfiles() {
  if (!isDesktop) return;
  try {
    profiles.value = await invoke<Profile[]>("list_profiles");
    if (profiles.value.length) activeNav.value = "Overview";
    if (!profiles.value.some((profile) => profile.profile_id === activeProfileId.value)) {
      activeProfileId.value = profiles.value[0]?.profile_id ?? "";
    }
    if (activeProfileId.value) await loadRepositories();
  } catch (reason) {
    error.value = String(reason);
  }
}

async function loadRepositories() {
  if (!activeProfileId.value) {
    repositories.value = [];
    return;
  }
  loadingRepos.value = true;
  try {
    repositories.value = await invoke<Repository[]>("list_repositories", { profileId: activeProfileId.value });
    error.value = "";
  } catch (reason) {
    repositories.value = [];
    error.value = String(reason);
  } finally {
    loadingRepos.value = false;
  }
}

async function loadFeature(page: "Actions" | "Pull requests" | "Activity") {
  if (!activeProfileId.value) {
    error.value = "Connect a GitHub profile in Settings to load this view.";
    return;
  }
  const profileId = activeProfileId.value;
  const requestId = ++featureRequestId;
  loadingFeature.value = true;
  error.value = "";
  try {
    if (page === "Actions") {
      const result = await invoke<WorkflowRun[]>("list_workflow_runs", { profileId });
      if (requestId === featureRequestId && profileId === activeProfileId.value) workflowRuns.value = result;
    } else if (page === "Pull requests") {
      const result = await invoke<PullRequest[]>("list_pull_requests", { profileId });
      if (requestId === featureRequestId && profileId === activeProfileId.value) pullRequests.value = result;
    } else {
      const result = await invoke<ActivityEvent[]>("list_activity", { profileId });
      if (requestId === featureRequestId && profileId === activeProfileId.value) activityEvents.value = result;
    }
  } catch (reason) {
    if (requestId === featureRequestId && profileId === activeProfileId.value) error.value = String(reason);
  } finally {
    if (requestId === featureRequestId) loadingFeature.value = false;
  }
}

function navigate(page: WorkspacePage) {
  activeNav.value = page;
  mobileNavOpen.value = false;
  error.value = "";
  if (page === "Actions" || page === "Pull requests" || page === "Activity") {
    void loadFeature(page);
  } else {
    featureRequestId += 1;
    loadingFeature.value = false;
  }
}

function createWorkspace() {
  workspaceDialogMode.value = "create";
  workspaceNameDraft.value = "";
  workspaceDialogOpen.value = true;
  workspaceMenuOpen.value = false;
}

function renameWorkspace() {
  if (!activeWorkspace.value) return;
  workspaceDialogMode.value = "rename";
  workspaceNameDraft.value = activeWorkspace.value.name;
  workspaceDialogOpen.value = true;
  workspaceMenuOpen.value = false;
}

function saveWorkspaceName() {
  const name = workspaceNameDraft.value.trim();
  if (!name) return;
  if (workspaceDialogMode.value === "rename" && activeWorkspace.value) {
    activeWorkspace.value.name = name;
    notice.value = `Workspace renamed to "${name}"`;
    workspaceDialogOpen.value = false;
    return;
  }
  const workspace: Workspace = {
    id: `workspace-${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`,
    name,
    todos: [],
  };
  workspaces.value.push(workspace);
  activeWorkspaceId.value = workspace.id;
  workspaceDialogOpen.value = false;
  activeNav.value = "Overview";
  notice.value = `Workspace "${workspace.name}" created`;
}

function addTodo() {
  const title = todoTitle.value.trim();
  if (!title || !activeWorkspace.value) return;
  activeWorkspace.value.todos.unshift({
    id: `todo-${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`,
    title,
    done: false,
    created_at: new Date().toISOString(),
  });
  todoTitle.value = "";
}

function deleteTodo(todoId: string) {
  if (activeWorkspace.value) {
    activeWorkspace.value.todos = activeWorkspace.value.todos.filter((todo) => todo.id !== todoId);
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
    workflowRuns.value = [];
    pullRequests.value = [];
    activityEvents.value = [];
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
    workflowRuns.value = [];
    pullRequests.value = [];
    activityEvents.value = [];
    featureRequestId += 1;
    loadingFeature.value = false;
    if (activeProfileId.value === profile.profile_id) activeProfileId.value = profiles.value[0]?.profile_id ?? "";
    if (activeProfileId.value) await loadRepositories();
    if (activeNav.value === "Actions" || activeNav.value === "Pull requests" || activeNav.value === "Activity") {
      void loadFeature(activeNav.value);
    }
    notice.value = `@${profile.username} disconnected`;
  } catch (reason) {
    error.value = String(reason);
  }
}

function selectProfile(profile: Profile) {
  activeProfileId.value = profile.profile_id;
  workflowRuns.value = [];
  pullRequests.value = [];
  activityEvents.value = [];
  featureRequestId += 1;
  loadingFeature.value = false;
  void loadRepositories();
  if (activeNav.value === "Actions" || activeNav.value === "Pull requests" || activeNav.value === "Activity") {
    void loadFeature(activeNav.value);
  }
}

function describeEvent(eventType: string) {
  const descriptions: Record<string, string> = {
    PushEvent: "Pushed updates to",
    PullRequestEvent: "Updated a pull request in",
    IssuesEvent: "Updated an issue in",
    CreateEvent: "Created a branch or tag in",
    WatchEvent: "Starred",
    ForkEvent: "Forked",
    ReleaseEvent: "Published a release in",
    IssueCommentEvent: "Commented in",
  };
  return descriptions[eventType] ?? `${eventType.replace(/Event$/, "").replace(/([A-Z])/g, " $1").trim()} in`;
}

onMounted(loadProfiles);
</script>

<template>
  <div class="shell">
    <aside class="sidebar" :class="{ 'sidebar-open': mobileNavOpen }">
      <div class="brand"><div class="brand-mark"><Command :size="18" /></div><span>devdesk</span><span class="brand-build">DESKTOP</span></div>
      <div class="workspace-label">WORKSPACE</div>
      <div class="workspace-control">
        <button class="workspace-picker" :aria-expanded="workspaceMenuOpen" @click="workspaceMenuOpen = !workspaceMenuOpen">
          <span class="workspace-icon">{{ activeWorkspace?.name.slice(0, 1).toUpperCase() }}</span>
          <span class="workspace-copy"><strong>{{ activeWorkspace?.name }}</strong><small>{{ workspaces.length }} workspace{{ workspaces.length === 1 ? '' : 's' }}</small></span>
          <ChevronDown :size="15" />
        </button>
        <div v-if="workspaceMenuOpen" class="workspace-menu">
          <div class="workspace-menu-label">YOUR WORKSPACES</div>
          <button v-for="workspace in workspaces" :key="workspace.id" class="workspace-option" :class="{ active: activeWorkspaceId === workspace.id }" @click="activeWorkspaceId = workspace.id; workspaceMenuOpen = false; notice = ''">
            <span class="workspace-option-icon">{{ workspace.name.slice(0, 1).toUpperCase() }}</span>
            <span>{{ workspace.name }}</span>
            <Check v-if="activeWorkspaceId === workspace.id" :size="14" />
          </button>
          <div class="workspace-menu-actions">
            <button @click="createWorkspace"><Plus :size="14" />New workspace</button>
            <button @click="renameWorkspace"><Settings2 :size="14" />Rename</button>
          </div>
        </div>
      </div>
      <nav class="primary-nav" aria-label="Main navigation">
        <button :class="{ selected: activeNav === 'Overview' }" @click="navigate('Overview')"><LayoutDashboard :size="17" />Overview</button>
        <button :class="{ selected: activeNav === 'Repositories' }" @click="navigate('Repositories')"><GitFork :size="17" />Repositories<span class="nav-count">{{ activeProfile?.repository_count ?? 0 }}</span></button>
        <button :class="{ selected: activeNav === 'Actions' }" @click="navigate('Actions')"><Workflow :size="17" />GitHub Actions</button>
        <button :class="{ selected: activeNav === 'Pull requests' }" @click="navigate('Pull requests')"><GitPullRequest :size="17" />Pull requests</button>
        <button :class="{ selected: activeNav === 'Todos' }" @click="navigate('Todos')"><CheckSquare :size="17" />Todos<span class="nav-count">{{ remainingTodoCount || '' }}</span></button>
        <button :class="{ selected: activeNav === 'Activity' }" @click="navigate('Activity')"><Activity :size="17" />Activity</button>
      </nav>
      <div class="sidebar-spacer"></div>
      <div class="sidebar-bottom">
        <div class="sidebar-status"><span class="status-dot"></span><span>{{ profiles.length }} account{{ profiles.length === 1 ? '' : 's' }} connected</span><button title="Refresh profiles" aria-label="Refresh profiles" @click="loadProfiles"><RefreshCw :size="14" /></button></div>
        <button class="nav-settings" :class="{ selected: activeNav === 'Settings' }" @click="navigate('Settings')"><Settings2 :size="17" />Settings</button>
        <div class="user-row"><div class="user-avatar">DS</div><div><strong>Devdesk</strong><small>{{ activeWorkspace?.name }}</small></div><button class="icon-button" title="Help" aria-label="Help"><CircleHelp :size="17" /></button></div>
      </div>
    </aside>

    <main class="main-panel">
      <header class="topbar">
        <button class="icon-button mobile-menu" aria-label="Open navigation" @click="mobileNavOpen = !mobileNavOpen"><Menu :size="19" /></button>
        <div class="breadcrumb">{{ activeWorkspace?.name }} <span>/</span> <strong>{{ activeNav }}</strong></div>
        <div class="topbar-right"><div class="connection-pill"><span class="status-dot"></span>{{ activeProfile ? 'GitHub connected' : 'No account connected' }}</div><button class="top-avatar" :title="activeWorkspace?.name">{{ activeWorkspace?.name.slice(0, 1).toUpperCase() }}</button></div>
      </header>

      <div class="page-content">
        <div v-if="error && activeNav !== 'Settings'" class="form-message error-message" role="alert"><X :size="15" /><span>{{ error }}</span><button class="text-action" @click="navigate('Settings')">Review connection <ArrowUpRight :size="13" /></button></div>
        <div v-if="notice && activeNav !== 'Settings'" class="form-message success-message" role="status"><Check :size="15" /><span>{{ notice }}</span></div>
        <section v-if="activeNav === 'Overview' || activeNav === 'Repositories'" class="welcome-row">
          <div><div class="eyebrow"><span class="eyebrow-line"></span>{{ activeWorkspace?.name.toUpperCase() }}</div><h1>{{ activeNav === 'Repositories' ? 'Repositories' : 'Your developer overview' }}</h1><p>{{ activeNav === 'Repositories' ? 'Browse, search, and open every repository you can access.' : 'Your repositories, workflow runs, pull requests, and personal tasks.' }}</p></div>
          <button class="date-chip"><span class="date-indicator"></span>{{ new Intl.DateTimeFormat(undefined, { weekday: 'long', month: 'long', day: 'numeric' }).format(new Date()) }}</button>
        </section>

        <section v-if="activeNav === 'Overview' || activeNav === 'Repositories'" class="metrics-grid" aria-label="Workspace summary">
          <article class="metric"><div class="metric-top"><span>CONNECTED PROFILES</span><div class="metric-icon green"><GitFork :size="16" /></div></div><div class="metric-value">{{ profiles.length.toString().padStart(2, '0') }}<span class="metric-caption">github account{{ profiles.length === 1 ? '' : 's' }}</span></div><div class="metric-foot"><span class="small-live"><i></i>All systems nominal</span></div></article>
          <article class="metric"><div class="metric-top"><span>REPOSITORIES</span><div class="metric-icon blue"><GitFork :size="16" /></div></div><div class="metric-value">{{ repositories.length.toString().padStart(2, '0') }}<span class="metric-caption">visible to active profile</span></div><div class="metric-foot">{{ activeProfile ? `@${activeProfile.username}` : 'Connect an account to begin' }}</div></article>
          <article class="metric"><div class="metric-top"><span>OPEN TODOS</span><div class="metric-icon neutral"><CheckSquare :size="16" /></div></div><div class="metric-value">{{ remainingTodoCount.toString().padStart(2, '0') }}<span class="metric-caption">in {{ activeWorkspace?.name }}</span></div><div class="metric-foot">{{ completedTodoCount }} completed · local to this workspace</div></article>
        </section>

        <section v-if="activeNav === 'Overview' || activeNav === 'Repositories'" class="content-grid">
          <div class="panel repo-panel">
            <div class="panel-heading"><div><div class="section-kicker">YOUR CODE</div><h2>{{ activeNav === 'Repositories' ? 'All repositories' : 'Recent repositories' }}</h2></div><button v-if="activeNav === 'Overview'" class="text-action" @click="navigate('Repositories')">View all <ArrowUpRight :size="14" /></button></div>
            <div v-if="repositories.length" class="repo-toolbar"><label class="search-field"><Search :size="15" /><input v-model="query" placeholder="Find a repository..." aria-label="Find a repository" /></label><button class="icon-button" title="Refresh repositories" aria-label="Refresh repositories" @click="loadRepositories"><RefreshCw :size="15" /></button></div>
            <div v-if="loadingRepos" class="empty-state"><span class="loading-ring"></span><strong>Loading repositories</strong><small>Fetching access from GitHub</small></div>
            <div v-else-if="!repositories.length" class="empty-state"><div class="empty-icon"><GitFork :size="20" /></div><strong>No repositories yet</strong><small>Connect a profile with repository access to see your projects here.</small><button class="subtle-action" @click="navigate('Settings')">Manage GitHub profiles <ArrowUpRight :size="13" /></button></div>
            <div v-else-if="visibleRepositories.length" class="repo-list"><a v-for="repo in visibleRepositories.slice(0, activeNav === 'Repositories' ? undefined : 6)" :key="repo.full_name" class="repo-row" :href="repo.html_url" target="_blank" rel="noreferrer"><div class="repo-symbol"><GitFork :size="15" /></div><div class="repo-copy"><strong>{{ repo.name }}</strong><small>{{ repo.description || repo.full_name }}</small></div><span v-if="repo.private" class="visibility-tag">PRIVATE</span><span v-else class="visibility-tag public">PUBLIC</span><span class="repo-updated">{{ shortDate(repo.updated_at) }}</span><ArrowUpRight class="repo-open" :size="14" /></a></div>
            <div v-else-if="!loadingRepos && repositories.length" class="empty-state compact-empty"><strong>No matching repositories</strong><small>Try another repository name or description.</small></div>
            <div v-if="repositories.length" class="repo-footer">Showing {{ activeNav === 'Repositories' ? visibleRepositories.length : Math.min(visibleRepositories.length, 6) }} of {{ visibleRepositories.length }} accessible repositories <span>{{ activeProfile ? `via @${activeProfile.username}` : '' }}</span></div>
          </div>

          <aside class="right-column">
            <div class="panel profile-panel"><div class="panel-heading compact"><div><div class="section-kicker">ACCOUNTS</div><h2>GitHub profiles</h2></div><span class="profile-count">{{ profiles.length }}</span></div>
              <div v-if="profiles.length" class="profile-list"><button v-for="profile in profiles" :key="profile.profile_id" class="profile-row" :class="{ active: activeProfileId === profile.profile_id }" @click="selectProfile(profile)"><img class="profile-avatar" :src="profile.avatar_url" alt="" /><span class="profile-copy"><strong>{{ profile.username }}</strong><small>{{ profile.repository_count }} repositories</small></span><span class="profile-state"><i></i>Active</span></button></div>
              <button class="add-profile" @click="navigate('Settings')"><Plus :size="15" />Add another profile</button>
            </div>
            <div class="permission-note"><div class="permission-icon"><Sparkles :size="16" /></div><div><strong>Access, on your terms.</strong><p>Tokens stay in your system keychain. Choose exactly which repositories to share.</p><button @click="navigate('Settings')">Review security <ArrowUpRight :size="12" /></button></div></div>
            <div class="panel quick-links"><div class="section-kicker">YOUR WORKFLOW</div><button @click="navigate('Actions')"><Workflow :size="15" />GitHub Actions<ArrowUpRight :size="13" /></button><button @click="navigate('Pull requests')"><GitPullRequest :size="15" />Pull requests<ArrowUpRight :size="13" /></button><button @click="navigate('Todos')"><CheckSquare :size="15" />Workspace todos<ArrowUpRight :size="13" /></button><button @click="navigate('Activity')"><Activity :size="15" />GitHub activity<ArrowUpRight :size="13" /></button></div>
          </aside>
        </section>

        <section v-if="activeNav === 'Todos'" class="feature-view">
          <div class="feature-intro"><div class="eyebrow"><span class="eyebrow-line"></span>{{ activeWorkspace?.name.toUpperCase() }} WORKSPACE</div><h1>Todos</h1><p>Personal tasks saved locally in this workspace.</p></div>
          <div class="panel todo-panel">
            <div class="panel-heading"><div><div class="section-kicker">PERSONAL TASKS</div><h2>{{ remainingTodoCount }} open · {{ completedTodoCount }} completed</h2></div><span class="profile-count">{{ activeWorkspace?.todos.length ?? 0 }} total</span></div>
            <form class="todo-form" @submit.prevent="addTodo"><input v-model="todoTitle" aria-label="New todo" placeholder="What needs to get done?" maxlength="160" /><button class="connect-button" type="submit" :disabled="!todoTitle.trim()"><Plus :size="15" />Add todo</button></form>
            <div v-if="activeWorkspace?.todos.length" class="todo-list">
              <label v-for="todo in activeWorkspace.todos" :key="todo.id" class="todo-row" :class="{ done: todo.done }"><input v-model="todo.done" type="checkbox" /><span class="todo-copy"><strong>{{ todo.title }}</strong><small>Added {{ fullDate(todo.created_at) }}</small></span><button class="icon-button danger-button" :aria-label="`Delete ${todo.title}`" title="Delete todo" @click.prevent="deleteTodo(todo.id)"><Trash2 :size="15" /></button></label>
            </div>
            <div v-else class="empty-state"><div class="empty-icon"><ListTodo :size="20" /></div><strong>This workspace is clear</strong><small>Add a todo above to keep track of what you want to do next.</small></div>
          </div>
        </section>

        <section v-if="activeNav === 'Actions'" class="feature-view">
          <div class="feature-intro"><div class="eyebrow"><span class="eyebrow-line"></span>CI/CD</div><h1>GitHub Actions</h1><p>Recent workflow runs across the ten most recently updated repositories for {{ activeProfile ? `@${activeProfile.username}` : 'your connected profile' }}.</p></div>
          <div class="panel data-panel"><div class="panel-heading"><div><div class="section-kicker">WORKFLOW RUNS</div><h2>Recent runs</h2></div><button class="icon-button" title="Refresh workflow runs" aria-label="Refresh workflow runs" :disabled="loadingFeature" @click="loadFeature('Actions')"><RefreshCw :size="15" /></button></div>
            <div v-if="loadingFeature" class="empty-state"><span class="loading-ring"></span><strong>Loading workflow runs</strong><small>Checking recent repositories on GitHub</small></div>
            <div v-else-if="workflowRuns.length" class="feature-list"><a v-for="run in workflowRuns" :key="`${run.repository}-${run.run_number}`" class="feature-row" :href="run.html_url" target="_blank" rel="noreferrer"><span class="feature-symbol"><Workflow :size="16" /></span><span class="feature-copy"><strong>{{ run.name }} <span class="feature-number">#{{ run.run_number }}</span></strong><small>{{ run.repository }} · {{ fullDate(run.updated_at) }}</small></span><span class="state-tag" :class="run.conclusion || run.status">{{ run.conclusion || run.status }}</span><ArrowUpRight :size="14" /></a></div>
            <div v-else-if="!error" class="empty-state"><div class="empty-icon"><Workflow :size="20" /></div><strong>No workflow runs found</strong><small>Recent runs from the latest updated repositories will appear here. This feature needs Actions: read-only access on your token.</small><button class="subtle-action" @click="navigate('Settings')">Review token permissions</button></div>
          </div>
        </section>

        <section v-if="activeNav === 'Pull requests'" class="feature-view">
          <div class="feature-intro"><div class="eyebrow"><span class="eyebrow-line"></span>COLLABORATION</div><h1>Pull requests</h1><p>Recently updated pull requests across repositories connected to {{ activeProfile ? `@${activeProfile.username}` : 'your profile' }}.</p></div>
          <div class="panel data-panel"><div class="panel-heading"><div><div class="section-kicker">PULL REQUESTS</div><h2>Recently updated</h2></div><button class="icon-button" title="Refresh pull requests" aria-label="Refresh pull requests" :disabled="loadingFeature" @click="loadFeature('Pull requests')"><RefreshCw :size="15" /></button></div>
            <div v-if="loadingFeature" class="empty-state"><span class="loading-ring"></span><strong>Loading pull requests</strong><small>Checking recent repositories on GitHub</small></div>
            <div v-else-if="pullRequests.length" class="feature-list"><a v-for="pullRequest in pullRequests" :key="`${pullRequest.repository}-${pullRequest.number}`" class="feature-row" :href="pullRequest.html_url" target="_blank" rel="noreferrer"><span class="feature-symbol" :class="pullRequest.merged ? 'merged' : pullRequest.state"><GitPullRequest :size="16" /></span><span class="feature-copy"><strong>{{ pullRequest.title }} <span class="feature-number">#{{ pullRequest.number }}</span></strong><small>{{ pullRequest.repository }} · @{{ pullRequest.author }} · {{ fullDate(pullRequest.updated_at) }}</small></span><span class="state-tag" :class="pullRequest.merged ? 'merged' : pullRequest.state">{{ pullRequest.merged ? 'Merged' : pullRequest.draft ? 'Draft' : pullRequest.state }}</span><ArrowUpRight :size="14" /></a></div>
            <div v-else-if="!error" class="empty-state"><div class="empty-icon"><GitPullRequest :size="20" /></div><strong>No pull requests found</strong><small>Pull requests from the latest updated repositories will appear here. This feature needs Pull requests: read-only access.</small><button class="subtle-action" @click="navigate('Settings')">Review token permissions</button></div>
          </div>
        </section>

        <section v-if="activeNav === 'Activity'" class="feature-view">
          <div class="feature-intro"><div class="eyebrow"><span class="eyebrow-line"></span>GITHUB ACTIVITY</div><h1>Activity overview</h1><p>Recent public GitHub events for {{ activeProfile ? `@${activeProfile.username}` : 'your connected profile' }}.</p></div>
          <div class="panel data-panel"><div class="panel-heading"><div><div class="section-kicker">PUBLIC EVENTS</div><h2>Recent activity</h2></div><button class="icon-button" title="Refresh activity" aria-label="Refresh activity" :disabled="loadingFeature" @click="loadFeature('Activity')"><RefreshCw :size="15" /></button></div>
            <div v-if="loadingFeature" class="empty-state"><span class="loading-ring"></span><strong>Loading activity</strong><small>Fetching recent public events from GitHub</small></div>
            <div v-else-if="activityEvents.length" class="feature-list"><a v-for="event in activityEvents" :key="event.id" class="feature-row" :href="event.repository_url" target="_blank" rel="noreferrer"><span class="feature-symbol"><Activity :size="16" /></span><span class="feature-copy"><strong>{{ describeEvent(event.event_type) }} <span class="feature-number">{{ event.repository }}</span></strong><small>{{ fullDate(event.created_at) }}</small></span><ArrowUpRight :size="14" /></a></div>
            <div v-else-if="!error" class="empty-state"><div class="empty-icon"><Activity :size="20" /></div><strong>No recent public events</strong><small>GitHub returns recent public account activity here. Private events are not included by this endpoint.</small></div>
          </div>
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
              <div class="permission-box"><div class="permission-title"><ShieldCheck :size="15" /><strong>Recommended read access</strong></div><ul><li><Check :size="12" />Metadata: read-only, required for repository discovery</li><li><Check :size="12" />Pull requests: read-only, for the PR overview</li><li><Check :size="12" />Actions: read-only, for workflow run summaries</li><li><Check :size="12" />Select only the repositories Devdesk should access</li></ul><p>Todos are stored locally. Activity shows recent public account events. Devdesk does not create or modify issues, pull requests, or workflows.</p></div>
              <div class="keychain-foot"><LockKeyhole :size="13" /><span>Validated and stored in your operating system credential store. Never saved in app settings.</span></div>
            </div>
          </div>
        </section>

        <footer class="page-footer"><span><span class="status-dot"></span>LOCAL DATA · PRIVATE BY DESIGN</span><span>Devdesk <span class="footer-divider">/</span> {{ activeWorkspace?.name }}</span><a href="https://docs.github.com/authentication/keeping-your-account-and-data-secure/creating-a-personal-access-token" target="_blank" rel="noreferrer">Token documentation <ArrowUpRight :size="11" /></a></footer>
      </div>
    </main>
    <button v-if="mobileNavOpen" class="mobile-scrim" aria-label="Close navigation" @click="mobileNavOpen = false"></button>
    <div v-if="workspaceDialogOpen" class="dialog-scrim" @click.self="workspaceDialogOpen = false">
      <form class="workspace-dialog" @submit.prevent="saveWorkspaceName">
        <div class="workspace-dialog-heading"><div><div class="section-kicker">WORKSPACE</div><h2>{{ workspaceDialogMode === 'create' ? 'Create a workspace' : 'Rename workspace' }}</h2></div><button class="icon-button" type="button" aria-label="Close dialog" @click="workspaceDialogOpen = false"><X :size="16" /></button></div>
        <label class="field-label" for="workspace-name">Workspace name</label>
        <input id="workspace-name" v-model="workspaceNameDraft" class="workspace-name-input" autofocus maxlength="60" placeholder="e.g. Product engineering" />
        <div class="workspace-dialog-actions"><button class="subtle-action" type="button" @click="workspaceDialogOpen = false">Cancel</button><button class="connect-button" type="submit" :disabled="!workspaceNameDraft.trim()">{{ workspaceDialogMode === 'create' ? 'Create workspace' : 'Save name' }}</button></div>
      </form>
    </div>
  </div>
</template>