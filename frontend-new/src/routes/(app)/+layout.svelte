<script lang="ts">
	import { onMount } from 'svelte';
	import { Bell, CheckCheck, ChevronLeft, Download, Folder, House, Inbox, MessageCircle, MessageSquare, Plus, User } from '@lucide/svelte';
	import { goto, invalidate, onNavigate } from '$app/navigation';
	import { navigating, page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { stamp } from '$lib/adapters/time';
	import { chatCrewRows } from '$lib/adapters/chat';
	import { ChatHeader, setChatHeader } from '$lib/chat/header.svelte';
	import { later } from '$lib/chat/later';
	import { QUICK_REMINDER, inboxAction } from '$lib/inbox/actions';
	import { setClock } from '$lib/clock';
	import { tabId } from '$lib/device';
	import { connectActivity } from '$lib/live/activity';
	import { signedOutTarget } from '$lib/live/leaving';
	import { settledRefresh } from '$lib/live/settle';
	import { SHORTCUT_KEYS, isTyping, shortcutFor } from '$lib/shortcuts';
	import { APP_VERSION, VERSION_LABEL } from '$lib/version';
	import { markSeen } from '$lib/releaseNotes';
	import type { Theme } from '$lib/theme';
	import AppShell from '$lib/ds/shells/AppShell.svelte';
	import { setPanelPlacement } from '$lib/ds/shells/panelPlacement';
	import { setMainLayout } from '$lib/ds/shells/mainLayout';
	import { setPhoneChrome } from '$lib/ds/shells/phoneChrome';
	import { PhoneScreen, setPhoneScreen } from '$lib/ds/shells/phoneScreen.svelte';
	import { phoneView } from '$lib/phone.svelte';
	import { LOADING_DELAY_MS, loadingPage, opensAnotherPage, type LoadingPage } from '$lib/navigation/loading';
	import PageLoading from './PageLoading.svelte';
	import Sidebar from '$lib/ds/chrome/Sidebar.svelte';
	import NavItem from '$lib/ds/navigation/NavItem.svelte';
	import NavGroupLabel from '$lib/ds/navigation/NavGroupLabel.svelte';
	import RecentsList from '$lib/ds/chrome/RecentsList.svelte';
	import UsageCard from '$lib/ds/chrome/UsageCard.svelte';
	import AccountMenu from '$lib/ds/chrome/AccountMenu.svelte';
	import ThemePickerButton from '$lib/ds/chrome/ThemePickerButton.svelte';
	import { setThemeContext } from '$lib/ds/chrome/themeContext';
	import WorkspaceSwitcher from '$lib/ds/chrome/WorkspaceSwitcher.svelte';
	import TopBar from '$lib/ds/chrome/TopBar.svelte';
	import QuickSearch, { type QuickSearchRow } from '$lib/ds/chrome/QuickSearch.svelte';
	import { searchGroups, type SearchGroup, type SearchHit } from '$lib/search';
	import ProjectTopBar, { type ProjectMember } from '$lib/ds/projects/ProjectTopBar.svelte';
	import ChatTopBar from '$lib/ds/thread/ChatTopBar.svelte';
	import SettingsNav from '$lib/ds/chrome/SettingsNav.svelte';
	import SettingsTopBar from '$lib/ds/chrome/SettingsTopBar.svelte';
	import AutosaveNote from '$lib/ds/settings/AutosaveNote.svelte';
	import CrewPanel from '$lib/ds/chrome/CrewPanel.svelte';
	import ShortcutsDialog from '$lib/ds/chrome/ShortcutsDialog.svelte';
	import ReleaseNotesDialog from '$lib/ds/chrome/ReleaseNotesDialog.svelte';
	import SessionExpired from '$lib/ds/chat/SessionExpired.svelte';
	import PhoneSignedOutSheet from '$lib/ds/thread/PhoneSignedOutSheet.svelte';
	import NeedsYouToast from '$lib/chat/NeedsYouToast.svelte';
	import Dialog from '$lib/ds/overlays/Dialog.svelte';
	import Modal from '$lib/ds/overlays/Modal.svelte';
	import ToastRegion from '$lib/ds/overlays/ToastRegion.svelte';
	import { toasts } from '$lib/ds/overlays/toasts.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import IconButton from '$lib/ds/actions/IconButton.svelte';
	import Icon from '$lib/ds/foundations/Icon.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';

	// The signed-in app around every page, as App / Home (Vr4nn) draws it: the sidebar (Vp4Ax) with
	// its counts, recents, usage and the person's account menu (LkJLt); the top bar (u8pcLQ); and
	// on Home, the crew column (EXxti) under the top bar. The person's live feed keeps the recents
	// and the inbox current, and signs this device out when another one says so. A chat (Flow / Chat
	// — Ask → think → answer, ENb07) takes the content edge to edge, puts its own name, Share and
	// menu in the top bar, and keeps the crew column beside it, full height. The Inbox (App / Inbox,
	// BFoiZ) also takes the content edge to edge, for its Coming up column, and puts Mark all read
	// and New reminder in the top bar. Settings (App / Settings / *) swap the sidebar for the
	// settings nav (Back to Nomi, or esc), name the page in the top bar, and lay out their own columns.
	// Left out on purpose (AGENTS.md §9): the LIFE › Money group (Money isn't part of v2 yet), the
	// top bar's search (the backend has no search to call: BACKEND_TODO.md), and the sidebar's
	// collapse button (the shell has no collapsed layout yet).
	let { data, children } = $props();

	const HOME = '/(app)';
	const CHAT = '/(app)/chat/[id]';
	const NEW_CHAT = '/(app)/chat/new';
	const INBOX = '/(app)/inbox';
	const PROJECTS = '/(app)/projects';
	const WORKSPACE = '/(app)/projects/[id]';
	const isHome = $derived(page.route.id === HOME);
	// A chat that couldn't open shows the error page, laid out as a page.
	const isChat = $derived(page.route.id === CHAT && !page.error);
	const inThread = $derived(isChat || page.route.id === NEW_CHAT);
	const isInbox = $derived(page.route.id === INBOX && !page.error);
	const isProjects = $derived(page.route.id === PROJECTS && !page.error);
	// A project's workspace has its own bar and no sidebar (Flow / Build draws it edge to edge).
	const isWorkspace = $derived(page.route.id === WORKSPACE && !page.error);
	const isSettings = $derived(!!page.route.id?.startsWith('/(app)/settings') && !page.error);
	// Settings' own index is the You tab on a phone, with its large title.
	const isYou = $derived(page.route.id === '/(app)/settings' && !page.error);
	// Settings pages that are their own phone screens (Flow / Mobile — You & Settings): they draw the
	// back bar to You and their title; the rest keep the settings top bar with its back link.
	const PHONE_SETTINGS = new Set(['/(app)/settings/general', '/(app)/settings/profile', '/(app)/settings/plan']);
	const ownSettingsTop = $derived(!page.error && PHONE_SETTINGS.has(page.route.id ?? ''));
	const path = $derived(page.url.pathname);

	// Another page on its way: its server load runs before anything changes, so after a moment its
	// place shows a shimmering placeholder saying which page is coming, instead of the old page
	// sitting still. A quick one just swaps.
	const going = $derived(opensAnotherPage(page.url.pathname, navigating.to?.url.pathname) ? (navigating.to?.url.pathname ?? null) : null);
	let slow = $state(false);
	$effect(() => {
		if (!going) {
			slow = false;
			return;
		}
		const timer = setTimeout(() => (slow = true), LOADING_DELAY_MS);
		return () => clearTimeout(timer);
	});
	const LOADING_LABEL: Record<LoadingPage, () => string> = {
		home: () => m.app_loading_page({ page: m.nav_home().toLowerCase() }),
		chat: m.app_loading_chat,
		chats: () => m.app_loading_page({ page: m.nav_chats().toLowerCase() }),
		projects: () => m.app_loading_page({ page: m.nav_projects().toLowerCase() }),
		project: m.app_loading_project,
		inbox: () => m.app_loading_page({ page: m.nav_inbox().toLowerCase() }),
		settings: () => m.app_loading_page({ page: m.nav_settings().toLowerCase() }),
	};
	const loadingLabel = $derived.by(() => {
		const to = going && loadingPage(going);
		return to ? LOADING_LABEL[to]() : m.app_loading();
	});

	// Moving to another page cross-fades into it where the browser can (reduced motion: at once).
	onNavigate((navigation) => {
		if (!document.startViewTransition || !opensAnotherPage(navigation.from?.url.pathname ?? '', navigation.to?.url.pathname)) return;
		if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) return;
		return new Promise((resolve) => {
			document.startViewTransition(async () => {
				resolve();
				await navigation.complete;
			});
		});
	});

	// Home and a chat have the crew column: Home runs its top bar across it, a chat stops beside it.
	setPanelPlacement({
		get underTopbar() {
			return isHome;
		},
	});
	setMainLayout({
		get bleed() {
			return inThread || isInbox || isSettings || isProjects;
		},
	});
	// On a phone the top-level screens share the tab bar (Flow / Mobile Chat → Chats); a chat or a
	// new one is a screen inside Chats, with its own bar. Chats draws its own large title.
	const isChats = $derived(page.route.id === '/(app)/chats' && !page.error);
	const tabs = $derived([
		{ href: '/', label: m.nav_home(), icon: House, current: isHome },
		{ href: '/chats', label: m.nav_chats(), icon: MessageCircle, current: isChats },
		{ href: '/projects', label: m.nav_projects(), icon: Folder, current: isProjects },
		{ href: '/inbox', label: m.nav_inbox(), icon: Inbox, current: isInbox },
		{ href: '/settings', label: m.nav_you(), icon: User, current: isSettings },
	]);
	// The server's guess is only for the first paint; the width decides from then on.
	const guess = () => data.phone;
	const phone = phoneView(guess());
	// A screen inside a tab (a project opened from Projects) hides the tab bar as a chat does.
	/** Counts the person's chats moving on, for the phone's needs-you toast to look again. */
	let chatChanges = $state(0);
	const phoneScreenState = new PhoneScreen();
	setPhoneScreen(phoneScreenState);
	setPhoneChrome({
		get phone() {
			return phone.current;
		},
		get tabs() {
			return inThread || phoneScreenState.inside ? null : { label: m.nav_main(), tabs };
		},
		get ownTitle() {
			// Home, the Inbox and You draw their own large titles (Flow / Mobile — Home, Inbox & You);
			// You holds Settings' pages and Log out, so it needs no drawer: a settings page has a back link
			// to it instead.
			return isChats || isHome || inThread || isProjects || isInbox || isYou || ownSettingsTop;
		},
	});
	// What the chat page says about itself as it changes (renamed, pinned, moved, deleted).
	const header = new ChatHeader();
	setChatHeader(header);

	// The clock: from the minute the page was loaded, ticking; the browser's zone when none is saved.
	const loadedAt = () => new Date(data.now);
	let now = $state(loadedAt());
	let browserZone = $state<string | undefined>();
	const timeZone = $derived(data.timeZone ?? browserZone);
	setClock({
		get now() {
			return now;
		},
		get timeZone() {
			return timeZone;
		},
	});

	let theme = $derived<Theme>(data.theme);
	let unread = $derived(data.unread);
	let shortcutsOpen = $state(false);

	// This version's release notes (baked into the build for its own environment): they open
	// when the person clicks the version or What's new.
	let releaseNotesOpen = $state(false);
	const releaseDate = new Intl.DateTimeFormat(getLocale(), { day: 'numeric', month: 'short', year: 'numeric' });
	const releaseNotes = $derived(data.releaseNotes.map((release) => ({ version: release.version, url: release.url || undefined, html: release.html, date: release.publishedAt ? releaseDate.format(new Date(release.publishedAt)) : '' })));
	const storage = () => {
		try {
			return window.localStorage;
		} catch {
			return undefined;
		}
	};
	$effect(() => {
		// Seen once it's been opened, however it's closed.
		if (releaseNotesOpen) markSeen(APP_VERSION, storage());
	});
	let sessionOver = $state(false);
	let logout: HTMLFormElement | undefined = $state();

	const language = getLocale();
	const languages = [
		{ value: 'en', label: m.language_en() },
		{ value: 'id', label: m.language_id() },
	];
	const shortcuts = [
		{ label: m.search_shortcut(), keys: SHORTCUT_KEYS.search },
		{ label: m.nav_new_chat(), keys: SHORTCUT_KEYS['new-chat'] },
		{ label: m.nav_settings(), keys: SHORTCUT_KEYS.settings },
		{ label: m.workspace_settings(), keys: SHORTCUT_KEYS['workspace-settings'] },
		{ label: m.shortcut_show(), keys: SHORTCUT_KEYS.shortcuts },
		{ label: m.shortcut_close(), keys: 'esc' },
	];

	const pageTitle = $derived(page.data as { title?: string; meta?: string; autosave?: boolean; csv?: string; connect?: string; exportLink?: { href: string; label: string } });
	const leaving = $derived(new Set(header.leaving.map((id) => `/chat/${id}`)));
	const recentItems = $derived(data.recents.filter((recent) => !leaving.has(recent.href)).map((recent) => ({ ...recent, active: recent.href === path })));
	const chatData = $derived(page.data as { chat?: { id: string; pinned: boolean } });
	const projectsWorkspace = $derived(data.projectsWorkspace);
	const workspaceSettings = $derived(projectsWorkspace ? `/settings/workspaces/${projectsWorkspace.id}` : undefined);
	const signInAgain = $derived(`/sign-in?redirect_to=${encodeURIComponent(page.url.pathname + page.url.search)}`);

	/** Calls one of the shell's own endpoints; a redirect means the sign-in is gone, which isn't ok. */
	async function send(path: string, method: 'PUT' | 'POST', body: unknown): Promise<Response | null> {
		const response = await fetch(path, { method, headers: { 'content-type': 'application/json' }, body: JSON.stringify(body) }).catch(() => null);
		return response && !response.redirected ? response : null;
	}

	function applyTheme(next: Theme) {
		if (next === 'system') delete document.documentElement.dataset.theme;
		else document.documentElement.dataset.theme = next;
	}

	/** Applied at once; saved with the person's preferences; put back if that fails. */
	async function changeTheme(next: Theme) {
		const before = theme;
		theme = next;
		applyTheme(next);
		const response = await send('/preferences', 'PUT', { theme: next });
		if (response?.ok) return;
		theme = before;
		applyTheme(before);
		toasts.show({ kind: 'error', title: m.theme_failed() });
	}

	setThemeContext({
		get theme() {
			return theme;
		},
		changeTheme,
	});

	/** Saved with the person's preferences, then the page loads again in it. */
	async function changeLanguage(next: string) {
		const response = await send('/preferences', 'PUT', { language: next });
		if (response?.ok) location.reload();
		else toasts.show({ kind: 'error', title: m.language_failed() });
	}

	async function chooseWorkspace(id: string) {
		const name = data.workspaces.find((workspace) => workspace.id === id)?.name ?? '';
		const response = await send('/preferences/project-org', 'PUT', { org_id: id });
		if (!response?.ok) {
			toasts.show({ kind: 'error', title: m.workspace_choose_failed() });
			return;
		}
		await Promise.all([invalidate('app:shell'), invalidate('app:projects')]);
		toasts.show({ kind: 'success', title: m.workspace_chosen({ name }) });
	}

	// Create workspace: a name, sent to the backend; its reason shown when it says no.
	let creating = $state(false);
	let newName = $state('');
	let nameError = $state<string | undefined>();
	let saving = $state(false);

	function startCreating() {
		newName = '';
		nameError = undefined;
		creating = true;
	}

	async function createWorkspace() {
		const name = newName.trim();
		if (!name) {
			nameError = m.workspace_name_required();
			return;
		}
		saving = true;
		const response = await send('/workspaces', 'POST', { name });
		saving = false;
		if (response?.status === 201) {
			creating = false;
			await invalidate('app:shell');
			toasts.show({ kind: 'success', title: m.workspace_created({ name }) });
			return;
		}
		const said = ((await response?.json().catch(() => null)) ?? {}) as { field?: string | null; message?: string | null };
		if (said.field === 'required') nameError = m.workspace_name_required();
		else if (said.field === 'long') nameError = m.workspace_name_long();
		else nameError = said.message ? `${m.workspace_create_failed()}: ${said.message}` : m.workspace_create_failed();
	}

	// The Inbox's Mark all read, in the top bar: the sidebar's count and the page reload with it.
	let markingRead = $state(false);

	async function markAllRead() {
		markingRead = true;
		const result = await inboxAction('readAll');
		markingRead = false;
		if (!result.ok) {
			if (result.message) toasts.show({ kind: 'error', title: m.inbox_action_failed(), detail: result.message });
			return;
		}
		await invalidate('app:shell');
		toasts.show({ kind: 'success', title: m.inbox_all_read() });
	}

	// The settings nav: its pages in their groups, narrowed by its search.
	let settingsQuery = $state('');
	const settingsGroups = $derived(
		[
			{
				label: m.settings_account(),
				items: [
					{ href: '/settings/profile', icon: 'user' as const, label: m.settings_profile() },
					{ href: '/settings/general', icon: 'settings-2' as const, label: m.settings_general() },
					{ href: '/settings/plan', icon: 'credit-card' as const, label: m.settings_plan(), count: data.usage?.card.plan },
				],
			},
			{
				label: m.settings_ai(),
				items: [
					{ href: '/settings/crew', icon: 'users' as const, label: m.settings_crew() },
					{ href: '/settings/models', icon: 'cpu' as const, label: m.settings_models() },
					{ href: '/settings/memory', icon: 'brain' as const, label: m.settings_memory() },
				],
			},
			{
				label: m.settings_workspace(),
				items: [
					{ href: '/settings/workspaces', icon: 'building-2' as const, label: m.settings_workspaces() },
					{ href: '/settings/connections', icon: 'plug' as const, label: m.settings_connections() },
				],
			},
		]
			.map((group) => ({ ...group, items: group.items.filter((item) => item.label.toLowerCase().includes(settingsQuery.trim().toLowerCase())) }))
			.filter((group) => group.items.length > 0),
	);

	// Who's in the workspace projects come from, as the Projects top bar's faces (up to six).
	const PROJECT_TONES: ProjectMember['tone'][] = ['teal', 'orange', 'plain'];
	const projectMembers = $derived(
		((page.data.members ?? []) as { name: string }[]).slice(0, 6).map((member, index) => ({
			name: member.name,
			initials: member.name.split(/\s+/).map((word) => word.charAt(0)).join('').slice(0, 2).toUpperCase(),
			tone: PROJECT_TONES[index % PROJECT_TONES.length],
		})),
	);

	// The page's own item, or its parent's (a workspace's page lights Workspaces).
	const settingsActive = $derived(
		settingsGroups.flatMap((group) => group.items).find((item) => path === item.href || path.startsWith(`${item.href}/`))?.href ?? path,
	);

	function onkeydown(event: KeyboardEvent) {
		const { key, code, metaKey, ctrlKey, shiftKey, altKey } = event;
		// Esc leaves settings for the app, unless something open (a menu, a dialog) took it first.
		if (isSettings && key === 'Escape' && !event.defaultPrevented && !isTyping(event.target) && !document.querySelector('dialog[open]')) {
			void goto('/');
			return;
		}
		const shortcut = shortcutFor({ key, code, metaKey, ctrlKey, shiftKey, altKey, typing: isTyping(event.target) });
		if (!shortcut) return;
		event.preventDefault();
		if (shortcut === 'search') {
			// The field is in Home's top bar (and the app's list pages'); from anywhere else, Home.
			if (searchField) searchField.focus();
			else {
				focusSearchNext = true;
				void goto('/');
			}
		} else if (shortcut === 'new-chat') void goto('/chat/new');
		else if (shortcut === 'settings') void goto('/settings');
		else if (shortcut === 'workspace-settings' && workspaceSettings) void goto(workspaceSettings);
		else if (shortcut === 'shortcuts') shortcutsOpen = true;
	}

	// Search (App / Home's top bar, ⌘K): what's typed is looked up after a pause, and a later answer
	// for what's no longer typed is let go; a row opens where it lives.
	let query = $state('');
	let found = $state<SearchGroup[] | null>(null);
	let searching = $state(false);
	let searchFailed = $state(false);
	let searchField = $state<HTMLInputElement | null>(null);
	let searchTimer: ReturnType<typeof setTimeout> | undefined;
	let focusSearchNext = false;
	function lookUp(value: string) {
		clearTimeout(searchTimer);
		const wanted = value.trim();
		found = null;
		searchFailed = false;
		searching = Boolean(wanted);
		if (!wanted) return;
		searchTimer = setTimeout(async () => {
			const response = await fetch(`/search?q=${encodeURIComponent(wanted)}`).catch(() => null);
			if (query.trim() !== wanted) return;
			searching = false;
			if (!response?.ok) return void (searchFailed = true);
			found = searchGroups(((await response.json()) as { results: SearchHit[] }).results, new Date());
		}, 250);
	}
	function openFound(row: QuickSearchRow) {
		query = '';
		found = null;
		void goto(row.href);
	}
	$effect(() => {
		if (searchField && focusSearchNext) {
			focusSearchNext = false;
			searchField.focus();
		}
	});

	onMount(() => {
		browserZone = Intl.DateTimeFormat().resolvedOptions().timeZone;
		now = new Date();
		const tick = setInterval(() => (now = new Date()), 15_000);
		// A chat deleted with time to undo goes at once when the app is left.
		const away = () => later.flush();
		window.addEventListener('pagehide', away);

		// Bursts of chat events (a turn finishing in several chats) reload the shell once, after any
		// navigation under way (a new chat's own event would otherwise keep Home from opening it).
		const shell = settledRefresh(() => void invalidate('app:shell'), () => !!navigating.to);
		const refresh = shell.schedule;
		const stop = connectActivity({
			deviceId: data.deviceId,
			tab: tabId(),
			handlers: {
				chatsChanged: () => {
					refresh();
					chatChanges += 1;
				},
				resumed: refresh,
				notification(notification) {
					unread += 1;
					if (isInbox) void invalidate('app:inbox');
					const link = notification.link;
					toasts.show({
						kind: 'info',
						icon: Bell,
						title: notification.title,
						...(link && { action: { label: m.notification_open(), onclick: () => void goto(link) } }),
					});
				},
				async signedOutHere() {
					await fetch('/sign-out', { method: 'POST' }).catch(() => null);
					location.assign(signedOutTarget());
				},
				lost(code) {
					if (code === 4401) sessionOver = true;
				},
			},
		});
		return () => {
			clearInterval(tick);
			window.removeEventListener('pagehide', away);
			shell.cancel();
			stop();
		};
	});
