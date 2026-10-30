<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { page } from '$app/state';
	import { deserialize } from '$app/forms';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import MemoryEditForm from '$lib/components/MemoryEditForm.svelte';
	import { kindLabel } from '$lib/memory';
	import type { UsedMemory } from '$lib/types';

	// Opened from a reply's "Used 2 memories": the memories it drew on, each fixed or forgotten
	// right here. (Why a reply missed is the thumbs-down's own form, DislikeFeedback.)
	let {
		open = $bindable(false),
		messageId,
		memoryCount,
	}: {
		open?: boolean;
		messageId: string;
		memoryCount: number;
	} = $props();

	let memories = $state<UsedMemory[] | null>(null);
	let loadFailed = $state(false);
	let editing = $state<string | null>(null);
	let note = $state<string | null>(null);


	$effect(() => {
		if (open) {
			editing = null;
			note = null;
		}
	});
	$effect(() => {
		if (open && memories === null && memoryCount > 0) load();
	});

	async function load() {
		loadFailed = false;
		const response = await fetch(`${page.url.pathname}/message/${messageId}/memories`);
		if (response.ok) memories = (await response.json()) as UsedMemory[];
		else loadFailed = true;
	}

	async function forget(memory: UsedMemory) {
		const body = new FormData();
		body.set('id', memory.id);
		const result = deserialize(await (await fetch('/memory?/forget', { method: 'POST', body })).text());
		if (result.type === 'success') {
			memories = (memories ?? []).filter((x) => x.id !== memory.id);
			note = m.mem_forgotten();
		}
	}
</script>

<BottomSheet bind:open>
	<div class="sheet">
		<h2 class="sheet__title">{m.used_title({ count: memoryCount })}</h2>
		<p class="sheet__lede">{m.used_lede()}</p>

		{#if memoryCount > 0}
			{#if loadFailed}
				<p class="sheet__note" role="alert">{m.used_load_failed()}</p>
			{:else if memories === null}
				<p class="sheet__note">{m.common_loading()}</p>
			{:else if memories.length === 0}
				<p class="sheet__note">{m.used_none_left()}</p>
			{:else}
				<ul class="used">
					{#each memories as memory (memory.id)}
						<li class="used__item" class:used__item--archived={memory.archived}>
							{#if editing === memory.id}
								<MemoryEditForm
									id={memory.id}
									content={memory.content}
									kind={memory.kind}
									oncancel={() => (editing = null)}
									onsaved={(content, kind) => {
										memories = (memories ?? []).map((x) => (x.id === memory.id ? { ...x, content, kind, archived: false } : x));
										editing = null;
										note = m.mem_edited();
									}}
								/>
							{:else}
								<span class="used__kind">{kindLabel(memory.kind)}</span>
								<span class="used__text">{memory.content}</span>
								{#if memory.archived}<span class="used__archived">{m.used_archived()}</span>{/if}
								<span class="used__actions">
									<button type="button" class="used__btn" onclick={() => (editing = memory.id)}>{m.mem_fix()}</button>
									<button type="button" class="used__btn used__btn--danger" onclick={() => forget(memory)}>{m.mem_forget()}</button>
								</span>
							{/if}
						</li>
					{/each}
				</ul>
			{/if}
		{/if}

		{#if note}<p class="sheet__note" role="status">{note}</p>{/if}
		<a class="sheet__link" href="/memory">{m.used_all_memories()}</a>
	</div>
</BottomSheet>

<style>
	.sheet {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.sheet__title {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.3125rem;
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}
	.sheet__lede,
	.sheet__note {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.9375rem;
	}
	.sheet__note {
		color: var(--md-sys-color-on-surface);
		font-weight: 600;
	}
	.used {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.used__item {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 6px 10px;
		padding: 12px 12px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container);
		color: var(--md-sys-color-on-surface);
	}
	.used__item:has(:global(form)) {
		display: block;
	}
	.used__item--archived .used__text {
		color: var(--md-sys-color-on-surface-variant);
		text-decoration: line-through;
	}
	.used__kind {
		padding: 2px 8px;
		border-radius: 999px;
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
		font-size: 0.6875rem;
		font-weight: 700;
	}
	.used__text {
		flex: 1 1 200px;
		min-width: 0;
		overflow-wrap: anywhere;
	}
	.used__archived {
		font-size: 0.75rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.used__actions {
		display: flex;
		gap: 4px;
	}
	.used__btn {
		min-height: 36px;
		padding: 0 12px;
		border: none;
		border-radius: 999px;
		background: transparent;
		color: var(--md-sys-color-primary);
		font: inherit;
		font-size: 0.875rem;
		font-weight: 600;
		cursor: pointer;
	}
	.used__btn:hover {
		background: color-mix(in srgb, var(--md-sys-color-primary) 8%, transparent);
	}
	.used__btn:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.used__btn--danger {
		color: var(--md-sys-color-error);
	}
	.sheet__link {
		align-self: flex-start;
		color: var(--md-sys-color-primary);
		font-weight: 600;
		font-size: 0.875rem;
	}
</style>
