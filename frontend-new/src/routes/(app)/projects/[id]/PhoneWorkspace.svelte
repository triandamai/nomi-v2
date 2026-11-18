<script lang="ts">
	import { untrack, tick } from 'svelte';
	import { Activity, Code, Database, Eye, FileCode, FileText, Folder, Globe, History, KeyRound, Link, Plus, Rocket, Settings, SquareTerminal, Trash2, Undo2, Users, Download, GitBranch } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import { goto } from '$app/navigation';
	import ChatView from '$lib/chat/ChatView.svelte';
	import WorkspaceSwitch from '$lib/ds/mobile/WorkspaceSwitch.svelte';
	import NavBar from '$lib/ds/mobile/NavBar.svelte';
	import SheetRow from '$lib/ds/mobile/SheetRow.svelte';
	import PhoneButton from '$lib/ds/mobile/PhoneButton.svelte';
	import PhoneToast from '$lib/ds/mobile/PhoneToast.svelte';
	import ErrorCard from '$lib/ds/mobile/ErrorCard.svelte';
	import LiveCard from '$lib/ds/mobile/LiveCard.svelte';
	import DnsRecord from '$lib/ds/mobile/DnsRecord.svelte';
	import AskQueue from '$lib/ds/mobile/AskQueue.svelte';
	import ChangeReady from '$lib/ds/mobile/ChangeReady.svelte';
	import BottomSheet from '$lib/ds/overlays/BottomSheet.svelte';
	import ActionSheet from '$lib/ds/overlays/ActionSheet.svelte';
	import SheetOption from '$lib/ds/overlays/SheetOption.svelte';
	import PhoneToastRegion from '$lib/ds/overlays/PhoneToastRegion.svelte';
	import DeleteProjectSheet from '$lib/ds/projects/DeleteProjectSheet.svelte';
	import PhoneWorkspaceNav, { type PhonePill } from '$lib/ds/projects/PhoneWorkspaceNav.svelte';
	import PhoneAskBar from '$lib/ds/projects/PhoneAskBar.svelte';
	import PhoneList from '$lib/ds/projects/PhoneList.svelte';
	import PhoneListRow from '$lib/ds/projects/PhoneListRow.svelte';
	import PhoneTile from '$lib/ds/projects/PhoneTile.svelte';
	import PhoneNote from '$lib/ds/projects/PhoneNote.svelte';
	import PresenceStrip from '$lib/ds/projects/PresenceStrip.svelte';
	import PhoneFollowBar from '$lib/ds/projects/PhoneFollowBar.svelte';
	import PhoneCodeLines from '$lib/ds/projects/PhoneCodeLines.svelte';
	import PhoneTerminal from '$lib/ds/projects/PhoneTerminal.svelte';
	import PhoneBuildShareSheet from '$lib/ds/projects/PhoneBuildShareSheet.svelte';
	import SubdomainField from '$lib/ds/projects/SubdomainField.svelte';
	import PeerAvatar from '$lib/ds/build/PeerAvatar.svelte';
	import Status from '$lib/ds/status/Status.svelte';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import Checkbox from '$lib/ds/inputs/Checkbox.svelte';
	import Segmented from '$lib/ds/inputs/Segmented.svelte';
	import Icon from '$lib/ds/foundations/Icon.svelte';
	import { plainLines } from '$lib/ds/chat/code';
	import { cellText } from '$lib/projects/database';
	import { domainView } from '$lib/projects/domains';
	import { terminalLines } from '$lib/projects/terminal';
	import { initials as initialsOf, whereLabel } from '$lib/projects/peers';
	import { requestMeta } from '$lib/projects/asks';
	import { ago } from '$lib/adapters/time';
	import SandboxWait from '$lib/projects/SandboxWait.svelte';
	import { failedPublish, phoneTree, publishPill, settledLive, type PhoneSheet } from './phone';
	import type { ToolSheet, Workspace } from './workspace.svelte';
	import type { PageData } from './$types';

	// A project's workspace on a phone (Flow / Mobile Build: Sign in → live URL zM0xh, Workspace tools
	// tJh5G, Landing page yY8qf, Roll back & hand-over Wv14s, Code, terminal & phone preview r3IvZk,
	// Invite & build together L8dgcS; contract docs/ux/work/phone-build/40-ux-contract.md). The same
	// `Workspace` as the desktop: the nav with the publish pill, Chat | Preview, who else is here, the
	// chat (the project's own, as the phone draws it) or the preview with the Ask bar; every tool a
	// bottom sheet in the desktop's order, the confirms that can be undone action sheets, and Code,
	// Terminal and a failed publish's log pushed screens with their own back. Only one sheet at a
	// time: a sheet opened from Settings replaces it, and closing it goes back to Settings.
	type Props = { ws: Workspace; data: PageData };
	let { ws, data }: Props = $props();

	let sheet = $state<PhoneSheet | null>(null);
	/** The pushed screens: Code's tree, a file, Terminal, a failed publish's log. */
	let screen = $state<'code' | 'file' | 'terminal' | 'log' | null>(null);
	const FROM_SETTINGS: PhoneSheet[] = ['people', 'environment', 'transfer', 'delete', 'visibility', 'access'];
	/** Opens a sheet in place of the one showing; the desktop's tools also tell the workspace (Monitor and Database load then). */
	function open(next: PhoneSheet) {
		sheet = next;
		const tool = next === 'deployments' || next === 'domain' || next === 'monitor' || next === 'database' || next === 'settings' || next === 'people' || next === 'environment';
		ws.sheet = tool ? (next as ToolSheet) : null;
		if (next === 'database') {
			// The live database once published, else the one Koda builds against (the desktop's rule).
			ws.dbSource = ws.liveUrl ? 'production' : 'development';
			ws.dbTable = null;
		}
	}
	/** A sheet closing (its X, Escape, the scrim) clears only itself; one opened from Settings goes back to it. */
	const closed = (kind: PhoneSheet) => () => {
		if (sheet !== kind) return;
		if (FROM_SETTINGS.includes(kind)) return open('settings');
		sheet = null;
		ws.sheet = null;
	};
	const shut = () => {
		sheet = null;
		ws.sheet = null;
	};

	const tab = $derived(ws.workspaceView === 'chat' ? 'chat' : 'preview');
	async function toChat(focus = false) {
		ws.workspaceView = 'chat';
		screen = null;
		if (!focus) return;
		await tick();
		document.querySelector<HTMLTextAreaElement>('[data-phone-workspace] textarea')?.focus();
	}

	// ── Publishing ────────────────────────────────────────────────────────────────────────────
	const pill: PhonePill = $derived(
		publishPill(ws.button, ws.role, Boolean(ws.mine), {
			onpublish: () => open('publish'),
			onask: () => void ws.askPublish(),
			onopen: () => open('mine'),
			asking: ws.asking,
		}),
	);
	const kind = $derived(ws.site ? m.pb_kind_site() : m.pb_kind_app());
	const statusLine = $derived(ws.liveUrl && ws.liveNumber ? m.pb_live_deploy({ number: ws.liveNumber }) : ws.status.label);
	const failure = $derived(ws.deploy ? failedPublish(ws.deploy, ws.time.timeZone) : null);
	/** The publish that went live while the workspace was open (Sign in → live URL, step 7). */
	let wentLive = $state<string | null>(null);
	let before = untrack(() => ws.deploy);
	$effect(() => {
		const now = ws.deploy;
		const live = settledLive(before, now);
		if (live) wentLive = live;
		before = now;
	});
	const liveCard = $derived(ws.deploy?.publishes.find((one) => one.id === wentLive && one.is_live) ?? null);
	const changes = $derived(ws.button.state === 'ready' ? ws.button.count : 0);

	// The changes sheet's answers: rolling back asks first; the toast after it offers the way back.
	let rollingTo = $state<{ id: string; number: number } | null>(null);

	// ── Domain ────────────────────────────────────────────────────────────────────────────────
	const ownDomain = $derived(ws.ownDomains.find((one) => one.primary) ?? ws.ownDomains[0] ?? null);

	// ── Code ──────────────────────────────────────────────────────────────────────────────────
	let openFolders = $state<string[]>([]);
	const tree = $derived(phoneTree(ws.tree, openFolders));
	function openPath(path: string) {
		ws.openFile(path);
		ws.phoneFile = true;
		screen = 'file';
	}
	$effect(() => {
		if (screen !== 'file') ws.phoneFile = false;
	});

	// ── Following: your view goes where they go (a teammate in Code opens the file here too) ──
	$effect(() => {
		const where = ws.followedView;
		if (!where) return;
		untrack(() => {
			if (where === 'code' && ws.followedPath) {
				ws.phoneFile = true;
				screen = 'file';
			} else if (where === 'terminal' && ws.sandboxOn) screen = 'terminal';
			else {
				screen = null;
				ws.workspaceView = 'pane';
			}
		});
	});
	/** Moving on your own stops following: the screen you're on is where you are. */
	$effect(() => {
		ws.view = screen === 'code' || screen === 'file' ? 'code' : screen === 'terminal' ? 'terminal' : 'preview';
	});

	// ── `?open=` from Projects (a card's sheet) or search: the same sheet or screen, here ──────
	$effect(() => {
		const asked = ws.opened;
		if (!asked) return;
		ws.opened = null;
		untrack(() => {
			if (asked.open === 'settings') open('settings');
			else if (asked.open === 'export') void openExport();
			else if (asked.open === 'share' && ws.liveUrl) open('share');
			else if (asked.open === 'takedown' && ws.liveUrl) open('takedown');
			else if (asked.open === 'code') {
				if (asked.file && ws.project.files.some((one) => one.path === asked.file)) openPath(asked.file);
				else screen = 'code';
			}
		});
	});

	// ── Preview ───────────────────────────────────────────────────────────────────────────────
	let device = $state<'phone' | 'desktop'>('phone');
	let frameWidth = $state(390);
	let frameHeight = $state(600);
	const scale = $derived(Math.min(1, frameWidth / 1440));

	// ── Tools ─────────────────────────────────────────────────────────────────────────────────
	async function openExport() {
		open('export');
		await ws.openExport();
		ws.exporting = false;
	}
	const kilobytes = (bytes: number) => (bytes < 1024 * 1024 ? `${Math.max(1, Math.round(bytes / 1024))} KB` : `${(bytes / (1024 * 1024)).toFixed(1)} MB`);
	const peopleCount = $derived(ws.people?.people.filter((one) => one.project_role || one.is_owner).length ?? 0);
	const rowsShown = $derived(ws.database && !('error' in ws.database) && ws.database.rows ? ws.database.rows : null);
	let addingSecret = $state(false);
	let removingSecret = $state<string | null>(null);
	let removingPerson = $state<{ id: string; name: string } | null>(null);
	let roleFor = $state<{ id: string; name: string; role: string } | null>(null);
	const days = (until: string | null) => (until ? Math.max(1, Math.round((Date.parse(until) - Date.now()) / 86_400_000)) : 7);
	async function invite() {
		const email = ws.inviteEmail.trim();
		if (await ws.peopleAct('invite', { email, role: ws.inviteRole }, m.people_invited({ email }))) open('people');
	}

	// ── Who's here ────────────────────────────────────────────────────────────────────────────
	const here = $derived(ws.peers.map((person) => ({ name: person.name, initials: person.initials, peer: person.peer, where: whereLabel(person.view, person.path, ws.where) })));
	const myWhere = $derived(whereLabel(ws.view, ws.view === 'code' ? ws.codePath : null, ws.where));
	function follow(index: number) {
		ws.following = ws.peers[index]?.user_id ?? null;
		shut();
	}
	const queue = $derived(
		ws.line.map((ask) => ({
			text: ask.text,
			who: { name: ask.who.name, initials: ask.who.initials, peer: ask.who.peer },
			state: ask.state === 'running' ? ('running' as const) : ask.state === 'next' ? ('next' as const) : ('queued' as const),
			meta: ask.meta,
		})),
	);
