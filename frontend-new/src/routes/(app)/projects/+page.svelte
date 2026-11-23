<script lang="ts">
	import { Activity, ArrowDownAZ, Building2, Calendar, CircleCheck, FolderPlus, MessageCircle, Wrench, CalendarCheck, CircleAlert, Clock, FileText, FolderSearch, Plus, SearchX, Settings, Store, X } from '@lucide/svelte';
	import { onMount, tick } from 'svelte';
	import { goto, invalidate } from '$app/navigation';
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import { pageAction } from '$lib/pageAction';
	import { filterCounts, projectCards, runningFor, visibleCards, type ProjectCardData } from '$lib/projects/list';
	import { alertTitle, databaseName, detailLine, liveLine, moveTargets, panelParts, projectChats, took } from '$lib/projects/about';
	import type { DeployOverview } from '$lib/adapters/api';
	import type { MoveChoices, ProjectAbout } from '$lib/server/projects';
	import { toasts } from '$lib/ds/overlays/toasts.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import ProjectPageHeader from '$lib/ds/projects/ProjectPageHeader.svelte';
	import ProjectComposer from '$lib/ds/projects/ProjectComposer.svelte';
	import ProjectToolbar, { type ProjectFilter } from '$lib/ds/projects/ProjectToolbar.svelte';
	import type { ProjectSort } from '$lib/ds/projects/SortMenu.svelte';
	import ProjectCard from '$lib/ds/projects/ProjectCard.svelte';
	import type { ProjectActions } from '$lib/ds/projects/ProjectCardMenu.svelte';
	import PhoneProjectsHeader from '$lib/ds/projects/PhoneProjectsHeader.svelte';
	import PhoneProjectSearch from '$lib/ds/projects/PhoneProjectSearch.svelte';
	import PhoneFilters from '$lib/ds/projects/PhoneFilters.svelte';
	import PhoneProjectDetail from '$lib/ds/projects/PhoneProjectDetail.svelte';
	import PhoneAskBar from '$lib/ds/projects/PhoneAskBar.svelte';
	import PhoneProjectEmpty from '$lib/ds/projects/PhoneProjectEmpty.svelte';
	import PhoneProjectMenu from '$lib/ds/projects/PhoneProjectMenu.svelte';
	import PhoneWorkspaceList from '$lib/ds/projects/PhoneWorkspaceList.svelte';
	import ActionSheet from '$lib/ds/overlays/ActionSheet.svelte';
	import DeploySheet from '$lib/ds/projects/DeploySheet.svelte';
	import DeleteProjectSheet from '$lib/ds/projects/DeleteProjectSheet.svelte';
	import ProjectMoveSheet from '$lib/ds/projects/ProjectMoveSheet.svelte';
	import BottomSheet from '$lib/ds/overlays/BottomSheet.svelte';
	import SheetRow from '$lib/ds/mobile/SheetRow.svelte';
	import SheetOption from '$lib/ds/overlays/SheetOption.svelte';
	import { phoneView } from '$lib/phone.svelte';
	import { phoneScreen } from '$lib/ds/shells/phoneScreen.svelte';
	import ProjectCardLoading from '$lib/ds/projects/ProjectCardLoading.svelte';
	import ProjectEmpty from '$lib/ds/projects/ProjectEmpty.svelte';
	import FirstProject from '$lib/ds/projects/FirstProject.svelte';
	import LoadError from '$lib/ds/projects/LoadError.svelte';
	import ProjectDialog from '$lib/ds/projects/ProjectDialog.svelte';
	import StopDialog from '$lib/ds/projects/StopDialog.svelte';
	import DeployDialog from '$lib/ds/projects/DeployDialog.svelte';
	import ProjectMoveDialog, { type MoveTarget } from '$lib/ds/projects/ProjectMoveDialog.svelte';
	import ProjectDetailPanel from '$lib/ds/projects/ProjectDetailPanel.svelte';
	import AskField from '$lib/ds/projects/AskField.svelte';

	// Projects (App / Projects JyWON): the projects of the workspace they come from. The composer
	// starts one (its chat opens with Koda on it); the toolbar filters, finds and sorts; each card
	// says where the project is and carries its one next action (Flow / Projects — Act from the card
	// syolc): Stop while Koda works (asked first), Resume once stopped, Deploy for a draft (its
	// facts first; the card then follows the publish to live). Its menu (Card menu M9O0iU) opens,
	// visits, moves it to another workspace, exports, deletes (asked first), and opens the
	// workspace's own Share, Settings and Take down. Selecting a card shows it in the detail panel
	// (Detail panel pgSon): what's in it, its chats, and asking Koda for a change; × or Escape closes
	// it. Before any project, the starters fill the composer; before a workspace is chosen, the page
	// asks for one; when the list can't be read it says so with Retry, and the composer still works
	// (Loading, offline & errors c6fFB). Left out (AGENTS.md §9, BACKEND_TODO.md): Community,
	// importing, rename and duplicate, failed and down states, and the panel's picture of the app.
	let { data } = $props();

	const PATH = '/projects';
	let now = $state(new Date());
	// Projects with a publish started here, and since when: the card follows it until it's done.
	let deploying = $state<Record<string, string>>({});
	const active = $derived(new Set(data.chats.filter((chat) => chat.own && chat.active).map((chat) => chat.id)));
	const cards = $derived(projectCards(data.list?.projects ?? [], now, { active, deploying, publishing: data.publishing === true }));
	let filter = $state<ProjectFilter>('all');
	let query = $state('');
	let sort = $state<ProjectSort>('updated');
	const shown = $derived(visibleCards(cards, filter, query, sort));
	const counts = $derived(filterCounts(cards));

	const orgId = $derived(data.list?.org_id ?? page.data.workspaces?.find((workspace: { chosen: boolean }) => workspace.chosen)?.id ?? null);
	const workspaceName = $derived(page.data.workspaces?.find((workspace: { id: string }) => workspace.id === orgId)?.name ?? '');
	const canMove = $derived((page.data.workspaces?.length ?? 0) > 1);
	// Each tab's own empty state (App / Projects → Empty / Tab eI37L draws Needs you's).
	const TAB_EMPTY: Record<ProjectFilter, { title: () => string; body: () => string }> = {
		all: { title: m.projects_tab_empty_title, body: m.projects_tab_empty_body },
		live: { title: m.projects_live_empty_title, body: m.projects_live_empty_body },
		building: { title: m.projects_building_empty_title, body: m.projects_building_empty_body },
		attention: { title: m.projects_attention_empty_title, body: m.projects_attention_empty_body },
	};

	// Back from a project deleted or moved away in its workspace: say so, then tidy the address.
	onMount(() => {
		const deleted = page.url.searchParams.get('deleted');
		const moved = page.url.searchParams.get('moved');
		if (!deleted && !moved) return;
		toasts.show({ kind: 'success', title: deleted ? m.projects_deleted({ name: deleted }) : m.projects_moved({ name: moved ?? '' }) });
		void goto(PATH, { replaceState: true, noScroll: true, keepFocus: true });
	});

	// While Koda works on a card or a publish runs, the page looks again: every 3 s for a publish
	// (its time runs on the card), every 5 s for the list while a card shows Koda at work.
	onMount(() => {
		let beats = 0;
		const timer = setInterval(() => {
			beats += 1;
			const ids = Object.keys(deploying);
			if (ids.length) now = new Date();
			if (document.visibilityState !== 'visible') return;
			if (ids.length && beats % 3 === 0) for (const id of ids) void followDeploy(id);
			if (beats % 5 === 0 && cards.some((card) => card.next === 'stop')) void invalidate('app:projects');
		}, 1000);
		return () => clearInterval(timer);
	});

	let description = $state('');
	let starting = $state(false);
	async function build(text: string) {
		if (starting) return;
		starting = true;
		const result = await pageAction(PATH, 'start', { description: text, org: orgId ?? '' });
		starting = false;
		if (!result.ok && result.message) toasts.show({ kind: 'error', title: m.projects_start_failed(), detail: result.message });
	}

	const starters = [
		{ icon: CalendarCheck, label: m.projects_starter_booking(), meta: m.projects_starter_app_db() },
		{ icon: Store, label: m.projects_starter_warung(), meta: m.projects_starter_app_db() },
		{ icon: FileText, label: m.projects_starter_portfolio(), meta: m.projects_starter_site() },
	].map((starter) => ({ ...starter, onpick: (label: string) => (description = label) }));

	// The list couldn't be read: Retry shows the cards' places while it's read again.
	let retrying = $state(false);
	async function retry() {
		retrying = true;
		await invalidate('app:projects');
		retrying = false;
	}
	const failedAt = $derived(data.failure ? new Intl.DateTimeFormat('en-GB', { hour: '2-digit', minute: '2-digit', hour12: false }).format(new Date(data.failure.at)) : '');

	// An action that didn't go through says why; an empty message is a sign-in being renewed, said nothing of.
	function failed(message: string | undefined, title = m.action_failed()) {
		if (message === '') return;
		toasts.show({ kind: 'error', title, ...(message && message !== title && { detail: message }) });
	}

	// The detail panel: what's in the selected project, read when it's selected.
	let selectedId = $state<string | null>(null);
	const selected = $derived(cards.find((card) => card.id === selectedId) ?? null);
	let about = $state<Record<string, ProjectAbout | null>>({});
	async function readAbout(id: string): Promise<ProjectAbout | null> {
		const response = await fetch(`/projects/${id}/about`).catch(() => null);
		const read = response?.ok ? ((await response.json()) as ProjectAbout) : null;
		about[id] = read;
		return read;
	}
	function select(card: ProjectCardData) {
		selectedId = card.id;
		asking = false;
		if (!(card.id in about)) void readAbout(card.id);
	}
	// On a desktop wide enough for the list and the panel, the first project shows in it, as drawn.
	onMount(() => {
		if (window.matchMedia('(min-width: 80rem)').matches && shown[0]) select(shown[0]);
	});
	function closePanel() {
		const id = selectedId;
		selectedId = null;
		asking = false;
		if (id) void tick().then(() => document.querySelector<HTMLElement>(`[data-project="${id}"] h3 button`)?.focus());
	}
	function onkeydown(event: KeyboardEvent) {
		if (event.key !== 'Escape' || !selectedId || event.defaultPrevented || document.querySelector('dialog[open]')) return;
		if (asking) asking = false;
		else closePanel();
	}

	// Asking from the panel (step 2): the change goes to Koda in the project's chat; the card shows the work.
	let asking = $state(false);
	let question = $state('');
	async function startAsking() {
		asking = true;
		await tick();
		document.querySelector<HTMLTextAreaElement>('[data-asking] textarea')?.focus();
	}
	async function sendAsk(card: ProjectCardData, text: string) {
		asking = false;
		const result = await pageAction(PATH, 'ask', { session: card.sessionId, text });
		if (!result.ok) return failed(result.message);
		await invalidate('app:projects');
	}

	// Stop (step 1): asked first; the toast offers Resume.
	let stopOpen = $state(false);
	let stopCard = $state<ProjectCardData | null>(null);
	async function stop(card: ProjectCardData) {
		const result = await pageAction(PATH, 'stop', { session: card.sessionId });
		if (!result.ok) return failed(result.message);
		toasts.show({ kind: 'info', title: m.projects_stopped_toast({ name: card.name }), detail: m.projects_stopped_toast_detail(), action: { label: m.projects_action_resume(), onclick: () => void resume(card) } });
		await invalidate('app:projects');
	}
	async function resume(card: ProjectCardData) {
		const result = await pageAction(PATH, 'resume', { session: card.sessionId });
		if (!result.ok) return failed(result.message);
		await invalidate('app:projects');
	}

	// Deploy (steps 6–8): its facts first, then the card follows the publish until it's live.
	let deployOpen = $state(false);
	let deployCard = $state<ProjectCardData | null>(null);
	let deployData = $state('');
	let started: Record<string, number> = {};
	async function askDeploy(card: ProjectCardData) {
		const read = card.id in about ? about[card.id] : await readAbout(card.id);
		const database = read?.database;
		deployData = database === null || database === undefined ? m.projects_part_unknown() : (databaseName(database) ?? m.projects_data_none());
		deployCard = card;
		deployOpen = true;
	}
	async function deploy(card: ProjectCardData) {
		const result = await pageAction(PATH, 'deploy', { id: card.id });
		if (!result.ok) return failed(result.message, m.deploy_couldnt_start());
		started[card.id] = Date.now();
		deploying[card.id] = new Date().toISOString();
	}
	async function followDeploy(id: string) {
		const card = cards.find((one) => one.id === id);
		const response = await fetch(`/projects/${id}/deploy`).catch(() => null);
		if (!card || !response?.ok) return;
		const overview = (await response.json()) as DeployOverview;
		const since = (started[id] ?? 0) - 5000;
		const run = overview.publishes.find((one) => one.environment === 'production' && Date.parse(one.created_at) >= since);
		if (!run || run.status === 'running' || overview.backend_setup) {
			if (run) deploying[id] = run.created_at;
			return;
		}
		delete deploying[id];
		delete started[id];
		if (run.status === 'succeeded') {
			const host = (overview.url ?? run.url ?? '').replace(/^https?:\/\//, '').replace(/\/+$/, '');
			const time = run.finished_at ? took(Math.max(0, Math.round((Date.parse(run.finished_at) - Date.parse(run.created_at)) / 1000))) : runningFor(run.created_at, new Date());
			toasts.show({
				kind: 'success',
				title: m.projects_live_toast({ name: card.name }),
				detail: m.projects_live_toast_detail({ url: host, time }),
				...(host && { action: { label: m.projects_copy_url(), onclick: () => void navigator.clipboard?.writeText(`https://${host}`).then(() => toasts.show({ kind: 'success', title: m.projects_url_copied() })) } }),
			});
		} else {
			failed(run.error ?? undefined, m.projects_deploy_failed_toast({ name: card.name }));
		}
		delete about[id];
		await invalidate('app:projects');
	}

	// Move (step 5): the workspaces it can go to; picking one and Move project moves it at once.
	let moveOpen = $state(false);
	let moveCard = $state<ProjectCardData | null>(null);
	let moveChoices = $state<MoveTarget[]>([]);
	let moveValue = $state<string>();
	async function askMove(card: ProjectCardData) {
		const response = await fetch(`/projects/${card.id}/move`).catch(() => null);
		if (!response?.ok) return failed(undefined);
		const choices = (await response.json()) as MoveChoices;
		moveChoices = moveTargets(choices);
		moveValue = undefined;
		moveCard = card;
		moveOpen = true;
	}
	async function move(card: ProjectCardData, target: string) {
		const workspace = moveChoices.find((one) => one.id === target)?.name ?? '';
		const result = await pageAction(PATH, 'move', { id: card.id, target, name: card.name });
		if (!result.ok) return failed(result.message);
		if (selectedId === card.id) selectedId = null;
		toasts.show({ kind: 'success', title: m.projects_moved_to({ name: card.name, workspace }) });
		await invalidate('app:projects');
	}

	let deleting = $state<ProjectCardData | null>(null);
	let busy = $state(false);
	async function remove() {
		if (!deleting) return;
		busy = true;
		const name = deleting.name;
		const id = deleting.id;
		const result = await pageAction(PATH, 'delete', { id });
		busy = false;
		if (!result.ok) return failed(result.message);
		deleting = null;
		if (selectedId === id) selectedId = null;
		toasts.show({ kind: 'success', title: m.projects_deleted({ name }) });
		await invalidate('app:projects');
	}

	// What needs the person: Answer opens the project's chat where Koda waits; Fix and Check say it
	// to Koda there, and the toast leads to it.
	async function askKoda(card: ProjectCardData, what: 'fix' | 'check') {
		const result = await pageAction(PATH, what, { session: card.sessionId });
		if (!result.ok) return failed(result.message);
		toasts.show({ kind: 'success', title: m.projects_asked_koda({ name: card.name }), action: { label: m.projects_action_open(), onclick: () => void goto(card.href) } });
		await invalidate('app:projects');
	}

	// Rename (F2 or the menu) on the card itself, and Duplicate (⌘D or the menu).
	let renamingId = $state<string | null>(null);
	async function rename(card: ProjectCardData, name: string) {
		const wanted = name.trim();
		renamingId = null;
		if (!wanted || wanted === card.name) return;
		const result = await pageAction(PATH, 'rename', { id: card.id, name: wanted });
		if (!result.ok) return failed(result.message);
		toasts.show({ kind: 'success', title: m.projects_renamed({ name: wanted }) });
		await invalidate('app:projects');
	}
	async function duplicate(card: ProjectCardData) {
		const result = await pageAction(PATH, 'duplicate', { id: card.id });
		if (!result.ok) return failed(result.message);
		const copy = String(result.data.duplicated ?? '');
		toasts.show({ kind: 'success', title: m.projects_duplicated({ name: card.name }), ...(copy && { action: { label: m.projects_duplicate_open(), onclick: () => void goto(`/projects/${copy}`) } }) });
		await invalidate('app:projects');
	}

	function action(card: ProjectCardData): (() => void) | undefined {
		if (card.next === 'stop')
			return () => {
				stopCard = card;
				stopOpen = true;
			};
		if (card.next === 'resume') return () => void resume(card);
		if (card.next === 'deploy') return () => void askDeploy(card);
		if (card.next === 'answer') return () => void goto(card.href);
		if (card.next === 'fix' || card.next === 'check') {
			const what = card.next;
			return () => void askKoda(card, what);
		}
		return undefined;
	}
	const workspace = (card: ProjectCardData, open: string) => () => goto(`${card.href}?open=${open}`);
	/** What a card's menu (a desktop) or its sheet (a phone) can do to it. */
	function cardActions(card: ProjectCardData): ProjectActions {
		return {
			onopen: () => goto(card.href),
			...(card.url && { onvisit: () => window.open(`https://${card.url}`, '_blank', 'noopener') }),
			onrename: () => (renamingId = card.id),
			onduplicate: () => void duplicate(card),
			...(canMove && { onmove: () => void askMove(card) }),
			...(card.liveUrl && { onshare: workspace(card, 'share') }),
			onexport: workspace(card, 'export'),
			onsettings: workspace(card, 'settings'),
			...(card.url && { ontakedown: workspace(card, 'takedown') }),
			ondelete: () => (deleting = card),
		};
	}

	// On a phone it's Flow / Mobile Projects: the header (the workspace, its people, Invite, the large
	// title and New), the search with Sort, the filters as chips (Needs you first when something
	// does), and each project a Mobile / Project Card, whose ⋯ opens its sheet. New, Sort and the
	// workspace open sheets too. The detail screen and the dialogs' sheets come with the next flows.
	const phoneGuess = () => Boolean(page.data.phone);
	const phone = phoneView(phoneGuess());
	type PhoneSheet = { kind: 'card'; card: ProjectCardData } | { kind: 'new' } | { kind: 'sort' } | { kind: 'workspace' };
	let phoneSheet = $state<PhoneSheet | null>(null);
	// A project's own screen is inside the Projects tab: no tab bar under it (the pen draws none).
	const screen = phoneScreen();
	$effect(() => {
		screen.inside = phone.current && selectedId !== null;
		return () => (screen.inside = false);
	});
	/** A card's sheet, its state read for the subtitle ("live · deploy #15 · 2d ago"). */
	function openCardSheet(card: ProjectCardData) {
		phoneSheet = { kind: 'card', card };
		if (!(card.id in about)) void readAbout(card.id);
	}
	const sheetClosed = (kind: PhoneSheet['kind']) => () => phoneSheet?.kind === kind && (phoneSheet = null);
	// The chips count what the search finds (Find & filter wDi32); Needs you leads while anything does.
	const phoneCounts = $derived(filterCounts(cards, query));
	const filterOptions = $derived([
		...(phoneCounts.attention > 0 ? [{ value: 'attention', label: m.projects_phone_needs_you(), count: phoneCounts.attention }] : []),
		{ value: 'all', label: m.projects_filter_all(), count: phoneCounts.all },
		{ value: 'live', label: m.projects_filter_live(), count: phoneCounts.live },
		{ value: 'building', label: m.projects_filter_building(), count: phoneCounts.building },
		...(phoneCounts.attention > 0 ? [] : [{ value: 'attention', label: m.projects_phone_needs_you(), count: phoneCounts.attention }]),
	]);
	const workspaceHref = $derived(orgId ? `/settings/workspaces/${orgId}` : null);
	type WorkspaceRow = { id: string; name: string; meta: string; chosen: boolean };
	const SORT_ICON = { updated: Clock, status: CircleAlert, name: ArrowDownAZ, created: Calendar } as const;
	// The phone's sort sheet (Find & filter → Sort M7SCRS): its own order and words.
	const PHONE_SORTS: { value: ProjectSort; label: () => string }[] = [
		{ value: 'updated', label: m.projects_sort_updated },
		{ value: 'status', label: m.projects_phone_sort_status },
		{ value: 'name', label: m.projects_phone_sort_name },
		{ value: 'created', label: m.projects_sort_created },
	];
	const workspaces = $derived((page.data.workspaces ?? []) as WorkspaceRow[]);
	async function chooseWorkspace(id: string) {
		const name = workspaces.find((one) => one.id === id)?.name ?? '';
		const response = await fetch('/preferences/project-org', { method: 'PUT', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ org_id: id }) }).catch(() => null);
		if (!response?.ok) return void toasts.show({ kind: 'error', title: m.workspace_choose_failed() });
		await Promise.all([invalidate('app:shell'), invalidate('app:projects')]);
		toasts.show({ kind: 'success', title: m.workspace_chosen({ name }) });
	}
