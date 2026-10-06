<script lang="ts">
	import type { Snippet } from 'svelte';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';

	// A page's title block: display title, a line on what the page is for, the agent shape it
	// belongs to, and the page's main actions.
	let {
		title,
		lede,
		agent,
		actions,
	}: { title: string; lede?: string; agent?: string; actions?: Snippet } = $props();
</script>

<header class="page-header">
	<div class="page-header__text">
		<h1 class="md-display-small page-header__title">{title}</h1>
		{#if lede}<p class="md-body-large page-header__lede">{lede}</p>{/if}
		{#if actions}<div class="page-header__actions">{@render actions()}</div>{/if}
	</div>
	{#if agent}<AgentShape {agent} size={72} class="page-header__mark" />{/if}
</header>

<style>
	.page-header {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 24px;
	}
	.page-header__text {
		display: flex;
		flex-direction: column;
		gap: 8px;
		min-width: 0;
	}
	.page-header__title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
		text-wrap: balance;
	}
	.page-header__lede {
		margin: 0;
		max-width: 62ch;
		color: var(--md-sys-color-on-surface-variant);
	}
	.page-header__actions {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		margin-top: 8px;
	}
	.page-header :global(.page-header__mark) {
		flex: none;
	}
	@media (max-width: 640px) {
		.page-header :global(.page-header__mark) {
			display: none;
		}
	}
</style>
