<script lang="ts">
	import { Globe } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import Status from '../status/Status.svelte';

	// Flow / Build — Workspace tools (U16wU) → Screen / Domain (J2hPJ) → Addr: one address of the
	// app, [12,14] and 10 apart, outlined (border-strong where `strong`): the globe, the address in
	// mono (linked when it's served), and its state on the right: Live with its dot, or what it's
	// waiting for in the warning colour.
	type Props = { address: string; href?: string; state?: { label: string; tone: 'live' | 'waiting' }; strong?: boolean };
	let { address, href, state: shown, strong = false }: Props = $props();
</script>

<div data-part="address" class={['flex items-center gap-2.5 rounded-sm px-3.5 py-3 outline -outline-offset-1', strong ? 'outline-border-strong' : 'outline-border']}>
	<span class="inline-flex shrink-0 text-text-2"><Icon icon={Globe} size={14} /></span>
	{#if href}
		<a {href} target="_blank" rel="noopener" class="min-w-0 flex-1 truncate rounded-xs font-mono text-ui text-text outline-offset-2 hover:underline">{address}</a>
	{:else}
		<span class="min-w-0 flex-1 truncate font-mono text-ui text-text">{address}</span>
	{/if}
	{#if shown?.tone === 'live'}
		<Status kind="live" label={shown.label} small />
	{:else if shown}
		<span class="shrink-0 text-label text-warning">{shown.label}</span>
	{/if}
</div>