</script>

<svelte:head><title>{m.nav_projects()} · {m.app_name()}</title></svelte:head>
<svelte:window {onkeydown} />

<!-- The phone's own empty list (Find & filter → No match OC2PS, Empty / Tab uDnnd): no box, a pill to clear. -->
{#snippet phoneEmpty()}
	{#if query.trim()}
		<PhoneProjectEmpty
			icon={SearchX}
			title={m.projects_no_match_title({ query: query.trim() })}
			body={m.projects_no_match_body({ workspace: workspaceName })}
			action={{ label: m.projects_clear_search(), icon: X, onclick: () => (query = '') }}
		/>
	{:else}
		<PhoneProjectEmpty icon={filter === 'attention' ? CircleCheck : FolderSearch} title={TAB_EMPTY[filter].title()} body={TAB_EMPTY[filter].body()} />
	{/if}
{/snippet}

{#if phone.current && selected}
	{@render phoneDetail(selected)}
	{@render phoneSheets()}
{:else if phone.current}
	<div class="flex h-full flex-col overflow-hidden">
		<div class="shrink-0 bg-bg">
			<PhoneProjectsHeader
				title={m.nav_projects()}
				workspace={workspaceName}
				members={(page.data.members as unknown[] | undefined)?.length}
				labels={{ workspace: m.projects_phone_workspace({ name: workspaceName }), members: m.projects_phone_people(), invite: m.projects_phone_invite(), new: m.projects_phone_new() }}
				onworkspace={workspaces.length ? () => (phoneSheet = { kind: 'workspace' }) : undefined}
				oninvite={workspaceHref ? () => goto(workspaceHref!) : undefined}
				onnew={orgId ? () => (phoneSheet = { kind: 'new' }) : undefined}
			/>
			{#if orgId && (retrying || !data.list)}
				<!-- Loading and couldn't load keep the search and filters, waiting (Mobile Projects — Loading, offline & errors). -->
				<PhoneProjectSearch placeholder={m.projects_search()} sortLabel={m.projects_phone_sort()} bind:value={query} />
				<PhoneFilters label={m.projects_filter()} options={filterOptions.map(({ count: _, ...option }) => option)} value={filter} onchange={(value) => (filter = value as ProjectFilter)} />
			{:else if orgId && cards.length}
				<PhoneProjectSearch placeholder={m.projects_search()} sortLabel={m.projects_phone_sort()} bind:value={query} onsort={() => (phoneSheet = { kind: 'sort' })} />
				<PhoneFilters label={m.projects_filter()} options={filterOptions} value={filter} onchange={(value) => (filter = value as ProjectFilter)} />
			{/if}
		</div>
		<div class="min-h-0 flex-1 overflow-y-auto pb-6">
			{#if !orgId}
				<div class="px-4">
					<ProjectEmpty
						icon={Building2}
						title={m.projects_no_workspace_title()}
						body={m.projects_no_workspace_body()}
						action={{ label: m.projects_choose_workspace(), onclick: () => goto('/settings/workspaces') }}
					/>
				</div>
			{:else if retrying || !data.list}
				{#if retrying}
					<ul class="flex flex-col gap-3 px-4">
						{#each { length: 3 }, index (index)}<li><ProjectCardLoading phone /></li>{/each}
					</ul>
				{:else}
					<div class="px-4"><LoadError phone reason={m.projects_load_failed_reason({ status: data.failure?.status ?? 503, time: failedAt })} onretry={retry} /></div>
				{/if}
			{:else if cards.length}
				{#if shown.length}
					<ul class="flex flex-col gap-3 px-4">
						{#each shown as card (card.id)}
							<li data-project={card.id}>
								<ProjectCard
									phone
									name={card.name}
									description={card.description}
									tone={card.tone}
									status={card.status}
									statusLabel={card.statusLabel}
									meta={card.meta}
									detail={card.detail}
									url={card.url}
									footer={card.footer}
									href={card.href}
									questions={card.questions}
									onaction={action(card)}
									renaming={renamingId === card.id}
									onrename={(name) => void rename(card, name)}
									oncancelrename={() => (renamingId = null)}
									onmore={() => openCardSheet(card)}
									selected={card.id === selectedId}
									onselect={() => select(card)}
								/>
							</li>
						{/each}
					</ul>
				{:else}
					<div class="px-4">{@render phoneEmpty()}</div>
				{/if}
			{:else}
				<div class="px-4 pt-2">
					<ProjectEmpty icon={FolderPlus} title={m.projects_phone_first_title()} body={m.projects_phone_first_body()} action={{ label: m.projects_phone_new_title(), onclick: () => (phoneSheet = { kind: 'new' }) }} />
				</div>
			{/if}
		</div>
	</div>
	{@render phoneSheets()}
{:else}
<div class="flex h-full">
	<div class="min-w-0 flex-1 overflow-y-auto">
		<div class="flex flex-col gap-6 px-4 pt-8 pb-6 lg:px-8">
			<ProjectPageHeader />
			{#if !orgId}
				<ProjectEmpty
					icon={Building2}
					title={m.projects_no_workspace_title()}
					body={m.projects_no_workspace_body()}
					action={{ label: m.projects_choose_workspace(), onclick: () => goto('/settings/workspaces') }}
				/>
			{:else}
				<ProjectComposer bind:value={description} domain={data.list?.base_domain ?? ''} onbuild={build} />
				{#if retrying}
					<ul class="flex flex-wrap gap-4">
						{#each { length: 4 }, index (index)}<li><ProjectCardLoading /></li>{/each}
					</ul>
				{:else if !data.list}
					<LoadError reason={m.projects_load_failed_reason({ status: data.failure?.status ?? 503, time: failedAt })} onretry={retry} />
				{:else if cards.length}
					<ProjectToolbar {counts} bind:filter bind:query bind:sort />
					{#if shown.length}
						<ul class="flex flex-wrap gap-4">
							{#each shown as card (card.id)}
								<li data-project={card.id}>
									<ProjectCard
										name={card.name}
										description={card.description}
										tone={card.tone}
										status={card.status}
										statusLabel={card.statusLabel}
										meta={card.meta}
										detail={card.detail}
										url={card.url}
										footer={card.footer}
										href={card.href}
										questions={card.questions}
										onaction={action(card)}
										renaming={renamingId === card.id}
										onrename={(name) => void rename(card, name)}
										oncancelrename={() => (renamingId = null)}
										selected={card.id === selectedId}
										onselect={() => select(card)}
										actions={cardActions(card)}
									/>
								</li>
							{/each}
						</ul>
					{:else if query.trim()}
						<ProjectEmpty
							icon={SearchX}
							title={m.projects_no_match_title({ query: query.trim() })}
							body={m.projects_no_match_body({ workspace: workspaceName })}
							action={{ label: m.projects_clear_search(), onclick: () => (query = '') }}
						/>
					{:else}
						<ProjectEmpty icon={FolderSearch} title={TAB_EMPTY[filter].title()} body={TAB_EMPTY[filter].body()} />
					{/if}
				{:else}
					<FirstProject {starters} />
				{/if}
			{/if}
		</div>
	</div>
	{@render detailPanel()}
</div>
{/if}

{#snippet phoneDetail(card: ProjectCardData)}
	{@const read = about[card.id]}
	{@const broke = card.status === 'failed' || card.status === 'down' || card.status === 'attention'}
	{@const run = action(card)}
	<PhoneProjectDetail
		name={card.name}
		tone={card.tone}
		status={detailLine(card, read, now)}
		dot={card.status === 'live' ? 'live' : card.status === 'failed' || card.status === 'down' ? 'failed' : card.status === 'building' ? 'working' : 'idle'}
		alert={broke
			? {
					title: card.statusLabel ?? (card.status === 'failed' ? alertTitle(card, read) : card.status === 'down' ? m.projects_status_down() : card.questions === 1 ? m.projects_status_question() : m.projects_status_questions({ count: card.questions ?? 0 })),
					detail: card.detail,
					...(run && {
						action: {
							label: card.status === 'failed' ? m.projects_action_fix() : card.status === 'down' ? m.projects_action_check() : m.projects_action_answer(),
							icon: card.status === 'failed' ? Wrench : card.status === 'down' ? Activity : MessageCircle,
							onclick: run,
						},
					}),
				}
			: undefined}
		workspaceHref={card.href}
		url={card.liveUrl}
		parts={panelParts(card, read)}
		onexport={workspace(card, 'export')}
		chats={projectChats(data.chats, card, now)}
		backLabel={m.nav_projects()}
		onback={closePanel}
		onmore={() => (phoneSheet = { kind: 'card', card })}
	>
		{#snippet ask()}
			<PhoneAskBar placeholder={m.projects_phone_ask()} bind:value={question} onsend={(text) => void sendAsk(card, text)} />
		{/snippet}
	</PhoneProjectDetail>
{/snippet}

{#snippet phoneSheets()}
{#if phoneSheet?.kind === 'card'}
	{@const card = phoneSheet.card}
	<!-- Card menu (M4p0u): its state for a subtitle, as the detail screen says it. -->
	<BottomSheet open title={card.name} subtitle={detailLine(card, about[card.id], now)} onclose={sheetClosed('card')}>
		<PhoneProjectMenu actions={cardActions(card)} live={card.status === 'live'} onpick={() => (phoneSheet = null)} />
	</BottomSheet>
{/if}
<BottomSheet open={phoneSheet?.kind === 'new'} title={m.projects_phone_new_title()} subtitle={m.projects_phone_new_body()} onclose={sheetClosed('new')}>
	<div class="px-4 pt-1 pb-2">
		<ProjectComposer phone bind:value={description} domain={data.list?.base_domain ?? ''} onbuild={(text) => ((phoneSheet = null), void build(text))} />
	</div>
</BottomSheet>
<BottomSheet open={phoneSheet?.kind === 'sort'} title={m.projects_phone_sort_title()} onclose={sheetClosed('sort')}>
	{#each PHONE_SORTS as option (option.value)}
		<SheetOption icon={SORT_ICON[option.value]} label={option.label()} chosen={option.value === sort} onclick={() => ((sort = option.value), (phoneSheet = null))} />
	{/each}
</BottomSheet>
<BottomSheet open={phoneSheet?.kind === 'workspace'} title={m.projects_phone_workspaces()} subtitle={m.projects_phone_workspaces_body()} onclose={sheetClosed('workspace')}>
	<PhoneWorkspaceList {workspaces} chosen={orgId} onchoose={(id) => ((phoneSheet = null), id !== orgId && void chooseWorkspace(id))} />
	<span class="pt-2"></span>
	<SheetRow icon={Plus} label={m.workspace_create()} onclick={() => ((phoneSheet = null), void goto('/settings/workspaces'))} />
	{#if workspaceHref}
		<SheetRow icon={Settings} label={m.projects_phone_workspace_settings()} more onclick={() => ((phoneSheet = null), void goto(workspaceHref!))} />
	{/if}
</BottomSheet>
{/snippet}

{#snippet detailPanel()}
{#if selected}
	{@const card = selected}
	{@const read = about[card.id]}
	<div class="fixed inset-y-0 right-0 z-30 flex w-full overflow-y-auto bg-bg shadow-float sm:w-95 xl:static xl:z-auto xl:shadow-none">
		<ProjectDetailPanel
			docked
			name={card.name}
			tone={card.tone}
			live={liveLine(card, read, now)}
			workspaceHref={card.href}
			url={card.liveUrl}
			parts={panelParts(card, read)}
			onexport={workspace(card, 'export')}
			chats={projectChats(data.chats, card, now)}
			onask={startAsking}
			ask={asking ? asker : undefined}
			onclose={closePanel}
		>
			{#snippet preview()}
				<div class="flex h-full items-center justify-center" style="background: var(--crew-{card.tone}-bg)">
					<span aria-hidden="true" class="font-app text-display font-semibold" style="color: var(--crew-{card.tone}-ink)">{card.name.charAt(0).toUpperCase()}</span>
				</div>
			{/snippet}
		</ProjectDetailPanel>
	</div>
{/if}
{/snippet}

{#snippet asker()}
	{#if selected}
		{@const card = selected}
		<div data-asking>
			<AskField project={card.name} bind:value={question} hint={m.projects_ask_hint_chat({ name: card.name })} onask={(text) => void sendAsk(card, text)} />
		</div>
	{/if}
{/snippet}

{#if phone.current}
	<!-- A phone's confirmations (Flow / Mobile Projects — Act from the card, Card menu). -->
	<ActionSheet
		bind:open={stopOpen}
		title={m.projects_stop_title({ name: stopCard?.name ?? '' })}
		body={m.projects_stop_body_plain()}
		actions={[
			...(stopCard ? [{ label: m.projects_stop_export_first(), onclick: () => stopCard && workspace(stopCard, 'export')() }] : []),
			{ label: m.projects_stop_building(), danger: true, onclick: () => stopCard && void stop(stopCard) },
		]}
		cancel={m.projects_keep_building()}
	/>
	<DeploySheet
		bind:open={deployOpen}
		name={deployCard?.name ?? ''}
		files={deployCard?.files ?? 0}
		url={data.list?.base_domain ? m.projects_deploy_new_url({ domain: data.list.base_domain }) : m.projects_deploy_new_url_plain()}
		data={deployData}
		ondeploy={() => deployCard && void deploy(deployCard)}
	/>
	<DeleteProjectSheet
		open={deleting !== null}
		name={deleting?.name ?? ''}
		{busy}
		onexport={deleting ? workspace(deleting, 'export') : undefined}
		ondelete={remove}
		onclose={() => (deleting = null)}
	/>
{:else}
<StopDialog bind:open={stopOpen} name={stopCard?.name ?? ''} onstop={() => stopCard && void stop(stopCard)} />

<DeployDialog
	bind:open={deployOpen}
	name={deployCard?.name ?? ''}
	files={deployCard?.files ?? 0}
	url={data.list?.base_domain ? m.projects_deploy_new_url({ domain: data.list.base_domain }) : m.projects_deploy_new_url_plain()}
	data={deployData}
	ondeploy={() => deployCard && void deploy(deployCard)}
/>
{/if}

{#if phone.current}
	<ProjectMoveSheet
		bind:open={moveOpen}
		bind:value={moveValue}
		name={moveCard?.name ?? ''}
		chats={moveCard ? data.chats.filter((chat) => chat.project === moveCard?.id).length : 0}
		workspaces={moveChoices}
		onmove={(target) => moveCard && void move(moveCard, target)}
	/>
{:else}
<ProjectMoveDialog
	bind:open={moveOpen}
	bind:value={moveValue}
	name={moveCard?.name ?? ''}
	chats={moveCard ? data.chats.filter((chat) => chat.project === moveCard?.id).length : 0}
	workspaces={moveChoices}
	onmove={(target) => moveCard && void move(moveCard, target)}
/>
{/if}

{#if !phone.current}
<ProjectDialog
	open={deleting !== null}
	title={m.projects_delete_title({ name: deleting?.name ?? '' })}
	description={m.projects_delete_body()}
	width={420}
	onclose={() => (deleting = null)}
>
	{#snippet actions()}
		<Button variant="secondary" label={m.action_cancel()} onclick={() => (deleting = null)} />
		<Button variant="danger" label={m.projects_delete()} loading={busy} onclick={remove} />
	{/snippet}
</ProjectDialog>
{/if}
