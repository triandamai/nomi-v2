<!-- frontend/src/lib/components/blocks/TodoListBlock.svelte -->
<script lang="ts">
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import WavyProgress from '$lib/components/m3/WavyProgress.svelte';
	import type { ContentBlock } from '$lib/types';

	let { block }: { block: Extract<ContentBlock, { kind: 'todo_list' }> } = $props();

	const done = $derived(block.items.filter((item) => item.status === 'done').length);
	const working = $derived(block.items.some((item) => item.status === 'in_progress'));
</script>

<div class="m3-block-card m3-block-card--todo">
	<div class="m3-todo-head">
		<AgentShape agent="planning" size={24} {working} />
		<span class="m3-todo-title">To-do</span>
		<span class="nomi-meta">{done} / {block.items.length}</span>
	</div>
	{#if block.items.length > 0}
		<WavyProgress value={done / block.items.length} tone="sky" label="{done} of {block.items.length} done" />
	{/if}
	<ul>
		{#each block.items as item (item.id)}
			<li class="m3-todo-item m3-todo-item--{item.status}">
				<span class="m3-todo-item__mark" aria-hidden="true">
					{#if item.status === 'done'}
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><path d="m5 12 5 5 9-10" /></svg>
					{/if}
				</span>
				<span>{item.text}</span>
				{#if item.status === 'in_progress'}<span class="sr-only">(in progress)</span>{/if}
			</li>
		{/each}
	</ul>
</div>

<style>
	.m3-block-card--todo {
		display: flex;
		flex-direction: column;
		gap: 10px;
		max-width: 520px;
		padding: 18px 20px 12px;
		border-radius: var(--nomi-shape-bubble-start);
		background: color-mix(in srgb, #7ab0ff 14%, var(--md-sys-color-surface-container-lowest));
	}
	.m3-todo-head {
		display: flex;
		align-items: center;
		gap: 10px;
	}
	.m3-todo-title {
		flex: 1;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.125rem;
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}
	.m3-block-card--todo ul {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
	}
	.m3-todo-item {
		display: flex;
		align-items: center;
		gap: 12px;
		min-height: 40px;
		font-family: var(--md-sys-typescale-body-medium-font);
		font-size: 0.9375rem;
		color: var(--md-sys-color-on-surface);
	}
	.m3-todo-item--done {
		color: var(--md-sys-color-on-surface-variant);
		text-decoration: line-through;
	}
	.m3-todo-item--in_progress {
		font-weight: 650;
	}
	.m3-todo-item__mark {
		flex: none;
		display: flex;
		align-items: center;
		justify-content: center;
		width: 22px;
		height: 22px;
		box-sizing: border-box;
		border-radius: 7px;
		border: 2px solid var(--md-sys-color-outline);
	}
	.m3-todo-item--in_progress .m3-todo-item__mark {
		border-color: #3e7be0;
		border-radius: var(--md-sys-shape-corner-full);
	}
	.m3-todo-item--done .m3-todo-item__mark {
		border-color: #3e7be0;
		background: #3e7be0;
		color: #ffffff;
	}
</style>
