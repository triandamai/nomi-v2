<script lang="ts">
	import { deserialize } from '$app/forms';
	import { page } from '$app/state';
	import ChatThread from '$lib/components/ChatThread.svelte';
	import CodeEditor from '$lib/components/CodeEditor.svelte';
	import List from '$lib/components/m3/List.svelte';
	import ListItem from '$lib/components/m3/ListItem.svelte';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let view = $state<'preview' | 'code'>('preview');
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
			saveError = (result.type === 'failure' && (result.data?.error as string)) || 'Failed to save.';
		}
	}
</script>

<div class="flex h-full">
	<div class="w-[420px] shrink-0" style="border-right: 1px solid var(--md-sys-color-outline-variant)">
		<ChatThread
			sessionId={page.params.sessionId as string}
			messages={data.messages}
			agentActivity={data.agentActivity}
			sendError={form?.error ?? null}
		/>
	</div>

	<div class="flex min-w-0 flex-1 flex-col">
		{#if !data.project}
			<div class="flex h-full items-center justify-center p-8 text-center">
				<p class="md-body-large" style="color: var(--md-sys-color-on-surface-variant)">
					Describe what you'd like to build in the chat — a project will show up here once nomi gets started.
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
					Preview
				</button>
				<button
					type="button"
					class="md-label-small rounded-full px-3 py-1"
					style="border: 1px solid var(--md-sys-color-outline); background: {view === 'code' ? 'var(--md-sys-color-secondary-container)' : 'transparent'}; color: var(--md-sys-color-on-surface); cursor: pointer"
					onclick={() => (view = 'code')}
				>
					Code
				</button>
			</div>

			<div class="min-h-0 flex-1">
				{#if view === 'preview'}
					<iframe
						title="Project preview"
						src={`/projects/${project.id}/preview/`}
						sandbox="allow-scripts"
						class="h-full w-full"
						style="border: none; background: white"
					></iframe>
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
								<p class="md-body-small" style="color: var(--md-sys-color-on-surface-variant)">No files yet.</p>
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
									<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">Select a file to edit.</p>
								</div>
							{/if}
						</div>
					</div>
				{/if}
			</div>
		{/if}
	</div>
</div>
