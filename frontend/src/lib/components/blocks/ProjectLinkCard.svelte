<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	// A request to build something moved to its own project chat: where to follow it.
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import type { ContentBlock } from '$lib/types';

	let { block }: { block: Extract<ContentBlock, { kind: 'project_link' }> } = $props();
</script>

<a class="project-link" href="/projects/session/{block.session_id}" data-sveltekit-reload>
	<span class="project-link__crew" aria-hidden="true">
		<AgentShape agent="planning" size={40} working />
		<AgentShape agent="coding" size={40} working />
	</span>
	<span class="project-link__text">
		<span class="project-link__eyebrow">{m.project_link_eyebrow()}</span>
		<span class="project-link__name">{block.name}</span>
		<span class="project-link__body">{m.project_link_body()}</span>
	</span>
	<span class="project-link__action">{m.project_link_open()}</span>
</a>

<style>
	.project-link {
		display: grid;
		grid-template-columns: auto 1fr;
		gap: 14px 16px;
		max-width: 440px;
		padding: 18px 20px;
		border-radius: var(--nomi-shape-bubble-start);
		background: var(--md-sys-color-tertiary-container);
		color: var(--md-sys-color-on-tertiary-container);
		text-decoration: none;
		transition:
			border-radius var(--nomi-motion-spatial-fast, 200ms),
			transform var(--nomi-motion-spatial-fast, 200ms);
	}
	/* The whole card is the link: no underlines from the chat's link styles. */
	a.project-link.project-link,
	a.project-link.project-link:hover {
		text-decoration: none;
	}
	.project-link:hover {
		border-radius: var(--md-sys-shape-corner-extra-large);
	}
	.project-link:active {
		transform: scale(0.98);
	}
	.project-link:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.project-link__crew {
		display: flex;
		align-items: center;
	}
	.project-link__crew :global(> :last-child) {
		margin-left: -12px;
	}
	.project-link__text {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.project-link__eyebrow {
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.6875rem;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		opacity: 0.8;
	}
	.project-link__name {
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.125rem;
		font-weight: 700;
		line-height: 1.3;
		overflow-wrap: anywhere;
	}
	.project-link__body {
		font-size: 0.875rem;
		line-height: 1.45;
		opacity: 0.85;
	}
	.project-link__action {
		grid-column: 1 / -1;
		justify-self: start;
		display: inline-flex;
		align-items: center;
		height: 40px;
		padding: 0 20px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-tertiary);
		color: var(--md-sys-color-on-tertiary);
		font-weight: 600;
		font-size: 0.875rem;
	}
</style>
