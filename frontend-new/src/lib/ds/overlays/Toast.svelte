<script lang="ts">
	import type { Component } from 'svelte';
	import { Check, CircleAlert, Info, X, type LucideProps } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Design System / Nomi → Overlays: what just happened, its detail in mono, one action and a
	// close. Errors interrupt (role alert); everything else waits its turn (role status).
	type Props = {
		kind: 'success' | 'error' | 'info';
		title: string;
		detail?: string;
		icon?: Component<LucideProps>;
		action?: { label: string; onclick: () => void };
		ondismiss?: () => void;
	};
	let { kind, title, detail, icon, action, ondismiss }: Props = $props();

	const glyph = $derived(icon ?? (kind === 'success' ? Check : kind === 'error' ? CircleAlert : Info));
</script>

<div
	role={kind === 'error' ? 'alert' : 'status'}
	class="flex w-100 max-w-full items-start gap-3 rounded-md border border-border-strong bg-surface px-3.5 py-3 text-text shadow-float"
>
	<span class={['inline-flex pt-px', kind === 'success' && 'text-accent', kind === 'error' && 'text-danger', kind === 'info' && 'text-text-2']}><Icon icon={glyph} /></span>
	<span class="flex min-w-0 flex-1 flex-col gap-0.5">
		<span class="text-ui font-medium text-text">{title}</span>
		{#if detail}<span class="font-mono text-label text-text-3">{detail}</span>{/if}
	</span>
	{#if action}
		<button type="button" onclick={action.onclick} class="cursor-pointer rounded-xs text-ui text-text-2 outline-offset-2 hover:text-text">{action.label}</button>
	{/if}
	{#if ondismiss}
		<button type="button" aria-label={m.toast_dismiss()} onclick={ondismiss} class="inline-flex cursor-pointer rounded-xs p-0.5 text-text-3 outline-offset-2 hover:text-text">
			<Icon icon={X} size={14} />
		</button>
	{/if}
</div>
