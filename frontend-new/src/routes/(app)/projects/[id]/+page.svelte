<script lang="ts">
	import { MonitorX } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import ChatView from '$lib/chat/ChatView.svelte';
	import ProjectWorkspace from '$lib/ds/projects/ProjectWorkspace.svelte';
	import ProjectBar from '$lib/ds/projects/ProjectBar.svelte';
	import ChatPaneHeader from '$lib/ds/projects/ChatPaneHeader.svelte';
	import PaneTabs, { type PaneView } from '$lib/ds/projects/PaneTabs.svelte';
	import FileList from '$lib/ds/build/FileList.svelte';
	import PhoneFrame from '$lib/ds/build/PhoneFrame.svelte';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';

	// A project's workspace (Flow / Build — Sign in → live URL o882RC, steps 5 and 6: Building, Live):
	// the project bar, the project's chat where Koda works (the chat itself, as on its own page), and
	// the preview pane. The preview shows the live app once there is one, else the project's own files
	// when they're a static site; anything else waits for its runtime. Code lists the project's files.
	// The project tools (Deployments, Domain, Monitor, Database, Settings), Publish, Share and the
	// terminal come with their sheets (AGENTS.md §9).
	let { data } = $props();

	const project = $derived(data.project);
	const hasIndex = $derived(project.files.some((file) => file.path === 'index.html'));
	const staticPreview = $derived(project.stack === 'static' && hasIndex ? `/projects/${project.id}/preview/` : null);
	const frame = $derived(data.live?.url ?? staticPreview);
	const status = $derived(
		data.live
			? { label: m.workspace_status_live(), tone: 'accent' as const }
			: project.status === 'building'
				? { label: m.workspace_status_building(), tone: 'quiet' as const }
				: project.status === 'ready'
					? { label: m.workspace_status_ready(), tone: 'warning' as const }
					: { label: m.workspace_status_planning(), tone: 'quiet' as const },
	);

	let view = $state<PaneView>('preview');
	let device = $state<'desktop' | 'phone'>('desktop');
	let reloads = $state(0);
	const shownUrl = $derived(data.live ? data.live.url.replace(/^https?:\/\//, '').replace(/\/$/, '') : m.workspace_preview_files());
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
			url={data.live?.url}
			nav={[]}
			navLabel={m.workspace_tools()}
			shareLabel={m.workspace_share()}
			exportLabel={m.workspace_export()}
			{initials}
			onexport={() => (location.href = `/projects/${project.id}/export`)}
		/>
	{/snippet}
	{#snippet chat()}
		<ChatPaneHeader title={m.workspace_chat()} crew={['koda', 'nomi']} />
		<div class="min-h-0 flex-1"><ChatView {data} pane /></div>
	{/snippet}
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
		address={frame ? { url: frame, shown: shownUrl, live: data.live ? m.workspace_live() : undefined, deploy: data.live?.deploys ? m.projects_deploy({ number: data.live.deploys }) : undefined } : undefined}
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
</ProjectWorkspace>
