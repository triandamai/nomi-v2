<script lang="ts">
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// A crew member in a list, with what they do and their state.
	type Props = { agent: CrewId; name: string; role: string; state?: string; href?: string };
	let { agent, name, role, state, href }: Props = $props();
</script>

{#snippet body()}
	<Mascot {agent} size={32} />
	<span class="text">
		<span class="name">{name}</span>
		<span class="role">{role}</span>
	</span>
	{#if state}<span class="state">{state}</span>{/if}
{/snippet}

{#if href}
	<a {href} class="row row--link">{@render body()}</a>
{:else}
	<div class="row">{@render body()}</div>
{/if}

<style>
	.row {
		display: flex;
		width: 100%;
		align-items: center;
		gap: 16px;
		min-height: 56px;
		padding: 8px 16px;
		border-radius: var(--md-sys-shape-corner-large);
		color: inherit;
		text-decoration: none;
	}
	.row--link:hover {
		background: var(--md-sys-color-surface-container-high);
	}
	.row--link:focus-visible {
		outline: 3px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.text {
		display: flex;
		min-width: 0;
		flex: 1;
		flex-direction: column;
		gap: 2px;
	}
	.name {
		font: var(--md-sys-typescale-body-large-weight) var(--md-sys-typescale-body-large-size) / var(--md-sys-typescale-body-large-line-height) var(--md-sys-typescale-body-large-font);
		color: var(--md-sys-color-on-surface);
	}
	.role {
		font: var(--md-sys-typescale-body-medium-weight) var(--md-sys-typescale-body-medium-size) / var(--md-sys-typescale-body-medium-line-height) var(--md-sys-typescale-body-medium-font);
		color: var(--md-sys-color-on-surface-variant);
	}
	.state {
		font-family: var(--md-ref-typeface-mono);
		font-size: var(--md-sys-typescale-label-small-size);
		color: var(--md-sys-color-on-surface-variant);
	}
</style>
