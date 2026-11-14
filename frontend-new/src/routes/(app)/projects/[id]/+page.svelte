<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import { Globe, History, Link, MonitorX, Rocket } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import { pageAction } from '$lib/pageAction';
	import { clock as appClock } from '$lib/clock';
	import { deployRows, publishState, publishing } from '$lib/projects/deploy';
	import type { DeployOverview } from '$lib/adapters/api';
	import ChatView from '$lib/chat/ChatView.svelte';
	import { toasts } from '$lib/ds/overlays/toasts.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import ProjectWorkspace from '$lib/ds/projects/ProjectWorkspace.svelte';
	import ProjectBar from '$lib/ds/projects/ProjectBar.svelte';
	import ChatPaneHeader from '$lib/ds/projects/ChatPaneHeader.svelte';
	import PaneTabs, { type PaneView } from '$lib/ds/projects/PaneTabs.svelte';
	import ProjectDialog from '$lib/ds/projects/ProjectDialog.svelte';
	import DeployRow from '$lib/ds/projects/DeployRow.svelte';
	import AddressRow from '$lib/ds/projects/AddressRow.svelte';
	import SubdomainField, { type SubdomainCheck } from '$lib/ds/projects/SubdomainField.svelte';
	import { invalidate } from '$app/navigation';
	import FileList from '$lib/ds/build/FileList.svelte';
	import PhoneFrame from '$lib/ds/build/PhoneFrame.svelte';
	import PublishButton from '$lib/ds/build/PublishButton.svelte';
	import WorkspaceSheet from '$lib/ds/build/WorkspaceSheet.svelte';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';

	// A project's workspace (Flow / Build — Sign in → live URL o882RC, Building and Live; Workspace
	// tools U16wU, Changes ready → Publish → Publish failed → Published → Deployments): the project
	// bar with Publish, the project's chat where Koda works (the chat itself, as on its own page), and
	// the preview pane, over which a project tool opens as a sheet. The preview shows the live app
	// once there is one, else the project's own files when they're a static site; anything else waits
	// for its runtime. Code lists the project's files. Publish confirms what goes live; while it runs
	// the workspace looks again every few seconds; a failed publish keeps the live deploy up and says
	// why. Deployments lists every publish, live and failed, with Roll back (a publish too). Domain,
	// Monitor, Database, Settings, Share and the terminal come with their sheets (AGENTS.md §9).
	let { data } = $props();

	const project = $derived(data.project);
	const time = appClock();
	const hasIndex = $derived(project.files.some((file) => file.path === 'index.html'));
	const staticPreview = $derived(project.stack === 'static' && hasIndex ? `/projects/${project.id}/preview/` : null);

	// The deploy overview, looked at again while a publish runs.
	let deploy = $state<DeployOverview | null>(untrack(() => data.deploy));
	$effect(() => {
		deploy = data.deploy;
	});
	let now = $state(new Date());
	const liveRow = $derived(deploy?.publishes.find((one) => one.is_live && one.environment === 'production'));
	const liveUrl = $derived(deploy?.url ?? data.live?.url ?? null);
	const liveNumber = $derived(liveRow?.version ?? data.live?.deploys ?? null);
	const frame = $derived(liveUrl ?? staticPreview);
	// What the Publish button says.
	const button = $derived(deploy ? publishState(deploy, project.files.length, now) : ({ state: 'off' } as const));
	const rows = $derived(deploy ? deployRows(deploy.publishes.filter((one) => one.environment === 'production'), now, time.timeZone) : []);
	const nextNumber = $derived((deploy?.publishes.reduce((top, one) => Math.max(top, one.version), 0) ?? 0) + 1);

	const status = $derived(
		liveUrl
			? { label: m.workspace_status_live(), tone: 'accent' as const }
			: project.status === 'building'
				? { label: m.workspace_status_building(), tone: 'quiet' as const }
				: project.status === 'ready'
					? { label: m.workspace_status_ready(), tone: 'warning' as const }
					: { label: m.workspace_status_planning(), tone: 'quiet' as const },
	);

	async function refresh() {
		const response = await fetch(`/projects/${project.id}/deploy`).catch(() => null);
		if (!response?.ok) return;
		const next = (await response.json()) as DeployOverview;
		settled(deploy, next);
		deploy = next;
	}

	// A publish that has just finished says so: live with a way back, or failed with its reason.
	function settled(before: DeployOverview | null, after: DeployOverview) {
		const was = before?.publishes.find((one) => one.status === 'running');
		if (!was) return;
		const done = after.publishes.find((one) => one.id === was.id);
		if (done?.status === 'succeeded') {
			const previous = after.publishes.find((one) => one.status === 'succeeded' && one.id !== done.id && one.can_roll_back && one.environment === done.environment);
			toasts.show({
				kind: 'success',
				title: m.deploy_live({ number: done.version }),
				...(previous && { action: { label: m.deploy_roll_back_to({ number: previous.version }), onclick: () => void rollBack(previous.id) } }),
			});
		} else if (done?.status === 'failed') {
			toasts.show({ kind: 'error', title: m.deploy_failed({ number: done.version }), detail: done.error ?? m.deploy_failed_body() });
		}
	}

	onMount(() => {
		const tick = setInterval(() => {
			now = new Date();
			if (deploy && publishing(deploy)) void refresh();
		}, 3000);
		return () => clearInterval(tick);
	});

	let confirming = $state(false);
	let sending = $state(false);
	async function startPublish() {
		sending = true;
		const result = await pageAction(`/projects/${project.id}`, 'publish');
		sending = false;
		confirming = false;
		if (!result.ok) {
			if (result.message) toasts.show({ kind: 'error', title: m.deploy_couldnt_start(), detail: result.message });
			return;
		}
		await refresh();
	}

	let rollingBack = $state<string>();
	async function rollBack(publishId: string) {
		rollingBack = publishId;
		const result = await pageAction(`/projects/${project.id}`, 'rollback', { publish: publishId });
		rollingBack = undefined;
		if (!result.ok) {
			if (result.message) toasts.show({ kind: 'error', title: m.deploy_couldnt_start(), detail: result.message });
			return;
		}
		await refresh();
	}

	// Domain (Workspace tools steps 8–9): the app's own address under Nomi's apps domain, renamed by
	// the project's admins, checked as it's typed.
	const address = $derived(data.address);
	let renaming = $state(false);
	let name = $state('');
	let check = $state<SubdomainCheck>({ state: 'idle' });
	let saving = $state(false);
	let checkTimer: ReturnType<typeof setTimeout> | undefined;
	function startRenaming() {
		name = address?.subdomain ?? '';
		check = { state: 'idle' };
		renaming = true;
	}
	function checkSoon(next: string) {
		clearTimeout(checkTimer);
		const wanted = next.trim().toLowerCase();
		if (!wanted || wanted === address?.subdomain) return void (check = { state: 'idle' });
		check = { state: 'checking' };
		checkTimer = setTimeout(async () => {
			const response = await fetch(`/projects/${project.id}/domain/check?name=${encodeURIComponent(wanted)}`).catch(() => null);
			const answer = response ? ((await response.json().catch(() => null)) as { available?: boolean; message?: string; suggestions?: string[]; error?: string } | null) : null;
			if (name.trim().toLowerCase() !== wanted) return;
			if (!answer || answer.error !== undefined) check = { state: 'taken', message: answer?.error || m.domain_check_failed() };
			else check = answer.available ? { state: 'available' } : { state: 'taken', message: answer.message || m.domain_taken(), suggestions: answer.suggestions };
		}, 400);
	}
	async function saveName() {
		saving = true;
		const result = await pageAction(`/projects/${project.id}`, 'domain', { name });
		saving = false;
		if (!result.ok) {
			check = { state: 'taken', message: result.message || m.action_failed() };
			return;
		}
		renaming = false;
		toasts.show({ kind: 'success', title: m.domain_changed() });
		await invalidate('app:workspace-project');
	}

	let view = $state<PaneView>('preview');
	let device = $state<'desktop' | 'phone'>('desktop');
	let reloads = $state(0);
	let sheet = $state<'deployments' | 'domain' | null>(null);
	// An opened sheet takes the focus, so Escape (and a screen reader) is in it at once.
	let sheetBox = $state<HTMLElement>();
	$effect(() => {
		if (sheet) sheetBox?.querySelector<HTMLElement>('section')?.focus();
	});
	const shownUrl = $derived(liveUrl ? liveUrl.replace(/^https?:\/\//, '').replace(/\/$/, '') : m.workspace_preview_files());
	const initials = $derived(
		(data.person?.name ?? '')
			.split(/\s+/)
			.map((word: string) => word.charAt(0))
			.join('')
			.slice(0, 2)
			.toUpperCase() || '·',
	);
</script>

<svelte:head><title>{project.name} · {m.app_name()}</title></svelte:head>

<ProjectWorkspace>
	{#snippet projectbar()}
		<ProjectBar
			backHref="/projects"
			backLabel={m.workspace_back()}
			agent="koda"
			name={project.name}
			{status}
			space={project.org_name}
			url={liveUrl ?? undefined}
			nav={[...(deploy ? [{ id: 'deployments', label: m.workspace_deployments(), icon: History }] : []), ...(data.address ? [{ id: 'domain', label: m.workspace_domain(), icon: Link }] : [])]}
			navLabel={m.workspace_tools()}
			active={sheet ?? undefined}
			onnav={(id) => (sheet = sheet === id ? null : (id as typeof sheet))}
			shareLabel={m.workspace_share()}
			exportLabel={m.workspace_export()}
			{initials}
			onexport={() => (location.href = `/projects/${project.id}/export`)}
		>
			{#snippet publish()}
				{#if button.state === 'ready'}
					<PublishButton state="ready" count={button.count} onpublish={() => (confirming = true)} />
				{:else if button.state === 'publishing'}
					<PublishButton state="publishing" elapsed={button.elapsed} />
				{:else if button.state === 'failed'}
					<PublishButton state="failed" onretry={() => (confirming = true)} />
				{:else if button.state === 'idle'}
					<PublishButton state="idle" />
				{/if}
			{/snippet}
		</ProjectBar>
	{/snippet}
	{#snippet chat()}
		<ChatPaneHeader title={m.workspace_chat()} crew={['koda', 'nomi']} />
		<div class="min-h-0 flex-1"><ChatView {data} pane /></div>
	{/snippet}
	<div class="relative flex min-h-0 flex-1 flex-col">
		<PaneTabs
			labels={{
				views: m.workspace_views(),
				preview: m.workspace_preview(),
				code: m.workspace_code(),
				terminal: m.workspace_terminal(),
				device: m.workspace_device(),
				desktop: m.workspace_desktop(),
				phone: m.workspace_phone(),
				reload: m.workspace_reload(),
				open: m.workspace_open_tab(),
				close: m.workspace_close_preview(),
			}}
			views={['preview', 'code']}
			bind:view
			bind:device
			address={frame ? { url: frame, shown: shownUrl, live: liveUrl ? m.workspace_live() : undefined, deploy: liveUrl && liveNumber ? m.projects_deploy({ number: liveNumber }) : undefined } : undefined}
			onreload={() => (reloads += 1)}
		/>
		<div class="min-h-0 flex-1 overflow-auto p-5">
			{#if view === 'code'}
				{#if project.files.length}
					<FileList files={project.files.map((file) => ({ path: file.path, state: 'done' as const }))} total={project.files.length} />
				{:else}
					<EmptyState icon={MonitorX} title={m.workspace_no_files_title()} body={m.workspace_no_files_body()} />
				{/if}
			{:else if frame}
				{#key reloads}
					{#if device === 'phone'}
						<div class="flex justify-center"><PhoneFrame src={frame} title={project.name} /></div>
					{:else}
						<iframe src={frame} title={project.name} class="h-full min-h-150 w-full rounded-md bg-surface outline -outline-offset-1 outline-border"></iframe>
					{/if}
				{/key}
			{:else}
				<EmptyState icon={MonitorX} title={m.workspace_nothing_title()} body={project.status === 'building' ? m.workspace_nothing_building() : m.workspace_nothing_body()} />
			{/if}
		</div>

		{#if sheet === 'domain' && address}
			<div bind:this={sheetBox} class="absolute inset-y-0 right-0 flex w-180 max-w-full">
				<WorkspaceSheet icon={Link} title={m.workspace_domain()} meta={project.name} onclose={() => (sheet = null)}>
					{#if !address.hostname}
						<EmptyState icon={Globe} title={m.domain_none_title()} body={m.domain_none_body()} />
					{:else}
						<div class="flex flex-col gap-2">
							<p class="font-mono text-label tracking-label text-text-3 uppercase">{m.domain_addresses()}</p>
							<AddressRow address={address.hostname} href={address.url ?? undefined} state={liveUrl ? { label: m.deploy_state_live(), tone: 'live' } : undefined} />
							{#if address.pending_hostname}<AddressRow address={address.pending_hostname} state={{ label: m.domain_next_publish(), tone: 'waiting' }} strong />{/if}
						</div>
						{#if renaming}
							<div class="flex flex-col gap-3">
								<SubdomainField
									bind:value={name}
									suffix={address.base_domain ?? address.hostname.split('.').slice(1).join('.')}
									{check}
									labels={{ field: m.domain_field(), checking: m.domain_checking(), available: m.domain_available(), suggestions: m.domain_free_instead() }}
									oninput={checkSoon}
									onpick={(picked) => {
										name = picked;
										checkSoon(picked);
									}}
								/>
								<div class="flex gap-2">
									<Button variant="inverse" label={m.domain_save()} loading={saving} disabled={!name.trim() || check.state === 'taken' || check.state === 'checking'} onclick={saveName} />
									<Button variant="ghost" label={m.action_cancel()} onclick={() => (renaming = false)} />
								</div>
								<p class="text-ui text-text-3">{m.domain_rename_note()}</p>
							</div>
						{:else if address.can_edit}
							<div><Button variant="secondary" label={m.domain_change()} onclick={startRenaming} /></div>
						{/if}
					{/if}
				</WorkspaceSheet>
			</div>
		{/if}
		{#if sheet === 'deployments' && deploy}
			<div bind:this={sheetBox} class="absolute inset-y-0 right-0 flex w-180 max-w-full">
				<WorkspaceSheet
					icon={History}
					title={m.workspace_deployments()}
					meta={rows.length === 1 ? m.deploy_count_one({ name: project.name }) : m.deploy_count({ name: project.name, count: rows.length })}
					onclose={() => (sheet = null)}
				>
					{#if !deploy.available}
						<EmptyState icon={Rocket} title={m.deploy_unavailable_title()} body={m.deploy_unavailable_body()} />
					{:else if !rows.length}
						<EmptyState icon={Rocket} title={m.deploy_none_title()} body={m.deploy_none_body()} />
					{:else}
						<div class="flex flex-col gap-4">
							{#if liveUrl}<p class="font-mono text-label tracking-label text-text-3 uppercase">{m.deploy_live_at({ address: shownUrl })}</p>{/if}
							<ul class="flex flex-col">
								{#each rows as row (row.id)}
									<DeployRow
										number={row.number}
										title={row.title}
										meta={row.meta}
										state={row.state}
										error={row.error}
										labels={{ live: m.deploy_state_live(), failed: m.deploy_state_failed(), running: m.deploy_state_running(), rollback: m.deploy_roll_back_to({ number: '{number}' }) }}
										busy={rollingBack === row.id}
										onrollback={() => rollBack(row.id)}
									/>
								{/each}
							</ul>
							<p class="text-ui text-text-3">{m.deploy_rollback_note({ number: nextNumber })}</p>
						</div>
					{/if}
				</WorkspaceSheet>
			</div>
		{/if}
	</div>
</ProjectWorkspace>

<ProjectDialog
	open={confirming}
	title={button.state === 'ready' ? (button.count === 1 ? m.deploy_confirm_one() : m.deploy_confirm({ count: button.count })) : m.deploy_confirm_retry()}
	description={liveUrl ? m.deploy_confirm_body({ address: shownUrl, number: nextNumber }) : m.deploy_confirm_first({ number: nextNumber })}
	width={480}
	onclose={() => (confirming = false)}
>
	{#if liveNumber}<p class="font-mono text-label text-text-3">{m.deploy_confirm_note({ number: liveNumber })}</p>{/if}
	{#snippet actions()}
		<Button variant="ghost" label={m.action_cancel()} onclick={() => (confirming = false)} />
		<Button variant="primary" iconEnd={Rocket} label={m.deploy_publish_as({ number: nextNumber })} loading={sending} onclick={startPublish} />
	{/snippet}
</ProjectDialog>
