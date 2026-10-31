<script lang="ts">
	import type { Component } from 'svelte';
	import { CircleAlert, type LucideProps } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import PhoneButton from './PhoneButton.svelte';

	// Mobile / Error Card: the desktop callout made for a phone. The provider's own words quoted in
	// mono, the model, key and request underneath, and the fixes as full-width buttons.
	type Action = { label: string; icon?: Component<LucideProps>; onclick?: () => void; href?: string; external?: boolean };
	type Props = { title: string; code?: string; provider: string; reason: string; meta?: string; primary?: Action; secondary?: Action };
	let { title, code, provider, reason, meta, primary, secondary }: Props = $props();
</script>

<section role="alert" class="flex w-full flex-col overflow-hidden rounded-md bg-bg outline -outline-offset-1 outline-border-strong">
	<header class="flex items-center gap-2 border-b border-border bg-danger-bg px-3.5 pt-2.5 pb-2.25">
		<span class="inline-flex text-danger"><Icon icon={CircleAlert} /></span>
		<span class="min-w-0 flex-1 text-body font-semibold text-danger">{title}</span>
		{#if code}<span class="font-mono text-label text-text-3">{code}</span>{/if}
	</header>
	<div class="flex flex-col gap-3 p-3.5">
		<blockquote class="flex flex-col gap-1 border-l-2 border-border-strong pl-2.5">
			<span class="font-mono text-label tracking-label text-text-3 uppercase">{m.mobile_said({ provider })}</span>
			<span class="font-mono text-phone-sm text-text">{reason}</span>
		</blockquote>
		{#if meta}<p class="font-mono text-label break-words text-text-3">{meta}</p>{/if}
		{#if primary || secondary}
			<div class="flex flex-col gap-2">
				{#if primary}<PhoneButton {...primary} />{/if}
				{#if secondary}<PhoneButton variant="secondary" {...secondary} />{/if}
			</div>
		{/if}
	</div>
</section>
