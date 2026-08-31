<script lang="ts">
	import { deserialize } from '$app/forms';
	import CodeEditor from '$lib/components/CodeEditor.svelte';
	import List from '$lib/components/m3/List.svelte';
	import ListItem from '$lib/components/m3/ListItem.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	let activePath = $state<string | null>(data.project.files[0]?.path ?? null);
	let activeContent = $state('');
	let saveError = $state<string | null>(null);
	let view = $state<'code' | 'plan' | 'preview'>(data.project.plan ? 'plan' : 'code');

	async function openFile(path: string) {
		view = 'code';
		const body = new FormData();
		body.set('path', path);
		const response = await fetch('?/loadFile', { method: 'POST', body });
		const result = deserialize(await response.text());
		activeContent = result.type === 'success' && typeof result.data?.content === 'string' ? result.data.content : '';
		activePath = path;
	}

	async function saveFile(content: string) {
		if (!activePath) return;
		saveError = null;
		const body = new FormData();
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
	<aside class="w-56 shrink-0 overflow-y-auto p-3" style="border-right: 1px solid var(--md-sys-color-outline-variant)">
		<h2 class="md-title-medium mb-2" style="color: var(--md-sys-color-on-surface)">{data.project.name}</h2>
		<div class="mb-3 flex gap-1">
			<button
				type="button"
				class="md-label-small rounded-full px-3 py-1"
				style="border: 1px solid var(--md-sys-color-outline); background: {view === 'plan' ? 'var(--md-sys-color-secondary-container)' : 'transparent'}; color: var(--md-sys-color-on-surface); cursor: pointer"
				onclick={() => (view = 'plan')}
			>
				Plan
			</button>
			<button
				type="button"
				class="md-label-small rounded-full px-3 py-1"
				style="border: 1px solid var(--md-sys-color-outline); background: {view === 'preview' ? 'var(--md-sys-color-secondary-container)' : 'transparent'}; color: var(--md-sys-color-on-surface); cursor: pointer"
				onclick={() => (view = 'preview')}
			>
				Preview
			</button>
		</div>
		<List>
			{#each data.project.files as file (file.path)}
				<button
					type="button"
					class="w-full text-left"
					style="background: none; border: none; padding: 0; cursor: pointer"
					onclick={() => openFile(file.path)}
				>
					<ListItem headline={file.path} selected={activePath === file.path && view === 'code'} />
				</button>
			{/each}
		</List>
		{#if data.project.files.length === 0}
			<p class="md-body-small" style="color: var(--md-sys-color-on-surface-variant)">No files yet.</p>
		{/if}
	</aside>

	<main class="min-w-0 flex-1">
		{#if saveError}
			<p class="md-body-medium p-2" style="color: var(--md-sys-color-error)">{saveError}</p>
		{/if}
		{#if view === 'plan'}
			<div class="h-full overflow-y-auto p-6">
				<pre class="md-body-medium whitespace-pre-wrap" style="color: var(--md-sys-color-on-surface)">{data.project.plan ?? 'No plan yet.'}</pre>
			</div>
		{:else if view === 'preview'}
			<iframe
				title="Project preview"
				src={`/projects/${data.project.id}/preview/`}
				sandbox="allow-scripts"
				class="h-full w-full"
				style="border: none; background: white"
			></iframe>
		{:else if activePath}
			{#key activePath}
				<CodeEditor path={activePath} value={activeContent} onSave={saveFile} />
			{/key}
		{:else}
			<div class="flex h-full items-center justify-center">
				<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">Select a file to edit.</p>
			</div>
		{/if}
	</main>
</div>
