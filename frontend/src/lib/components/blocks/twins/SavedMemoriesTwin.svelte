<script lang="ts">
	// What the crew remembered from the person's message, said where it happened (Chat responses →
	// Memory / Saved), with edit and undo right there. Edit rewrites it in place (MemoryEditForm,
	// the Memory page's own form); undo forgets it through the Memory page's forget action.
	import { m } from '$lib/paraglide/messages';
	import { deserialize } from '$app/forms';
	import MemorySaved from '$lib/components/ds/chat/MemorySaved.svelte';
	import MemoryEditForm from '$lib/components/MemoryEditForm.svelte';
	import type { MessageMemory } from '$lib/types';

	let { memories }: { memories: MessageMemory[] } = $props();

	const initial = () => memories.map((memory) => ({ ...memory }));
	let shown = $state(initial());
	$effect(() => {
		shown = memories.map((memory) => ({ ...memory }));
	});
	let editing = $state<string | null>(null);
	let error = $state<string | null>(null);

	async function forget(id: string) {
		error = null;
		const body = new FormData();
		body.set('id', id);
		const response = await fetch('/memory?/forget', { method: 'POST', body, headers: { accept: 'application/json' } }).catch(() => null);
		const result = response ? deserialize(await response.text()) : null;
		if (result?.type === 'success') shown = shown.filter((memory) => memory.id !== id);
		else error = m.mem_forget_failed();
	}
</script>

{#if shown.length > 0}
	<div class="saved">
		{#each shown as memory (memory.id)}
			{#if editing === memory.id}
				<MemoryEditForm
					id={memory.id}
					content={memory.content}
					kind={memory.kind}
					onsaved={(content, kind) => {
						memory.content = content;
						memory.kind = kind;
						editing = null;
					}}
					oncancel={() => (editing = null)}
				/>
			{:else}
				<MemorySaved fact={memory.content} onedit={() => (editing = memory.id)} onundo={() => forget(memory.id)} />
			{/if}
		{/each}
		{#if error}<p class="saved__error" role="alert">{error}</p>{/if}
	</div>
{/if}

<style>
	.saved {
		display: flex;
		flex-direction: column;
		gap: 6px;
		margin-top: 8px;
	}
	.saved__error {
		margin: 0;
		color: var(--md-sys-color-error);
		font-size: 0.8125rem;
	}
</style>