</script>

<svelte:window {onkeydown} />

{#snippet sidebar()}
	{#if isSettings}
		<SettingsNav
			backHref="/"
			groups={settingsGroups}
			active={settingsActive}
			user={{ name: data.person.name, detail: data.usage?.account, avatar: data.person.avatar ?? undefined }}
			version={VERSION_LABEL}
			bind:query={settingsQuery}
			onwhatsnew={releaseNotes.length > 0 ? () => (releaseNotesOpen = true) : undefined}
			onlogout={() => logout?.requestSubmit()}
		/>
	{:else}
		<Sidebar newChatHref="/chat/new" newChatShortcut={SHORTCUT_KEYS['new-chat']}>
			<NavItem href="/" icon={House} label={m.nav_home()} active={path === '/'} />
			<NavItem href="/inbox" icon={Inbox} label={m.nav_inbox()} count={unread > 0 ? unread : undefined} active={path.startsWith('/inbox')} />
			<NavGroupLabel label={m.nav_talk()} />
			<NavItem href="/chats" icon={MessageSquare} label={m.nav_chats()} count={data.chats} active={path.startsWith('/chats')} />
			{#if projectsWorkspace}
				<NavGroupLabel label={m.nav_build()}>
					<WorkspaceSwitcher current={projectsWorkspace.name} workspaces={data.workspaces} settingsHref={workspaceSettings} onchoose={chooseWorkspace} oncreate={startCreating} />
				</NavGroupLabel>
			{:else}
				<NavGroupLabel label={m.nav_build()} />
			{/if}
			<NavItem href="/projects" icon={Folder} label={m.nav_projects()} count={projectsWorkspace?.projects} active={path.startsWith('/projects')} />
			<!-- LIFE › Money: left out until Money is part of v2 (AGENTS.md §9). -->
			{#snippet recents()}<RecentsList allHref="/chats" items={recentItems} />{/snippet}
			{#snippet footer()}
				{#if data.usage}<UsageCard plan={data.usage.card.plan} used={data.usage.card.used} caption={data.usage.card.caption} href="/settings/plan" />{/if}
				<AccountMenu
					name={data.person.name}
					avatar={data.person.avatar ?? undefined}
					plan={data.usage?.account}
					upgradeHref={data.usage?.canUpgrade ? '/settings/plan' : undefined}
					settingsHref="/settings"
					profileHref="/settings/profile"
					planHref="/settings/plan"
					{theme}
					ontheme={changeTheme}
					{language}
					{languages}
					onlanguage={changeLanguage}
					version={VERSION_LABEL}
					onshortcuts={() => (shortcutsOpen = true)}
					onwhatsnew={releaseNotes.length > 0 ? () => (releaseNotesOpen = true) : undefined}
					onlogout={() => logout?.requestSubmit()}
				/>
			{/snippet}
		</Sidebar>
	{/if}
{/snippet}

{#snippet search()}
	<!-- On a phone the bar keeps its room for the page's name; ⌘K isn't a phone's. -->
	<span class="max-sm:hidden">
		<QuickSearch
			bind:value={query}
			bind:field={searchField}
			groups={found}
			{searching}
			failed={searchFailed}
			shortcut={SHORTCUT_KEYS.search}
			labels={{ field: m.search_field(), groups: { chat: m.search_group_chat(), memory: m.search_group_memory(), file: m.search_group_file() }, empty: m.search_empty({ query: query.trim() }), searching: m.search_searching(), failed: m.search_failed() }}
			oninput={lookUp}
			onpick={openFound}
		/>
	</span>
{/snippet}

{#snippet themeButton()}
	<span class="lg:hidden">
		<ThemePickerButton {theme} ontheme={changeTheme} />
	</span>
{/snippet}

{#snippet topbar()}
	{#if isHome}
		<TopBar title={m.home_title()} meta={stamp(now, language, timeZone)}>
			{#snippet actions()}
				{@render search()}
				{@render themeButton()}
			{/snippet}
		</TopBar>
	{:else if isChat && chatData.chat}
		{@const id = chatData.chat.id}
		<ChatTopBar
			title={header.shown(id, 'title', pageTitle.title ?? '')}
			meta={header.shown(id, 'meta', pageTitle.meta) ?? undefined}
			pinned={header.shown(id, 'pinned', chatData.chat.pinned)}
			renaming={header.renaming}
			onrename={(name) => header.handlers?.rename(name)}
			oncancelrename={() => (header.renaming = false)}
			onpick={(action) => header.handlers?.pick(action)}
			onshare={() => header.handlers?.share()}
		/>
	{:else if isSettings}
		<!-- On a phone a settings page is a screen inside You: back to it, then where the person is. -->
		<div class="flex min-w-0 items-center gap-1">
		{#if phone.current}
			<a href="/settings" aria-label={m.phone_you_back()} class="-ml-2 inline-flex size-11 shrink-0 items-center justify-center rounded-full text-text outline-offset-1 focus-visible:outline-2 focus-visible:outline-focus"><Icon icon={ChevronLeft} size={22} /></a>
		{/if}
		<div class="min-w-0 flex-1"><SettingsTopBar section={m.settings_nav()} page={pageTitle.title ?? ''} meta={pageTitle.meta}>
			{#snippet actions()}
				{#if pageTitle.autosave}<AutosaveNote text={m.general_autosave()} />{/if}
				{#if pageTitle.csv}<Button label={m.plan_export()} variant="ghost" iconStart={Download} href={pageTitle.csv} data-sveltekit-reload />{/if}
				{#if pageTitle.exportLink}<Button label={pageTitle.exportLink.label} variant="secondary" iconEnd={Download} href={pageTitle.exportLink.href} data-sveltekit-reload />{/if}
				{#if pageTitle.connect}<Button label={m.models_connect()} variant="inverse" iconEnd={Plus} href={pageTitle.connect} data-sveltekit-noscroll />{/if}
				{@render themeButton()}
			{/snippet}
		</SettingsTopBar></div>
		</div>
	{:else if isProjects}
		<ProjectTopBar
			workspaces={data.workspaces}
			onchoose={chooseWorkspace}
			oncreate={startCreating}
			settingsHref={workspaceSettings}
			members={projectMembers}
			memberCount={(page.data.members as unknown[] | undefined)?.length}
			oninvite={workspaceSettings ? () => goto(workspaceSettings) : undefined}
		/>
	{:else if isInbox}
		<TopBar title={pageTitle.title ?? ''} meta={pageTitle.meta}>
			{#snippet actions()}
				<!-- On a phone it's the icon alone, so the bar keeps room for the page's name. -->
				<span class="max-sm:hidden"><Button label={m.inbox_mark_all_read()} variant="ghost" iconStart={CheckCheck} disabled={unread === 0} loading={markingRead} onclick={markAllRead} /></span>
				<span class="sm:hidden"><IconButton icon={CheckCheck} label={m.inbox_mark_all_read()} variant="ghost" disabled={unread === 0 || markingRead} onclick={markAllRead} /></span>
				<Button label={m.inbox_new_reminder()} variant="inverse" iconEnd={Plus} onclick={() => document.getElementById(QUICK_REMINDER)?.focus()} />
				{@render themeButton()}
			{/snippet}
		</TopBar>
	{:else if page.error}
		<TopBar title={page.route.id === CHAT ? m.nav_chats() : m.error_title()}>
			{#snippet actions()}{@render themeButton()}{/snippet}
		</TopBar>
	{:else}
		<!-- Other pages name themselves in their load: { title, meta }. -->
		<TopBar title={pageTitle.title ?? ''} meta={pageTitle.meta}>
			{#snippet actions()}
				{@render search()}
				{@render themeButton()}
			{/snippet}
		</TopBar>
	{/if}
{/snippet}

{#snippet crew()}
	<CrewPanel summary={data.crew.summary} members={isChat ? chatCrewRows(data.crew.rows, header.crew) : data.crew.rows} />
{/snippet}

{#snippet content()}
	{#if going && slow}<PageLoading label={loadingLabel} />{:else}{@render children()}{/if}
{/snippet}

{#if isWorkspace}
	{@render content()}
{:else}
	<AppShell {sidebar} {topbar} panel={isHome || inThread ? crew : undefined}>
		{@render content()}
	</AppShell>
{/if}

<form bind:this={logout} method="POST" action="/sign-out" hidden></form>

<ShortcutsDialog bind:open={shortcutsOpen} {shortcuts} />
<ReleaseNotesDialog bind:open={releaseNotesOpen} releases={releaseNotes} />

<Dialog bind:open={creating} title={m.workspace_create_title()} description={m.workspace_create_body()}>
	<form
		onsubmit={(event) => {
			event.preventDefault();
			void createWorkspace();
		}}
	>
		<TextField label={m.workspace_name()} bind:value={newName} error={nameError} required autocomplete="off" oninput={() => (nameError = undefined)} />
	</form>
	{#snippet actions()}
		<Button label={m.action_cancel()} variant="ghost" onclick={() => (creating = false)} />
		<Button label={m.workspace_create_action()} variant="inverse" loading={saving} onclick={createWorkspace} />
	{/snippet}
</Dialog>

{#if phone.current}
	<NeedsYouToast
		changes={chatChanges}
		openChat={path.startsWith('/chat/') ? path.split('/')[2] : null}
		titleOf={(sessionId) => data.recents.find((recent) => recent.href === `/chat/${sessionId}`)?.title ?? m.recents_untitled()}
		onopen={(sessionId) => void goto(`/chat/${sessionId}`)}
	/>
	<!-- On a phone (Mobile Chat — Errors & recovery, scenario 7): a sheet that can't be dismissed. -->
	<PhoneSignedOutSheet open={sessionOver || header.signedOutDraft !== null} draft={header.signedOutDraft || undefined} signInHref={signInAgain} />
{:else}
<Modal open={sessionOver || header.signedOutDraft !== null} labelledby="session-over-title">
	<!-- SessionExpired's heading has no id to point at (its props are the old app twin's). A message
	     sent as the sign-in ran out shows here; it waits on this device and goes after signing in. -->
	<span id="session-over-title" hidden>{m.session_title()}</span>
	<SessionExpired draft={header.signedOutDraft || undefined} signInHref={signInAgain} />
</Modal>
{/if}

<!-- A project's workspace on a phone shows the toasts at the top of its screen itself (PhoneToastRegion). -->
{#if !(isWorkspace && phone.current)}<ToastRegion />{/if}
