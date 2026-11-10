<script lang="ts">
	import type { Component } from 'svelte';
	import { LoaderCircle, type LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Auth / Sign in or create → Continue: the form's one action, 44 high across the column, accent
	// with the label (13, 500) and its arrow. Secondary is the bordered surface the Google button
	// uses, for a second way on (an invite's "I have an account"). Loading keeps the label and adds
	// the spinner for this step only; it can't be pressed twice.
	type Props = {
		label: string;
		variant?: 'primary' | 'secondary';
		iconEnd?: Component<LucideProps>;
		loading?: boolean;
		disabled?: boolean;
		/** Makes it a link. */
		href?: string;
		type?: 'submit' | 'button';
		onclick?: (event: MouseEvent) => void;
		[attribute: `data-${string}`]: unknown;
	};
	let { label, variant = 'primary', iconEnd, loading = false, disabled = false, href, type = 'submit', onclick, ...rest }: Props = $props();
</script>

{#snippet content()}
	{#if loading}<span class="inline-flex animate-spin motion-reduce:animate-none"><Icon icon={LoaderCircle} size={14} /></span>{/if}
	<span>{label}</span>
	{#if iconEnd && !loading}<span class="inline-flex"><Icon icon={iconEnd} size={14} /></span>{/if}
{/snippet}

{#if href}
	<a
		{href}
		{...rest}
		class={[
			'flex h-11 w-full cursor-pointer items-center justify-center gap-2 rounded-sm text-ui font-medium outline-offset-0 transition-colors select-none focus-visible:outline-3',
			variant === 'primary' ? 'lift bg-accent text-accent-ink' : 'border border-border-strong bg-surface text-text hover:bg-surface-2 focus-visible:border-transparent',
		]}>{@render content()}</a
	>
{:else}
	<button
		{type}
		{onclick}
		disabled={disabled || loading}
		aria-busy={loading || undefined}
		{...rest}
		class={[
			'flex h-11 w-full cursor-pointer items-center justify-center gap-2 rounded-sm text-ui font-medium outline-offset-0 transition-colors select-none focus-visible:outline-3 disabled:cursor-default',
			variant === 'primary' ? 'lift bg-accent text-accent-ink' : 'border border-border-strong bg-surface text-text hover:enabled:bg-surface-2 focus-visible:border-transparent',
			disabled && !loading && 'opacity-40',
		]}>{@render content()}</button
	>
{/if}

<style>
	/* The primary lifts on hover like Button / Primary / Hover: a faint white edge and glow. */
	.lift:not(:disabled):hover,
	.lift[data-hover] {
		box-shadow:
			inset 0 0 0 1px rgb(255 255 255 / 0.2),
			inset 0 0 8px rgb(255 255 255 / 0.15);
	}
</style>
