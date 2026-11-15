<script lang="ts">
	import { Building2, CalendarCheck, FileText, FolderSearch, SearchX, Store } from '@lucide/svelte';
	import { onMount, tick } from 'svelte';
	import { goto, invalidate } from '$app/navigation';
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import { pageAction } from '$lib/pageAction';
	import { filterCounts, projectCards, runningFor, visibleCards, type ProjectCardData } from '$lib/projects/list';
	import { databaseName, liveLine, moveTargets, panelParts, projectChats, took } from '$lib/projects/about';
	import type { DeployOverview } from '$lib/adapters/api';
	import type { MoveChoices, ProjectAbout } from '$lib/server/projects';
	import { toasts } from '$lib/ds/overlays/toasts.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import ProjectPageHeader from '$lib/ds/projects/ProjectPageHeader.svelte';
	import ProjectComposer from '$lib/ds/projects/ProjectComposer.svelte';
	import ProjectToolbar, { type ProjectFilter } from '$lib/ds/projects/ProjectToolbar.svelte';
	import type { ProjectSort } from '$lib/ds/projects/SortMenu.svelte';
	import ProjectCard from '$lib/ds/projects/ProjectCard.svelte';
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
</script>

<svelte:head><title>{m.nav_projects()} · {m.app_name()}</title></svelte:head>
<svelte:window {onkeydown} />

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
										actions={{
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
										}}
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
</div>

{#snippet asker()}
	{#if selected}
		{@const card = selected}
		<div data-asking>
			<AskField project={card.name} bind:value={question} hint={m.projects_ask_hint_chat({ name: card.name })} onask={(text) => void sendAsk(card, text)} />
		</div>
	{/if}
{/snippet}

<StopDialog bind:open={stopOpen} name={stopCard?.name ?? ''} onstop={() => stopCard && void stop(stopCard)} />

<DeployDialog
	bind:open={deployOpen}
	name={deployCard?.name ?? ''}
	files={deployCard?.files ?? 0}
	url={data.list?.base_domain ? m.projects_deploy_new_url({ domain: data.list.base_domain }) : m.projects_deploy_new_url_plain()}
	data={deployData}
	ondeploy={() => deployCard && void deploy(deployCard)}
/>

<ProjectMoveDialog
	bind:open={moveOpen}
	bind:value={moveValue}
	name={moveCard?.name ?? ''}
	chats={moveCard ? data.chats.filter((chat) => chat.project === moveCard?.id).length : 0}
	workspaces={moveChoices}
	onmove={(target) => moveCard && void move(moveCard, target)}
/>

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
