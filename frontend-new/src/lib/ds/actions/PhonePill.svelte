<script lang="ts">
	import type { Component } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Mobile / Pill (Inverse, Secondary, Ghost; Flow / Mobile Projects): a phone's button, 40 tall
	// and fully round, [0,16], its label at 14 / 500 with a 16 icon 6 apart. Inverse is the one that
	// needs the person (text fill), Secondary opens or resumes (border-strong), Ghost stops or cancels
	// (text-2, the icon first). A link where it goes somewhere.
	type Props = {
		label: string;
		variant?: 'inverse' | 'secondary' | 'ghost';
		icon?: Component<LucideProps>;
		href?: string;
		type?: 'button' | 'submit';
		loading?: boolean;
		onclick?: () => void;
	};
	let { label, variant = 'secondary', icon, href, type = 'button', loading = false, onclick }: Props = $props();
	const iconFirst = $derived(variant === 'ghost');
</script>

{#snippet inside()}
	{#if icon && iconFirst}<Icon {icon} size={16} />{/if}
	<span>{label}</span>
	{#if icon && !iconFirst}<Icon {icon} size={16} />{/if}
{/snippet}

{#if href}
	<a
		{href}
		class={[
			'inline-flex h-10 shrink-0 items-center justify-center gap-1.5 rounded-full px-4 text-phone-sm font-medium whitespace-nowrap outline-offset-2',
			variant === 'inverse' ? 'bg-text text-bg' : variant === 'secondary' ? 'border border-border-strong px-3.75 text-text' : 'text-text-2',
		]}>{@render inside()}</a
	>
{:else}
	<button
		{type}
		disabled={loading}
		aria-busy={loading || undefined}
		class={[
			'inline-flex h-10 shrink-0 cursor-pointer items-center justify-center gap-1.5 rounded-full px-4 text-phone-sm font-medium whitespace-nowrap outline-offset-2 disabled:cursor-default disabled:opacity-60',
			variant === 'inverse' ? 'bg-text text-bg' : variant === 'secondary' ? 'border border-border-strong px-3.75 text-text' : 'text-text-2',
		]}
		{onclick}>{@render inside()}</button
	>
{/if}
