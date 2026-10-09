<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { deserialize } from '$app/forms';
	import { page } from '$app/state';
	import { beforeNavigate } from '$app/navigation';
	import { onMount } from 'svelte';
	import ChatThread from '$lib/components/ChatThread.svelte';
	import CodeEditor from '$lib/components/CodeEditor.svelte';
	import ProjectPreview from '$lib/components/ProjectPreview.svelte';
	import { ProjectRuntime } from '$lib/projectRuntime.svelte';
	import List from '$lib/components/m3/List.svelte';
	import ListItem from '$lib/components/m3/ListItem.svelte';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let view = $state<'preview' | 'code' | 'terminal'>('preview');
	let runtime = $state<ProjectRuntime | null>(null);

	// Projects with a package.json run in the browser (WebContainer); static ones don't.
	const projectId = $derived(data.project?.id ?? null);
	const runsLive = $derived(Boolean(data.project && data.project.stack !== 'static'));

	$effect(() => {
		if (!projectId || !runsLive) return;
		const live = new ProjectRuntime(projectId);
		runtime = live;
		void live.start();
		return () => {
			runtime = null;
			void live.stop();
		};
	});

	// The page needs its cross-origin isolation headers, which only a full load brings. Arriving
	// by an in-app link, load it once more; leaving, load the next page in full so the isolation
	// doesn't follow the person around the app.
	onMount(() => {
		if (!runsLive || window.crossOriginIsolated) return;
		const key = `nomi-isolated:${location.pathname}`;
		try {
			if (sessionStorage.getItem(key)) return;
			sessionStorage.setItem(key, '1');
		} catch {
			return;
		}
		location.reload();
	});
	beforeNavigate(({ to, cancel, willUnload }) => {
		if (willUnload || !to || !window.crossOriginIsolated) return;
		if (to.url.pathname === location.pathname) return;
		cancel();
		location.href = to.url.href;
	});
	let activePath = $state<string | null>(null);
	let activeContent = $state('');
	let saveError = $state<string | null>(null);

	async function openFile(projectId: string, path: string) {
		const body = new FormData();
		body.set('projectId', projectId);
		body.set('path', path);
		const response = await fetch('?/loadFile', { method: 'POST', body });
		const result = deserialize(await response.text());
		activeContent = result.type === 'success' && typeof result.data?.content === 'string' ? result.data.content : '';
		activePath = path;
	}

	async function saveFile(projectId: string, content: string) {
		if (!activePath) return;
		saveError = null;
		const body = new FormData();
		body.set('projectId', projectId);
		body.set('path', activePath);
		body.set('content', content);
		const response = await fetch('?/saveFile', { method: 'POST', body });
		const result = deserialize(await response.text());
		if (result.type !== 'success') {
			saveError = (result.type === 'failure' && (result.data?.error as string)) || m.project_save_failed();
			return;
		}
		await runtime?.writeFile(activePath, content);
	}
</script>

<div class="flex h-full">
	<div class="w-[420px] shrink-0" style="border-right: 1px solid var(--md-sys-color-outline-variant)">
		<ChatThread
			sessionId={page.params.sessionId as string}
			title={data.project?.name ?? m.project_new()}
			context={m.project_context()}
			messages={data.messages}
			agentActivity={data.agentActivity}
			agentStatus={data.agentStatus}
			sendError={form?.error ?? null}
			thinkingLevel={data.thinkingLevel}
		/>
	</div>

	<div class="flex min-w-0 flex-1 flex-col">
		{#if !data.project}
			<div class="flex h-full items-center justify-center p-8 text-center">
				<p class="md-body-large" style="color: var(--md-sys-color-on-surface-variant)">
					{m.project_empty()}
				</p>
			</div>
		{:else}
			{@const project = data.project}
			<div class="flex items-center gap-2 px-4 py-3" style="border-bottom: 1px solid var(--md-sys-color-outline-variant)">
				<h2 class="md-title-medium flex-1 truncate" style="color: var(--md-sys-color-on-surface)">{project.name}</h2>
				<button
					type="button"
					class="md-label-small rounded-full px-3 py-1"
					style="border: 1px solid var(--md-sys-color-outline); background: {view === 'preview' ? 'var(--md-sys-color-secondary-container)' : 'transparent'}; color: var(--md-sys-color-on-surface); cursor: pointer"
					onclick={() => (view = 'preview')}
				>
					{m.project_preview()}
				</button>
				<button
					type="button"
					class="md-label-small rounded-full px-3 py-1"
					style="border: 1px solid var(--md-sys-color-outline); background: {view === 'code' ? 'var(--md-sys-color-secondary-container)' : 'transparent'}; color: var(--md-sys-color-on-surface); cursor: pointer"
					onclick={() => (view = 'code')}
				>
					{m.project_code()}
				</button>
				{#if runtime}
					<button
						type="button"
						class="md-label-small rounded-full px-3 py-1"
						style="border: 1px solid var(--md-sys-color-outline); background: {view === 'terminal' ? 'var(--md-sys-color-secondary-container)' : 'transparent'}; color: var(--md-sys-color-on-surface); cursor: pointer"
						onclick={() => (view = 'terminal')}
					>
						{m.project_terminal()}
					</button>
				{/if}
			</div>
			{#if runtime?.checkFailed}
				<p class="md-body-small px-4 py-2" style="background: var(--md-sys-color-error-container); color: var(--md-sys-color-on-error-container)">
					{runtime.handedToKoda ? m.project_check_failed_koda() : m.project_check_failed()}
				</p>
			{/if}

			<div class="min-h-0 flex-1">
				{#if view === 'preview'}
					<ProjectPreview projectId={project.id} {runtime} />
				{:else if view === 'terminal' && runtime}
					<pre class="terminal">{runtime.log || m.project_terminal_empty()}</pre>
				{:else}
					<div class="flex h-full">
						<aside class="w-56 shrink-0 overflow-y-auto p-3" style="border-right: 1px solid var(--md-sys-color-outline-variant)">
							<List>
								{#each project.files as file (file.path)}
									<button
										type="button"
										class="w-full text-left"
										style="background: none; border: none; padding: 0; cursor: pointer"
										onclick={() => openFile(project.id, file.path)}
									>
										<ListItem headline={file.path} selected={activePath === file.path} />
									</button>
								{/each}
							</List>
							{#if project.files.length === 0}
								<p class="md-body-small" style="color: var(--md-sys-color-on-surface-variant)">{m.project_no_files()}</p>
							{/if}
						</aside>
						<div class="min-w-0 flex-1">
							{#if saveError}
								<p class="md-body-medium p-2" style="color: var(--md-sys-color-error)">{saveError}</p>
							{/if}
							{#if activePath}
								{#key activePath}
									<CodeEditor path={activePath} value={activeContent} onSave={(content) => saveFile(project.id, content)} />
								{/key}
							{:else}
								<div class="flex h-full items-center justify-center">
									<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">{m.project_select_file()}</p>
								</div>
							{/if}
						</div>
					</div>
				{/if}
			</div>
		{/if}
	</div>
</div>

<style>
	.terminal {
		height: 100%;
		margin: 0;
		padding: 12px 16px;
		overflow: auto;
		background: #0f1a14;
		color: #d7e6dc;
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
		line-height: 1.55;
		white-space: pre-wrap;
		word-break: break-word;
	}
</style>
