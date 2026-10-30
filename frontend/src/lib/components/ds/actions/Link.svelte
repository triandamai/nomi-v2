<script lang="ts">
	import { ArrowUpRight } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// A link on its own line ("Upgrade to Pro ↗"), or one inside a sentence. `quiet` is a card
	// footer's secondary link.
	type Props = {
		href: string;
		label: string;
		variant?: 'standalone' | 'inline';
		/** Another site: opens in a new tab, without access to this page. */
		external?: boolean;
		/** Standalone only: secondary text in regular weight, for a card's footer ("Open in Sheets ↗"). */
		quiet?: boolean;
	};
	let { href, label, variant = 'standalone', external = false, quiet = false }: Props = $props();
</script>

<a
	{href}
	target={external ? '_blank' : undefined}
	rel={external ? 'noopener noreferrer' : undefined}
	class="link link--{variant}"
	class:link--quiet={quiet}
>
	{label}
	{#if variant === 'standalone'}<Icon icon={ArrowUpRight} size={16} />{/if}
</a>

<style>
	.link {
		border-radius: var(--md-sys-shape-corner-extra-small);
		color: var(--md-sys-color-primary);
		text-underline-offset: 3px;
	}
	.link:hover {
		text-decoration: underline;
	}
	.link:focus-visible {
		outline: 3px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.link--standalone {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		text-decoration: none;
		font: var(--md-sys-typescale-label-large-weight) var(--md-sys-typescale-label-large-size) / var(--md-sys-typescale-label-large-line-height) var(--md-sys-typescale-label-large-font);
	}
	.link--quiet {
		color: var(--md-sys-color-on-surface-variant);
		font-weight: 400;
	}
	.link--inline {
		text-decoration: underline;
	}
</style>
