<!-- frontend/src/lib/components/blocks/TodoListBlock.svelte -->
<script lang="ts">
	import type { ContentBlock } from '$lib/types';

	let { block }: { block: Extract<ContentBlock, { kind: 'todo_list' }> } = $props();
</script>

<div class="m3-block-card m3-block-card--todo">
	<p class="md-label-large" style="color: var(--md-sys-color-on-surface-variant)">To-do list</p>
	<ul>
		{#each block.items as item (item.id)}
			<li class="m3-todo-item m3-todo-item--{item.status}">
				<span class="m3-todo-item__mark" aria-hidden="true">
					{#if item.status === 'done'}✓{:else if item.status === 'in_progress'}◐{:else}○{/if}
				</span>
				<span>{item.text}</span>
			</li>
		{/each}
	</ul>
</div>

<style>
	.m3-block-card--todo {
		padding: 12px 14px;
		border-radius: var(--md-sys-shape-corner-medium);
		border: 1px solid var(--md-sys-color-outline-variant);
		background: var(--md-sys-color-surface-container-low);
	}
	.m3-block-card--todo ul {
		list-style: none;
		margin: 8px 0 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.m3-todo-item {
		display: flex;
		align-items: center;
		gap: 8px;
		font-family: var(--md-sys-typescale-body-medium-font);
		font-size: var(--md-sys-typescale-body-medium-size);
		color: var(--md-sys-color-on-surface);
	}
	.m3-todo-item--done {
		color: var(--md-sys-color-on-surface-variant);
		text-decoration: line-through;
	}
	.m3-todo-item__mark {
		width: 18px;
		text-align: center;
		color: var(--md-sys-color-primary);
	}
</style>
