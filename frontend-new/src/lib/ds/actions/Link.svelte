<script lang="ts">
	import { ArrowUpRight } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Design System / Nomi → Actions. A link on its own line ("Upgrade to Pro ↗"), or one inside a
	// sentence ("…or switch to Nomi credits") in text-2, as the pen draws it; both underline on hover.
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
	class={[
		'rounded-xs outline-offset-2',
		variant === 'standalone' && 'inline-flex items-center gap-1 text-ui hover:underline hover:decoration-border-strong hover:underline-offset-2',
		variant === 'standalone' && (quiet ? 'text-text-2 hover:text-text' : 'font-medium text-text'),
		variant === 'inline' && 'text-text-2 hover:text-text hover:underline hover:decoration-text-3 hover:underline-offset-2',
	]}
>
	{label}
	{#if variant === 'standalone'}<span class="inline-flex text-text-2"><Icon icon={ArrowUpRight} size={14} /></span>{/if}
</a>
