<script lang="ts">
	import { onMount } from 'svelte';
	import { Bell, CheckCheck, Download, Folder, House, Inbox, MessageSquare, Plus } from '@lucide/svelte';
	import { goto, invalidate } from '$app/navigation';
	import { page } from '$app/state';
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
	import { SHORTCUT_KEYS, isTyping, shortcutFor } from '$lib/shortcuts';
	import { VERSION_LABEL } from '$lib/version';
	import type { Theme } from '$lib/theme';
	import AppShell from '$lib/ds/shells/AppShell.svelte';
	import { setPanelPlacement } from '$lib/ds/shells/panelPlacement';
	import { setMainLayout } from '$lib/ds/shells/mainLayout';
	import Sidebar from '$lib/ds/chrome/Sidebar.svelte';
	import NavItem from '$lib/ds/navigation/NavItem.svelte';
	import NavGroupLabel from '$lib/ds/navigation/NavGroupLabel.svelte';
	import RecentsList from '$lib/ds/chrome/RecentsList.svelte';
	import UsageCard from '$lib/ds/chrome/UsageCard.svelte';
	import AccountMenu from '$lib/ds/chrome/AccountMenu.svelte';
	import WorkspaceSwitcher from '$lib/ds/chrome/WorkspaceSwitcher.svelte';
	import TopBar from '$lib/ds/chrome/TopBar.svelte';
	import ChatTopBar from '$lib/ds/thread/ChatTopBar.svelte';
	import SettingsNav from '$lib/ds/chrome/SettingsNav.svelte';
	import SettingsTopBar from '$lib/ds/chrome/SettingsTopBar.svelte';
	import AutosaveNote from '$lib/ds/settings/AutosaveNote.svelte';
	import CrewPanel from '$lib/ds/chrome/CrewPanel.svelte';
	import ShortcutsDialog from '$lib/ds/chrome/ShortcutsDialog.svelte';
	import SessionExpired from '$lib/ds/chat/SessionExpired.svelte';
	import Dialog from '$lib/ds/overlays/Dialog.svelte';
	import Modal from '$lib/ds/overlays/Modal.svelte';
	import ToastRegion from '$lib/ds/overlays/ToastRegion.svelte';
	import { toasts } from '$lib/ds/overlays/toasts.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import IconButton from '$lib/ds/actions/IconButton.svelte';
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
	const isHome = $derived(page.route.id === HOME);
	// A chat that couldn't open shows the error page, laid out as a page.
	const isChat = $derived(page.route.id === CHAT && !page.error);
	const inThread = $derived(isChat || page.route.id === NEW_CHAT);
	const isInbox = $derived(page.route.id === INBOX && !page.error);
	const isSettings = $derived(!!page.route.id?.startsWith('/(app)/settings') && !page.error);
	const path = $derived(page.url.pathname);

	// Home and a chat have the crew column: Home runs its top bar across it, a chat stops beside it.
	setPanelPlacement({
		get underTopbar() {
			return isHome;
		},
	});
	setMainLayout({
		get bleed() {
			return inThread || isInbox || isSettings;
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
	let sessionOver = $state(false);
	let logout: HTMLFormElement | undefined = $state();

	const language = getLocale();
	const languages = [
		{ value: 'en', label: m.language_en() },
		{ value: 'id', label: m.language_id() },
	];
	const shortcuts = [
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
		await invalidate('app:shell');
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
					{ href: '/settings/models', icon: 'cpu' as const, label: m.settings_models() },
					{ href: '/settings/memory', icon: 'brain' as const, label: m.settings_memory() },
				],
			},
			{
				label: m.settings_workspace(),
				items: [{ href: '/settings/connections', icon: 'plug' as const, label: m.settings_connections() }],
			},
		]
			.map((group) => ({ ...group, items: group.items.filter((item) => item.label.toLowerCase().includes(settingsQuery.trim().toLowerCase())) }))
			.filter((group) => group.items.length > 0),
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
		if (shortcut === 'new-chat') void goto('/chat/new');
		else if (shortcut === 'settings') void goto('/settings');
		else if (shortcut === 'workspace-settings' && workspaceSettings) void goto(workspaceSettings);
		else if (shortcut === 'shortcuts') shortcutsOpen = true;
	}

	onMount(() => {
		browserZone = Intl.DateTimeFormat().resolvedOptions().timeZone;
		now = new Date();
		const tick = setInterval(() => (now = new Date()), 15_000);
		// A chat deleted with time to undo goes at once when the app is left.
		const away = () => later.flush();
		window.addEventListener('pagehide', away);

		// Bursts of chat events (a turn finishing in several chats) reload the shell once.
		let pending: ReturnType<typeof setTimeout> | undefined;
		const refresh = () => {
			clearTimeout(pending);
			pending = setTimeout(() => invalidate('app:shell'), 400);
		};
		const stop = connectActivity({
			deviceId: data.deviceId,
			tab: tabId(),
			handlers: {
				chatsChanged: refresh,
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
					location.assign('/sign-in');
				},
				lost(code) {
					if (code === 4401) sessionOver = true;
				},
			},
		});
		return () => {
			clearInterval(tick);
			window.removeEventListener('pagehide', away);
			clearTimeout(pending);
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
			active={path}
			user={{ name: data.person.name, detail: data.usage?.account, avatar: data.person.avatar ?? undefined }}
			version={VERSION_LABEL}
			bind:query={settingsQuery}
			onlogout={() => logout?.requestSubmit()}
		/>
	{:else}
		<Sidebar newChatHref="/chat/new">
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
					onlogout={() => logout?.requestSubmit()}
				/>
			{/snippet}
		</Sidebar>
	{/if}
{/snippet}

{#snippet topbar()}
	{#if isHome}
		<TopBar title={m.home_title()} meta={stamp(now, language, timeZone)} />
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
		<SettingsTopBar section={m.settings_nav()} page={pageTitle.title ?? ''} meta={pageTitle.meta}>
			{#snippet actions()}
				{#if pageTitle.autosave}<AutosaveNote text={m.general_autosave()} />{/if}
				{#if pageTitle.csv}<Button label={m.plan_export()} variant="ghost" iconStart={Download} href={pageTitle.csv} data-sveltekit-reload />{/if}
				{#if pageTitle.exportLink}<Button label={pageTitle.exportLink.label} variant="secondary" iconEnd={Download} href={pageTitle.exportLink.href} data-sveltekit-reload />{/if}
				{#if pageTitle.connect}<Button label={m.models_connect()} variant="inverse" iconEnd={Plus} href={pageTitle.connect} data-sveltekit-noscroll />{/if}
			{/snippet}
		</SettingsTopBar>
	{:else if isInbox}
		<TopBar title={pageTitle.title ?? ''} meta={pageTitle.meta}>
			{#snippet actions()}
				<!-- On a phone it's the icon alone, so the bar keeps room for the page's name. -->
				<span class="max-sm:hidden"><Button label={m.inbox_mark_all_read()} variant="ghost" iconStart={CheckCheck} disabled={unread === 0} loading={markingRead} onclick={markAllRead} /></span>
				<span class="sm:hidden"><IconButton icon={CheckCheck} label={m.inbox_mark_all_read()} variant="ghost" disabled={unread === 0 || markingRead} onclick={markAllRead} /></span>
				<Button label={m.inbox_new_reminder()} variant="inverse" iconEnd={Plus} onclick={() => document.getElementById(QUICK_REMINDER)?.focus()} />
			{/snippet}
		</TopBar>
	{:else if page.error}
		<TopBar title={page.route.id === CHAT ? m.nav_chats() : m.error_title()} />
	{:else}
		<!-- Other pages name themselves in their load: { title, meta }. -->
		<TopBar title={pageTitle.title ?? ''} meta={pageTitle.meta} />
	{/if}
{/snippet}

{#snippet crew()}
	<CrewPanel summary={data.crew.summary} members={isChat ? chatCrewRows(data.crew.rows, header.crew) : data.crew.rows} />
{/snippet}

<AppShell {sidebar} {topbar} panel={isHome || inThread ? crew : undefined}>
	{@render children()}
</AppShell>

<form bind:this={logout} method="POST" action="/sign-out" hidden></form>

<ShortcutsDialog bind:open={shortcutsOpen} {shortcuts} />

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

<Modal open={sessionOver || header.signedOutDraft !== null} labelledby="session-over-title">
	<!-- SessionExpired's heading has no id to point at (its props are the old app twin's). A message
	     sent as the sign-in ran out shows here; it waits on this device and goes after signing in. -->
	<span id="session-over-title" hidden>{m.session_title()}</span>
	<SessionExpired draft={header.signedOutDraft || undefined} signInHref={signInAgain} />
</Modal>

<ToastRegion />
