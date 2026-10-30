<script lang="ts">
	import Status from '../status/Status.svelte';
	import type { CrewId } from '../agents/crew';

	// A project in a list: its tile (its initial), what it is, whether it's live and where, and when
	// it last changed. `tone` picks the crew colour where the pen tints the tile; the M3 tile keeps
	// to the theme's container colours, since agent gradients belong to agents.
	type Props = {
		href: string;
		name: string;
		description?: string;
		tone: CrewId;
		status: { kind: 'live' | 'ready' | 'idle' | 'failed' | 'attention'; label: string };
		url?: string;
		updated: string;
	};
	let { href, name, description, tone, status, url, updated }: Props = $props();
</script>

<a {href} class="project" data-tone={tone}>
	<span class="who">
		<span class="tile" aria-hidden="true">{name.charAt(0).toUpperCase()}</span>
		<span class="text">
			<span class="name">{name}</span>
			{#if description}<span class="description">{description}</span>{/if}
		</span>
	</span>
	<span class="where">
		<Status kind={status.kind} label={status.label} />
		{#if url}<span class="meta url">{url}</span>{/if}
	</span>
	<span class="meta">{updated}</span>
</a>

<style>
	.project {
		display: flex;
		width: 100%;
		min-height: 72px;
		align-items: center;
		gap: 16px;
		padding: 12px 16px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
		text-decoration: none;
	}
	.project:hover {
		background: var(--md-sys-color-surface-container-low);
	}
	.project:focus-visible {
		outline: 3px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.who {
		display: flex;
		min-width: 0;
		flex: 1;
		align-items: center;
		gap: 16px;
	}
	.tile {
		display: inline-flex;
		width: 40px;
		height: 40px;
		flex-shrink: 0;
		align-items: center;
		justify-content: center;
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
		font: var(--md-sys-typescale-title-medium-weight) var(--md-sys-typescale-title-medium-size) / var(--md-sys-typescale-title-medium-line-height) var(--md-sys-typescale-title-medium-font);
	}
	.text {
		display: flex;
		min-width: 0;
		flex: 1;
		flex-direction: column;
		gap: 2px;
	}
	.name,
	.description {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.name {
		font: var(--md-sys-typescale-title-small-weight) var(--md-sys-typescale-title-small-size) / var(--md-sys-typescale-title-small-line-height) var(--md-sys-typescale-title-small-font);
	}
	.description {
		color: var(--md-sys-color-on-surface-variant);
		font: var(--md-sys-typescale-body-medium-weight) var(--md-sys-typescale-body-medium-size) / var(--md-sys-typescale-body-medium-line-height) var(--md-sys-typescale-body-medium-font);
	}
	.where {
		display: flex;
		width: 240px;
		flex-shrink: 0;
		flex-direction: column;
		gap: 4px;
	}
	.meta {
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-ref-typeface-mono);
		font-size: var(--md-sys-typescale-label-small-size);
	}
	.url {
		word-break: break-all;
	}
	@media (max-width: 767px) {
		.where {
			display: none;
		}
	}
</style>