</script>

<svelte:head><title>{ws.project.name} · {m.app_name()}</title></svelte:head>

<PhoneToastRegion />

<div data-phone-workspace class="flex h-dvh flex-col bg-bg text-text">
	{#if ws.followed}
		<PhoneFollowBar name={ws.followed.name} peer={ws.followed.peer} where={whereLabel(ws.followed.view, ws.followed.path, ws.where)} onstop={() => (ws.following = null)} />
	{/if}

	{#if screen === 'code'}
		<NavBar title={m.workspace_code()} who={{ agent: 'koda', name: m.pb_read_only() }} onback={() => (screen = null)} />
		<div class="min-h-0 flex-1 overflow-y-auto px-4 pt-4 pb-5">
			{#if ws.project.files.length}
				<div class="flex flex-col gap-3">
					<p class="font-mono text-label tracking-label text-text-3 uppercase">{m.pb_all_files()}</p>
					<PhoneList label={m.workspace_code()}>
						{#each tree as entry (entry.path)}
							<PhoneListRow
								icon={entry.kind === 'folder' ? Folder : entry.kind === 'text' ? FileText : FileCode}
								title={entry.label}
								meta={entry.meta}
								mono
								current={entry.kind === 'folder' ? entry.open : undefined}
								onclick={() => {
									if (entry.kind !== 'folder') return openPath(entry.path);
									openFolders = entry.open ? openFolders.filter((one) => one !== entry.path) : [...openFolders, entry.path];
								}}
							/>
						{/each}
					</PhoneList>
					<PhoneNote text={m.pb_code_note()} />
				</div>
			{:else}
				<EmptyState icon={Code} title={m.workspace_no_files_title()} body={m.workspace_no_files_body()} />
			{/if}
		</div>
	{:else if screen === 'file' && ws.codePath}
		<NavBar title={ws.codePath.split('/').pop() ?? ws.codePath} who={{ agent: 'koda', name: ws.codeBinary ? m.code_binary() : ws.codeText === null ? m.code_reading() : m.pb_read_only() }} onback={() => (screen = 'code')} />
		<div class="min-h-0 flex-1 overflow-y-auto px-4 pt-4 pb-5">
			{#if ws.codeText !== null}
				<PhoneCodeLines label={ws.codePath} lines={ws.codeLines ?? plainLines(ws.codeText)} onask={(from, to) => (ws.askAbout(from, to), void toChat())} />
			{/if}
		</div>
	{:else if screen === 'terminal'}
		{@const lines = terminalLines(ws.runtime?.log ?? '')}
		<NavBar title={m.workspace_terminal()} who={{ agent: 'koda', name: m.sandbox_terminal_run() }} onback={() => (screen = null)} />
		<PhoneTerminal
			label={m.sandbox_terminal_run()}
			lines={lines.length ? lines : [{ kind: 'out', text: m.sandbox_terminal_empty() }]}
			failure={ws.sandboxPhase === 'failed' ? { title: m.sandbox_failed_title(), detail: ws.runtime?.error || m.sandbox_failed_body(), fix: m.projects_action_fix(), onfix: () => (ws.askToFix(), void toChat()) } : undefined}
		/>
	{:else if screen === 'log' && failure}
		<NavBar title={m.pb_log_title()} who={{ agent: 'koda', name: m.projects_deploy({ number: failure.number }) }} onback={() => (screen = null)} />
		<PhoneTerminal label={m.pb_log_title()} lines={failure.tail.length ? failure.tail.map((text) => ({ kind: 'out' as const, text })) : [{ kind: 'out', text: m.sandbox_terminal_empty() }]} />
	{:else}
		<PhoneWorkspaceNav project={ws.project.name} status={statusLine} live={Boolean(ws.liveUrl)} {pill} onback={() => void goto('/projects')} onmenu={() => open('tools')} />
		<WorkspaceSwitch value={tab} previewCount={changes || undefined} onchange={(next) => (ws.workspaceView = next === 'chat' ? 'chat' : 'pane')} />
		{#if here.length}
			<PresenceStrip people={here} onopen={() => open('presence')} onfollow={ws.following ? undefined : () => follow(0)} />
		{/if}
		<!-- The chat stays put under the preview (its live link, draft and place in the thread), so
		     switching tabs never reconnects it; only the tab on show is visible. -->
		<div class={['min-h-0 flex-1', tab !== 'chat' && 'hidden']}>
			<ChatView {data} pane quote={ws.quote} onunquote={() => (ws.quote = null)} onactivity={() => void ws.loadQueue()} after={together} />
		</div>
		{#if tab === 'preview'}
			<div class="relative min-h-0 flex-1">
				{#if ws.frame}
					<div class="flex h-full flex-col">
						<div class="flex justify-center px-4 pt-1 pb-2">
							<Segmented label={m.workspace_device()} value={device} onchange={(value) => (device = value as 'phone' | 'desktop')} options={[{ value: 'phone', label: m.workspace_phone() }, { value: 'desktop', label: m.workspace_desktop() }]} />
						</div>
						<div bind:clientWidth={frameWidth} bind:clientHeight={frameHeight} class="relative min-h-0 flex-1 overflow-hidden">
							{#key ws.reloads}
								{#if device === 'phone'}
									<iframe src={ws.frame} title={ws.project.name} class="h-full w-full bg-surface"></iframe>
								{:else}
									<iframe src={ws.frame} title={ws.project.name} class="origin-top-left bg-surface" style="width: 1440px; height: {frameHeight / scale}px; transform: scale({scale})"></iframe>
								{/if}
							{/key}
						</div>
						{#if device === 'desktop'}<p class="px-4 py-2 text-center font-mono text-label text-text-3">{m.pb_desktop_note()}</p>{/if}
					</div>
				{:else}
					<div class="h-full overflow-y-auto px-4 pt-6 pb-24">
						{#if ws.sandboxOn}
							<div aria-live="polite">
								<SandboxWait phase={ws.sandboxPhase} log={ws.runtime?.log ?? ''} error={ws.runtime?.error ?? null} onretry={() => void ws.runtime?.retry()} />
							</div>
						{:else}
							<EmptyState icon={Eye} title={m.workspace_nothing_title()} body={ws.project.status === 'building' ? m.workspace_nothing_building() : m.workspace_nothing_body()} />
						{/if}
					</div>
				{/if}
				<div class="absolute inset-x-4 bottom-4 pb-safe">
					<PhoneAskBar onask={() => void toChat(true)} />
				</div>
			</div>
		{/if}
	{/if}
</div>

{#snippet together()}
	<div class="flex flex-col gap-4">
		{#if liveCard && ws.liveUrl}
			<LiveCard
				meta={m.pb_live_meta({ number: liveCard.version, time: new Intl.DateTimeFormat('en-GB', { timeZone: ws.time.timeZone, hour: '2-digit', minute: '2-digit', hour12: false }).format(new Date(liveCard.finished_at ?? liveCard.created_at)) })}
				url={ws.liveUrl}
				summary={m.pb_live_summary()}
				onshare={() => open('share')}
			/>
		{/if}
		{#if failure}
			<div class="flex flex-col gap-3">
				<p class="text-phone leading-relaxed text-text">{failure.live ? m.pb_failed_said_live({ step: failure.step, number: failure.live }) : m.pb_failed_first({ step: failure.step })}</p>
				<ErrorCard
					title={m.pb_build_failed()}
					provider={failure.step}
					reason={failure.message}
					meta={m.pb_failed_meta({ number: failure.number, step: failure.step, elapsed: failure.elapsed })}
					primary={{
						label: m.pb_fix_it(),
						onclick: () => {
							ws.quote = { file: m.pb_log_title(), range: m.projects_deploy({ number: failure.number }), snippet: failure.message, text: m.sandbox_fix_text({ error: failure.message }) };
							void toChat(true);
						},
					}}
					secondary={{ label: m.pb_view_log(), onclick: () => (screen = 'log') }}
				/>
			</div>
		{/if}
		{#each ws.theirs as row (row.id)}
			{@const name = row.requested_by_name}
			<div class="flex flex-col items-stretch gap-1">
				<ChangeReady who={{ name, initials: initialsOf(name), peer: ws.peers.find((one) => one.name === name)?.peer ?? 1 }} meta={requestMeta(row, ws.deploy?.unpublished_changes ?? null, ws.time.timeZone)} onpublish={() => void ws.answerRequest(row, 'approve')} onreview={() => (ws.workspaceView = 'pane')} />
				<PhoneButton variant="secondary" label={m.publish_request_decline()} onclick={() => void ws.answerRequest(row, 'decline')} />
			</div>
		{/each}
		{#if queue.length > 1}
			<!-- AskQueue's button sits on the ask that's next: it's offered only when that one is a teammate's behind yours. -->
			<AskQueue asks={queue} onfirst={ws.movable >= 0 && ws.movable === queue.findIndex((ask) => ask.state === 'next') ? (index) => void ws.letFirst(index) : undefined} />
		{/if}
	</div>
{/snippet}

<!-- ··· (Workspace tools wbexi): the desktop's tools in its order, then under the hood, then Share and Export. -->
<BottomSheet open={sheet === 'tools'} title={ws.project.name} subtitle={ws.liveUrl && ws.liveNumber ? m.pb_tools_live({ kind, number: ws.liveNumber }) : m.pb_tools_sub({ kind, status: ws.status.label })} onclose={closed('tools')}>
	<div class="flex flex-col gap-0.5 px-4">
		{#if ws.deploy}<SheetRow icon={History} label={m.workspace_deployments()} meta={ws.rows.length ? String(ws.rows.length) : undefined} more onclick={() => open('deployments')} />{/if}
		{#if data.address}<SheetRow icon={Link} label={m.workspace_domain()} meta={ownDomain?.hostname ?? m.pb_free_address()} more onclick={() => open('domain')} />{/if}
		{#if ws.deploy}<SheetRow icon={Activity} label={m.workspace_monitor()} more onclick={() => open('monitor')} />{/if}
		{#if data.database}<SheetRow icon={Database} label={m.workspace_database()} more onclick={() => open('database')} />{/if}
		<SheetRow icon={Settings} label={m.project_settings()} more onclick={() => open('settings')} />
		{#if ws.site && !data.database}<p class="px-5 py-2 text-ui text-text-3">{m.pb_no_database()}</p>{/if}
		<p class="px-1 pt-1.5 pb-0.5 font-mono text-label tracking-label text-text-3 uppercase">{m.pb_under_hood()}</p>
		<SheetRow icon={Code} label={m.workspace_code()} meta={m.pb_read_only()} more onclick={() => ((screen = 'code'), shut())} />
		{#if ws.sandboxOn}<SheetRow icon={SquareTerminal} label={m.workspace_terminal()} meta={m.pb_what_koda_ran()} more onclick={() => ((screen = 'terminal'), shut())} />{/if}
		<div class="flex gap-2 pt-2">
			{#if ws.liveUrl}<PhoneButton variant="secondary" label={m.workspace_share()} onclick={() => open('share')} />{/if}
			<PhoneButton variant="secondary" icon={Download} label={m.workspace_export()} onclick={() => void openExport()} />
		</div>
	</div>
</BottomSheet>

<!-- Publish (pvAvh): the count, which deploy it'll be and which it replaces. -->
<BottomSheet
	open={sheet === 'publish'}
	title={ws.button.state === 'ready' ? (ws.button.count === 1 ? m.deploy_confirm_one() : m.deploy_confirm({ count: ws.button.count })) : m.deploy_confirm_retry()}
	subtitle={ws.liveNumber ? m.pb_publish_sub({ next: ws.nextNumber, live: ws.liveNumber }) : m.deploy_confirm_first({ number: ws.nextNumber })}
	onclose={closed('publish')}
>
	<div class="flex flex-col gap-3 px-4 pb-2">
		<p class="text-phone-sm text-text-2">{m.pb_publish_body()}</p>
		<PhoneButton label={m.deploy_publish_as({ number: ws.nextNumber })} icon={Rocket} disabled={ws.sending} onclick={() => void ws.startPublish().then(shut)} />
	</div>
</BottomSheet>

<!-- A member's request (Asked to publish): what it is, and taking it back. -->
<BottomSheet open={sheet === 'mine' && Boolean(ws.mine)} title={m.publish_request_mine_title()} subtitle={m.publish_request_mine_body()} onclose={closed('mine')}>
	<div class="flex flex-col gap-2 px-4 pb-2">
		{#if ws.mine}<p class="font-mono text-label text-text-3">{requestMeta(ws.mine, ws.deploy?.unpublished_changes ?? null, ws.time.timeZone)}</p>{/if}
		<PhoneButton variant="secondary" label={m.publish_request_keep()} onclick={shut} />
		<PhoneButton variant="danger" label={m.publish_request_cancel()} disabled={Boolean(ws.answering)} onclick={() => ws.mine && void ws.answerRequest(ws.mine, 'cancel').then(shut)} />
	</div>
</BottomSheet>

<!-- Deployments (pefkG): every publish, live or failed, with Roll back on the ones that can be. -->
<BottomSheet open={sheet === 'deployments'} title={m.workspace_deployments()} subtitle={ws.rows.length === 1 ? m.deploy_count_one({ name: ws.project.name }) : m.deploy_count({ name: ws.project.name, count: ws.rows.length })} onclose={closed('deployments')}>
	<div class="flex flex-col gap-2.5 px-4 pb-2">
		{#if !ws.deploy?.available}
			<EmptyState icon={Rocket} title={m.deploy_unavailable_title()} body={m.deploy_unavailable_body()} />
		{:else if !ws.rows.length}
			<EmptyState icon={Rocket} title={m.deploy_none_title()} body={m.deploy_none_body()} />
		{:else}
			<PhoneList label={m.workspace_deployments()}>
				{#each ws.rows as row (row.id)}
					<PhoneListRow lead={row.number} title={row.title} meta={row.error ? `${row.meta} · ${row.error}` : row.meta}>
						{#snippet end()}
							{#if row.state === 'live'}<Status kind="live" label={m.deploy_state_live()} />
							{:else if row.state === 'failed'}<Status kind="failed" label={m.deploy_state_failed()} />
							{:else if row.state === 'running'}<Status kind="working" label={m.deploy_state_running()} />
							{:else if row.state === 'rollback' && ws.role === 'admin'}
								<button type="button" class="inline-flex h-10 shrink-0 cursor-pointer items-center gap-1.5 rounded-md border border-border-strong px-2.75 text-phone-sm text-text outline-offset-1 pointer-coarse:min-h-11" onclick={() => (rollingTo = { id: row.id, number: row.version })}>
									<Icon icon={Undo2} size={14} />{m.pb_roll_back()}
								</button>
							{/if}
						{/snippet}
					</PhoneListRow>
				{/each}
			</PhoneList>
			<p class="text-ui text-text-3">{m.pb_rollback_note()}</p>
		{/if}
	</div>
</BottomSheet>

<ActionSheet
	open={rollingTo !== null}
	title={m.pb_rollback_title({ number: rollingTo?.number ?? 0 })}
	body={m.pb_rollback_body({ live: ws.liveNumber ?? 0 })}
	actions={[{ label: m.deploy_roll_back_to({ number: rollingTo?.number ?? 0 }), onclick: () => rollingTo && void ws.rollBack(rollingTo.id) }]}
	cancel={m.action_cancel()}
	onclose={() => (rollingTo = null)}
/>

<!-- Domain (py7VN, JpLGa): the free address, a domain of their own with its records, and (G9) changing the address. -->
<BottomSheet open={sheet === 'domain'} title={m.workspace_domain()} subtitle={ownDomain ? ownDomain.hostname : m.pb_domain_sub()} onclose={closed('domain')}>
	<div class="flex flex-col gap-2.5 px-4 pb-2">
		{#if !ws.address?.hostname}
			<EmptyState icon={Globe} title={m.domain_none_title()} body={m.domain_none_body()} />
		{:else}
			<PhoneList label={m.domain_addresses()}>
				<PhoneListRow icon={Globe} title={ws.address.hostname} mono>
					{#snippet end()}{#if ws.liveUrl}<Status kind="live" label={m.deploy_state_live()} />{/if}{/snippet}
				</PhoneListRow>
				{#if ws.address.pending_hostname}<PhoneListRow icon={Globe} title={ws.address.pending_hostname} meta={m.domain_next_publish()} mono />{/if}
			</PhoneList>
			{#each ws.ownDomains as own (own.hostname)}
				{@const shown = domainView(own, ws.time.timeZone)}
				<section aria-label={own.hostname} class="flex flex-col gap-2.5">
					<p class="flex h-12 items-center rounded-lg bg-surface-2 px-3.5 font-mono text-phone text-text">{own.hostname}</p>
					<p class="font-mono text-label tracking-label text-text-3 uppercase">{m.pb_registrar()}</p>
					{#each shown.records as record (`${record.type}:${record.name}`)}
						<DnsRecord type={record.type} name={record.name} value={record.value} state={record.found ? 'verified' : 'waiting'} />
					{/each}
					<p class="text-ui text-text-3">{shown.checked} · {shown.note}</p>
					{#if ws.address.can_edit && shown.canMakePrimary}
						<PhoneButton label={m.pb_make_primary({ domain: own.hostname })} disabled={ws.domainBusy !== null} onclick={() => void ws.domainAction('domainPrimary', own.hostname, m.domain_primary_done({ hostname: own.hostname }))} />
					{:else}
						<PhoneButton label={m.pb_check_records()} disabled={ws.domainBusy !== null} onclick={() => void ws.domainAction('domainCheck', own.hostname)} />
					{/if}
					{#if ws.address.can_edit}<PhoneButton variant="secondary" icon={Trash2} label={m.domain_remove()} onclick={() => (ws.removingDomain = own.hostname)} />{/if}
				</section>
			{/each}
			{#if ws.address.can_edit}
				{#if ws.renaming}
					<div class="flex flex-col gap-2.5">
						<SubdomainField
							bind:value={ws.name}
							suffix={ws.address.base_domain ?? ws.address.hostname.split('.').slice(1).join('.')}
							check={ws.check}
							labels={{ field: m.domain_field(), checking: m.domain_checking(), available: m.domain_available(), suggestions: m.domain_free_instead() }}
							oninput={ws.checkSoon}
							onpick={(picked) => {
								ws.name = picked;
								ws.checkSoon(picked);
							}}
						/>
						<PhoneButton label={m.domain_save()} disabled={ws.saving || !ws.name.trim() || ws.check.state === 'taken' || ws.check.state === 'checking'} onclick={ws.saveName} />
						<PhoneButton variant="secondary" label={m.action_cancel()} onclick={() => (ws.renaming = false)} />
						<p class="text-ui text-text-3">{m.domain_rename_note()}</p>
					</div>
				{:else}
					<PhoneButton variant="secondary" label={m.domain_change()} onclick={ws.startRenaming} />
				{/if}
			{/if}
		{/if}
		{#if ws.address?.can_edit && ws.liveUrl}
			<form
				class="flex flex-col gap-2 border-t border-border pt-3"
				onsubmit={async (event) => {
					event.preventDefault();
					if (await ws.domainAction('domainAdd', ws.newDomain.trim().toLowerCase())) ws.newDomain = '';
				}}
			>
				<TextField label={m.domain_add_field()} bind:value={ws.newDomain} placeholder="warungkopi.id" mono name="hostname" autocomplete="off" />
				<PhoneButton variant="secondary" icon={Plus} label={m.domain_add()} disabled={!ws.newDomain.trim() || ws.domainBusy !== null} onclick={async () => (await ws.domainAction('domainAdd', ws.newDomain.trim().toLowerCase())) && (ws.newDomain = '')} />
			</form>
		{/if}
		{#if ws.domainError}<p role="alert" class="text-ui text-danger">{ws.domainError}</p>{/if}
	</div>
</BottomSheet>

<ActionSheet
	open={ws.removingDomain !== null}
	title={m.domain_remove_title({ hostname: ws.removingDomain ?? '' })}
	body={m.domain_remove_body()}
	actions={[{ label: m.domain_remove(), danger: true, onclick: () => { const hostname = ws.removingDomain ?? ''; void ws.domainAction('domainRemove', hostname, m.domain_removed({ hostname })); } }]}
	cancel={m.action_cancel()}
	onclose={() => (ws.removingDomain = null)}
/>

<!-- Monitor (M4GVXb, a site's vhsAL): today, looked at again every 30 seconds while it's open. -->
<BottomSheet open={sheet === 'monitor'} title={m.workspace_monitor()} subtitle={m.monitor_today()} onclose={closed('monitor')}>
	<div class="flex flex-col gap-2.5 px-4 pb-2">
		{#if !ws.liveUrl}
			<EmptyState icon={Activity} title={m.monitor_unpublished_title()} body={m.monitor_unpublished_body()} />
		{:else if ws.monitorFailed}
			<EmptyState icon={Activity} title={m.monitor_failed_title()} body={m.monitor_failed_body()} />
		{:else if !ws.monitoring}
			<p role="status" class="text-phone-sm text-text-3">{m.pb_monitor_reading()}</p>
		{:else}
			<div class="flex flex-col gap-2">
				{#if ws.site}
					<div class="flex gap-2">
						<PhoneTile label={m.monitor_visits()} value={ws.stats?.visitors} note={m.monitor_today()} notMeasured={m.monitor_not_measured()} />
						<PhoneTile label={m.monitor_form_sends()} value={ws.siteNumbers?.formSends} note={ws.siteNumbers?.formShare ? m.monitor_form_share({ share: ws.siteNumbers.formShare }) : undefined} notMeasured={m.monitor_not_measured()} />
					</div>
				{:else}
					<div class="flex gap-2">
						<PhoneTile label={m.monitor_requests()} value={ws.stats?.requests} note={m.monitor_today()} notMeasured={m.monitor_not_measured()} />
						<PhoneTile label={m.monitor_errors()} value={ws.stats?.errors} note={ws.stats?.errorShare ? m.monitor_error_share({ share: ws.stats.errorShare }) : undefined} notMeasured={m.monitor_not_measured()} />
					</div>
					<div class="flex gap-2">
						<PhoneTile label="P95" value={ws.stats?.p95} note={m.monitor_p95_note()} notMeasured={m.monitor_not_measured()} />
						<PhoneTile label={m.monitor_visitors()} value={ws.stats?.visitors} note={m.monitor_today()} notMeasured={m.monitor_not_measured()} />
					</div>
				{/if}
			</div>
			{#if ws.site}
				<p class="font-mono text-label tracking-label text-text-3 uppercase">{m.monitor_sources()}</p>
				{#if ws.sources?.length}
					<PhoneList label={m.monitor_sources()}>
						{#each ws.sources as source (source.name)}<PhoneListRow icon={Globe} title={source.name} meta={source.value} />{/each}
					</PhoneList>
				{:else}
					<p class="text-ui text-text-3">{m.monitor_sources_none()}</p>
				{/if}
			{/if}
			<p class="font-mono text-label tracking-label text-text-3 uppercase">{m.monitor_log()}</p>
			{#if ws.monitoring.logs?.length}
				<div role="log" aria-label={m.monitor_log()} class="flex flex-col gap-1.5 rounded-xl px-3 py-2 outline -outline-offset-1 outline-border">
					{#each ws.monitoring.logs.slice(-12) as line, index (index)}<p class="font-mono text-label break-all text-text-2">{line}</p>{/each}
				</div>
			{:else}
				<p class="text-ui text-text-3">{ws.monitoring.note ?? m.monitor_log_empty()}</p>
			{/if}
		{/if}
	</div>
</BottomSheet>

<!-- Database (jyEKQ): read only on a phone (D4), the live tables once published. -->
<BottomSheet open={sheet === 'database'} title={m.workspace_database()} subtitle={ws.database && !('error' in ws.database) ? m.pb_tables({ count: ws.database.tables.length }) : undefined} onclose={closed('database')}>
	<div class="flex flex-col gap-2.5 px-4 pb-2">
		{#if !ws.canRead}
			<EmptyState icon={Database} title={m.deploy_unavailable_title()} body={m.database_unavailable_body()} />
		{:else if !ws.database && ws.dbLoading}
			<p role="status" class="text-phone-sm text-text-3">{m.database_loading()}</p>
		{:else if ws.database && 'error' in ws.database}
			<EmptyState icon={Database} title={ws.database.status === 404 ? m.database_none_title() : m.database_failed_title()} body={ws.database.error || m.database_failed_body()} />
		{:else if ws.database && !ws.database.tables.length}
			<EmptyState icon={Database} title={m.database_empty_title()} body={m.database_empty_body()} />
		{:else if ws.database}
			{@const shownTable = ws.database.table}
			<div role="radiogroup" aria-label={m.workspace_database()} class="flex flex-wrap gap-2">
				{#each ws.database.tables as table (`${table.schema}.${table.name}`)}
					{@const chosen = shownTable?.name === table.name && shownTable?.schema === table.schema}
					<button type="button" role="radio" aria-checked={chosen} class={['inline-flex h-10 items-center rounded-full px-2.75 font-mono text-ui outline-offset-1', chosen ? 'border border-text bg-text text-bg' : 'cursor-pointer border border-border-strong text-text-2']} onclick={() => (ws.dbTable = `${table.schema}.${table.name}`)}>{table.name}</button>
				{/each}
			</div>
			{#if rowsShown}
				<PhoneList label={shownTable?.name}>
					{#each rowsShown.rows.slice(0, 20) as row, index (index)}
						<PhoneListRow title={cellText(row[0])} meta={row.slice(1, 4).map(cellText).join(' · ')} />
					{/each}
				</PhoneList>
			{/if}
			<PhoneNote boxed text={m.pb_db_note()} />
		{/if}
	</div>
</BottomSheet>

<!-- Settings (tweH7): People, Environment, Visibility; then ownership. No Name row (D3). -->
<BottomSheet open={sheet === 'settings'} title={m.project_settings()} subtitle={ws.project.name} onclose={closed('settings')}>
	<div class="flex flex-col gap-1 px-4">
		{#if ws.people}<SheetRow icon={Users} label={m.people_title()} meta={m.pb_people_count({ count: peopleCount })} more onclick={() => open('people')} />{/if}
		{#if ws.secrets}<SheetRow icon={KeyRound} label={m.env_title()} meta={m.pb_variables({ count: ws.secrets.secrets.length })} more onclick={() => open('environment')} />{/if}
		{#if ws.liveUrl && data.showcase}<SheetRow icon={Eye} label={m.pb_visibility_row()} meta={ws.showcased ? m.pb_public() : m.bshare_unlisted()} more onclick={() => open('visibility')} />{/if}
		<p class="px-1 pt-1.5 pb-0.5 font-mono text-label tracking-label text-text-3 uppercase">{m.settings_ownership()}</p>
		<SheetRow icon={Users} label={m.settings_transfer()} more onclick={() => (ws.openDialog('transfer'), (ws.dialog = null), open('transfer'))} />
		{#if ws.liveUrl}<SheetRow icon={Globe} label={m.settings_takedown()} onclick={() => open('takedown')} />{/if}
		<SheetRow icon={Trash2} label={m.projects_menu_delete()} danger onclick={() => open('delete')} />
	</div>
</BottomSheet>

<BottomSheet open={sheet === 'visibility'} title={m.pb_visibility_row()} subtitle={ws.project.name} onclose={closed('visibility')}>
	<SheetOption icon={Globe} label={m.pb_public()} detail={m.pb_public_detail()} chosen={ws.showcased} onclick={() => data.showcase?.can_edit && void ws.setShown(true)} />
	<SheetOption icon={Link} label={m.bshare_unlisted()} detail={m.pb_unlisted_detail()} chosen={!ws.showcased} onclick={() => data.showcase?.can_edit && void ws.setShown(false)} />
</BottomSheet>

<!-- People (OgP6t, E8mu9): who's on it and their place; admins invite, change a role, take someone off. -->
<BottomSheet open={sheet === 'people' && Boolean(ws.people)} title={m.people_title()} subtitle={m.pb_people_sub({ name: ws.project.name, count: peopleCount })} onclose={closed('people')}>
	{#if ws.people}
		{@const people = ws.people}
		<div class="flex flex-col gap-2.5 px-4 pb-2">
			{#if ws.invited}
				{@const sent = ws.invited}
				{#if sent.path}
					<PhoneToast kind="success" title={m.pb_invite_sent()} detail={m.pb_invite_sent_to({ email: sent.email, days: days(sent.expiresAt) })} action={{ label: m.share_copy(), onclick: () => void navigator.clipboard?.writeText(`${location.origin}${sent.path}`) }} />
				{:else}
					<PhoneToast kind="success" title={m.people_invited({ email: sent.email })} />
				{/if}
			{/if}
			{#if ws.projectAdmin}
				<form
					class="flex items-end gap-2"
					onsubmit={(event) => {
						event.preventDefault();
						open('invite');
					}}
				>
					<div class="min-w-0 flex-1"><TextField label={m.people_email()} type="email" bind:value={ws.inviteEmail} error={ws.peopleError} autocomplete="email" /></div>
					<button type="submit" class="inline-flex h-11 shrink-0 cursor-pointer items-center rounded-lg bg-accent px-4 text-phone font-medium text-accent-ink outline-offset-2">{m.pb_invite()}</button>
				</form>
			{/if}
			<PhoneList label={m.people_title()}>
				{#each people.people as person (person.user_id)}
					{@const label = person.is_owner ? m.people_owner() : person.project_role ? (ws.roles.find((one) => one.value === person.project_role)?.label ?? '') : people.access === 'org' ? m.people_via_workspace() : m.people_not_added()}
					<PhoneListRow title={person.name} meta={[person.user_id === people.me ? m.workspace_you() : null, person.is_owner ? m.people_started_it() : null].filter(Boolean).join(' · ') || undefined}>
						{#snippet start()}<span class="inline-flex size-8 shrink-0 items-center justify-center rounded-full bg-surface-2 font-mono text-label text-text-2">{initialsOf(person.name)}</span>{/snippet}
						{#snippet end()}
							{#if ws.projectAdmin && !person.is_owner && person.user_id !== people.me}
								<button type="button" aria-haspopup="dialog" aria-label={m.people_role_of({ name: person.name })} class="inline-flex h-11 shrink-0 cursor-pointer items-center font-mono text-caption text-text-2 underline underline-offset-2 outline-offset-1" onclick={() => (roleFor = { id: person.user_id, name: person.name, role: person.project_role ?? '' })}>{label}</button>
							{:else}
								<span class="font-mono text-caption text-text-2">{label}</span>
							{/if}
						{/snippet}
					</PhoneListRow>
				{/each}
			</PhoneList>
			{#if ws.projectAdmin}
				<SheetRow icon={Eye} label={m.people_access()} meta={people.access === 'org' ? m.people_access_org({ workspace: ws.project.org_name }) : m.people_access_members()} more onclick={() => open('access')} />
			{/if}
			<p class="text-ui text-text-3">{m.people_note()}</p>
		</div>
	{/if}
</BottomSheet>

<BottomSheet open={roleFor !== null} title={m.people_role_of({ name: roleFor?.name ?? '' })} onclose={() => (roleFor = null)}>
	{#each ws.roles as role (role.value)}
		<SheetOption label={role.label} chosen={roleFor?.role === role.value} onclick={() => { const who = roleFor; roleFor = null; if (who) void ws.peopleAct('role', { user: who.id, role: role.value }, m.people_role_changed({ name: who.name })); }} />
	{/each}
	{#if roleFor?.role}
		<SheetOption icon={Trash2} label={m.people_remove({ name: roleFor.name })} onclick={() => { const who = roleFor; roleFor = null; if (who) removingPerson = { id: who.id, name: who.name }; }} />
	{/if}
</BottomSheet>

<ActionSheet
	open={removingPerson !== null}
	title={m.people_remove({ name: removingPerson?.name ?? '' })}
	actions={[{ label: m.people_remove({ name: removingPerson?.name ?? '' }), danger: true, onclick: () => { const who = removingPerson; if (who) void ws.peopleAct('remove', { user: who.id }, m.people_removed({ name: who.name })); } }]}
	cancel={m.action_cancel()}
	onclose={() => (removingPerson = null)}
/>

<BottomSheet open={sheet === 'access'} title={m.people_access()} subtitle={ws.project.name} onclose={closed('access')}>
	{#if ws.people}
		{@const access = ws.people.access}
		<SheetOption icon={Users} label={m.people_access_org({ workspace: ws.project.org_name })} chosen={access === 'org'} onclick={() => void ws.peopleAct('access', { access: 'org' }, m.people_access_opened()).then(() => open('people'))} />
		<SheetOption icon={KeyRound} label={m.people_access_members()} chosen={access === 'members'} onclick={() => void ws.peopleAct('access', { access: 'members' }, m.people_access_limited()).then(() => open('people'))} />
	{/if}
</BottomSheet>

<!-- Invite (OvEGT): the address and what they may do, in plain words. -->
<BottomSheet open={sheet === 'invite'} title={m.pb_invite_title({ name: ws.project.name })} subtitle={m.pb_invite_sub()} onclose={() => sheet === 'invite' && open('people')}>
	<form
		class="flex flex-col gap-2.5 px-4 pb-2"
		onsubmit={(event) => {
			event.preventDefault();
			void invite();
		}}
	>
		<TextField label={m.people_email()} type="email" bind:value={ws.inviteEmail} error={ws.peopleError} autocomplete="email" />
		<div role="radiogroup" aria-label={m.people_invite_as()} class="-mx-4 flex flex-col">
			<SheetOption label={m.role_build()} detail={m.pb_role_build_help()} chosen={ws.inviteRole === 'build'} onclick={() => (ws.inviteRole = 'build')} />
			<SheetOption label={m.role_prompt()} detail={m.pb_role_prompt_help()} chosen={ws.inviteRole === 'prompt'} onclick={() => (ws.inviteRole = 'prompt')} />
			<SheetOption label={m.role_view()} detail={m.pb_role_view_help()} chosen={ws.inviteRole === 'view'} onclick={() => (ws.inviteRole = 'view')} />
		</div>
		<PhoneButton label={m.projects_send_invite()} disabled={ws.busyPeople || !ws.inviteEmail.trim()} onclick={() => void invite()} />
	</form>
</BottomSheet>

<!-- Environment (mAp2k): names only, never values; a variable added applies on the next publish. -->
<BottomSheet open={sheet === 'environment' && Boolean(ws.secrets)} title={m.env_title()} subtitle={m.pb_env_sub({ count: ws.secrets?.secrets.length ?? 0 })} onclose={closed('environment')}>
	{#if ws.secrets}
		{@const secrets = ws.secrets}
		<div class="flex flex-col gap-2.5 px-4 pb-2">
			{#if secrets.secrets.length}
				<PhoneList label={m.env_title()}>
					{#each secrets.secrets as secret (secret.name)}
						<PhoneListRow title={secret.name} mono meta={[m.env_updated({ when: ago(secret.updated_at, new Date()) }), secret.updated_by_name].filter(Boolean).join(' · ')}>
							{#snippet end()}
								{#if secrets.can_edit}
									<button type="button" aria-label={m.env_remove({ name: secret.name })} class="inline-flex size-11 shrink-0 cursor-pointer items-center justify-center rounded-full text-text-2 outline-offset-1" onclick={() => (removingSecret = secret.name)}><Icon icon={Trash2} size={18} /></button>
								{/if}
							{/snippet}
						</PhoneListRow>
					{/each}
				</PhoneList>
			{:else}
				<p class="text-phone-sm text-text-3">{m.env_none()}</p>
			{/if}
			{#if secrets.can_edit}
				{#if addingSecret}
					<form
						class="flex flex-col gap-2"
						onsubmit={async (event) => {
							event.preventDefault();
							if (await ws.saveSecret()) addingSecret = false;
						}}
					>
						<TextField label={m.env_name()} bind:value={ws.envName} mono autocomplete="off" placeholder="ADMIN_PIN" error={ws.envError?.field === 'name' ? ws.envError.message : undefined} />
						<TextField label={m.env_value()} type="password" bind:value={ws.envValue} mono autocomplete="off" error={ws.envError && ws.envError.field !== 'name' ? ws.envError.message : undefined} />
						<PhoneButton label={m.env_add()} disabled={ws.savingEnv || !ws.envName.trim() || !ws.envValue} onclick={async () => (await ws.saveSecret()) && (addingSecret = false)} />
					</form>
				{:else}
					<PhoneButton variant="secondary" icon={Plus} label={m.env_add()} onclick={() => (addingSecret = true)} />
				{/if}
			{/if}
			<p class="text-ui text-text-3">{m.env_note()}</p>
		</div>
	{/if}
</BottomSheet>

<ActionSheet
	open={removingSecret !== null}
	title={m.env_remove({ name: removingSecret ?? '' })}
	actions={[{ label: m.env_remove({ name: removingSecret ?? '' }), danger: true, onclick: () => removingSecret && void ws.removeSecret(removingSecret) }]}
	cancel={m.action_cancel()}
	onclose={() => (removingSecret = null)}
/>

<!-- Transfer (jg2Am): to another workspace the person is in, the name typed to confirm. -->
<BottomSheet open={sheet === 'transfer'} title={m.transfer_title({ name: ws.project.name })} subtitle={m.transfer_body()} onclose={closed('transfer')}>
	<div class="flex flex-col gap-2.5 pb-2">
		{#if ws.targets.length}
			<div role="radiogroup" aria-label={m.transfer_to()} class="flex flex-col">
				{#each ws.targets as one (one.id)}
					<SheetOption label={one.name} detail={one.owner_name ? m.transfer_owner({ name: one.owner_name }) : undefined} chosen={ws.target === one.id} onclick={() => (ws.target = one.id)} />
				{/each}
			</div>
		{:else}
			<p class="px-4 text-phone-sm text-text-3">{m.transfer_none()}</p>
		{/if}
		<div class="flex flex-col gap-2.5 px-4">
			<TextField label={m.confirm_type_name({ name: ws.project.name })} bind:value={ws.confirmName} mono autocomplete="off" error={ws.actError} />
			<PhoneButton label={m.transfer_send()} disabled={ws.acting || !ws.target || ws.confirmName.trim() !== ws.project.name} onclick={() => void ws.act('transfer', { to: ws.target ?? '', name: ws.confirmName.trim() })} />
		</div>
	</div>
</BottomSheet>

<!-- Take down (l94PT): reversible, so an action sheet. -->
<ActionSheet
	open={sheet === 'takedown'}
	title={m.takedown_title({ name: ws.project.name })}
	body={m.pb_takedown_body()}
	actions={[{ label: m.settings_takedown(), danger: true, onclick: () => void ws.act('takedown', {}) }]}
	cancel={m.action_cancel()}
	onclose={closed('takedown')}
/>

<!-- Delete (Fbg8O): the name typed, the export offered first. -->
<DeleteProjectSheet
	open={sheet === 'delete'}
	name={ws.project.name}
	busy={ws.acting}
	onexport={() => void openExport()}
	ondelete={() => void ws.act('delete', { name: ws.project.name })}
	onclose={closed('delete')}
/>

<!-- Share (CFzPR, a site's eMzUB): once live. -->
{#if ws.liveUrl}
	<PhoneBuildShareSheet
		open={sheet === 'share'}
		project={ws.project.name}
		url={ws.liveUrl}
		card={ws.site ? { title: ws.project.name, ...(ws.project.description && { description: ws.project.description }) } : undefined}
		visibility={ws.site && data.showcase ? (ws.showcased ? 'public' : 'unlisted') : undefined}
		onvisibility={data.showcase?.can_edit ? (next) => void ws.setShown(next === 'public') : undefined}
		oninvite={ws.people && ws.projectAdmin ? () => open('invite') : undefined}
		onclose={closed('share')}
	/>
{/if}

<!-- Export (bR1XV): what it holds, then GitHub or a zip with equal weight. -->
<BottomSheet open={sheet === 'export'} title={m.pb_export_title({ name: ws.project.name })} subtitle={m.pb_export_sub()} onclose={closed('export')}>
	<div class="flex flex-col gap-0.5 px-4">
		{#snippet holds(icon: typeof GitBranch, label: string, meta: string)}
			<p class="flex h-13 items-center gap-3.5 px-5">
				<span class="inline-flex shrink-0 text-text-2"><Icon {icon} size={20} /></span>
				<span class="min-w-0 flex-1 truncate text-phone text-text">{label}</span>
				<span class="font-mono text-label text-text-3">{meta}</span>
			</p>
		{/snippet}
		{@render holds(GitBranch, m.pb_repo(), ws.summary ? (ws.summary.files === 1 ? m.export_holds_one({ size: kilobytes(ws.summary.bytes) }) : m.export_holds({ files: ws.summary.files, size: kilobytes(ws.summary.bytes) })) : m.export_reading())}
		{#if ws.summary && ws.summary.database !== 'none'}{@render holds(Database, m.pb_dump(), m.export_database({ kind: ws.summary.database }))}{/if}
		{#if ws.summary?.env.length}{@render holds(KeyRound, m.pb_env_names(), m.pb_names_only())}{/if}
		<div class="flex gap-2 pt-2">
			<PhoneButton variant="secondary" label={m.export_github()} onclick={() => ((ws.pushError = undefined), open('github'))} />
			<PhoneButton variant="secondary" icon={Download} label={m.export_zip()} onclick={() => (location.href = `/projects/${ws.project.id}/export`)} />
		</div>
	</div>
</BottomSheet>

<BottomSheet open={sheet === 'github'} title={m.github_title()} subtitle={m.github_body()} onclose={() => sheet === 'github' && open('export')}>
	<div class="flex flex-col gap-2.5 px-4 pb-2">
		<TextField label={m.github_repo()} bind:value={ws.repo} mono placeholder="owner/{ws.project.name}" autocomplete="off" error={ws.pushError} />
		<Checkbox label={m.github_create()} bind:checked={ws.createRepo} />
		<Checkbox label={m.github_private()} bind:checked={ws.privateRepo} />
		<p class="text-ui text-text-3">{m.github_note()} <a href="/settings/connections" class="text-text-2 underline underline-offset-2">{m.github_connections()}</a></p>
		<PhoneButton label={m.github_push()} disabled={ws.sendingPush || !ws.repo.trim()} onclick={() => void ws.push().then(() => !ws.pushError && shut())} />
	</div>
</BottomSheet>

<!-- Who's here (GnUU0): everyone else with where they are and Follow; you last. -->
<BottomSheet open={sheet === 'presence'} title={m.pb_presence_title()} subtitle={m.pb_presence_sub({ count: ws.peers.length + 1 })} onclose={closed('presence')}>
	<div class="flex flex-col gap-2 px-4 pb-2">
		{#each here as person, index (person.name)}
			<div class="flex items-center gap-3 rounded-xl bg-surface-2 px-3.5 py-3">
				<PeerAvatar initials={person.initials} peer={person.peer} />
				<span class="flex min-w-0 flex-1 flex-col gap-0.5">
					<span class="truncate text-phone text-text">{person.name}</span>
					<span class="truncate text-ui text-text-3">{m.pb_viewing({ where: person.where })}</span>
				</span>
				<button type="button" class={['inline-flex h-9 shrink-0 cursor-pointer items-center rounded-full px-3.5 text-phone-sm font-medium text-bg outline-offset-1 pointer-coarse:min-h-11', person.peer === 2 ? 'bg-peer-2' : 'bg-peer-1']} aria-label={m.pb_follow_name({ name: person.name })} onclick={() => follow(index)}>{m.presence_follow()}</button>
			</div>
		{/each}
		<div class="flex items-center gap-3 rounded-xl px-3.5 py-3">
			<PeerAvatar initials={ws.faces[0] ?? ws.initials} />
			<span class="flex min-w-0 flex-1 flex-col gap-0.5">
				<span class="truncate text-phone text-text">{ws.myName}</span>
				<span class="truncate text-ui text-text-3">{m.pb_you_where({ where: myWhere })}</span>
			</span>
		</div>
	</div>
</BottomSheet>
