<script lang="ts">
	import type { Component } from 'svelte';
	import { LoaderCircle, type LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Design System / Nomi → Actions. One primary per view; Inverse for the main action inside a
	// panel, so mint stays rare. Loading shows real progress in its label ("Deploying · 00:14").
	// data-hover / data-focus / data-active show a state without a pointer (the gallery uses them).
	type Props = {
		label: string;
		variant?: 'primary' | 'inverse' | 'secondary' | 'ghost' | 'danger';
		iconStart?: Component<LucideProps>;
		iconEnd?: Component<LucideProps>;
		loading?: boolean;
		disabled?: boolean;
		/** Makes it a link. */
		href?: string;
		type?: 'button' | 'submit' | 'reset';
		onclick?: (event: MouseEvent) => void;
		[attribute: `data-${string}`]: unknown;
		[attribute: `aria-${string}`]: unknown;
	};
	let { label, variant = 'primary', iconStart, iconEnd, loading = false, disabled = false, href, type = 'button', onclick, ...rest }: Props = $props();

	const leading = $derived(loading ? LoaderCircle : iconStart);
	const lifts = $derived(variant === 'primary' || variant === 'inverse');
</script>

{#snippet content()}
	{#if leading}
		<span class={['inline-flex', variant === 'secondary' && 'text-text-2', variant === 'ghost' && 'text-text-3', loading && 'animate-spin motion-reduce:animate-none']}>
			<Icon icon={leading} size={14} />
		</span>
	{/if}
	<span>{label}</span>
	{#if iconEnd && !loading}
		<span class={['inline-flex', variant === 'secondary' && 'text-text-2', variant === 'ghost' && 'text-text-3']}>
			<Icon icon={iconEnd} size={14} />
		</span>
	{/if}
{/snippet}

{#if href}
	<a
		{href}
		{...rest}
		class={[
			'inline-flex h-7.5 shrink-0 cursor-pointer items-center justify-center gap-1.5 rounded-sm px-3 text-ui whitespace-nowrap outline-offset-1 transition-colors select-none pointer-coarse:min-h-11 data-focus:outline-2 data-focus:outline-focus',
			variant === 'primary' && 'bg-accent font-medium text-accent-ink',
			variant === 'inverse' && 'bg-text font-medium text-bg',
			variant === 'secondary' && 'border border-border-strong text-text hover:bg-surface-2 active:bg-border data-active:bg-border data-hover:bg-surface-2',
			variant === 'ghost' && 'text-text-2 hover:bg-surface-2 hover:text-text data-hover:bg-surface-2 data-hover:text-text',
			variant === 'danger' && 'border border-danger-border bg-danger-bg font-medium text-danger hover:border-danger data-hover:border-danger',
			lifts && 'lift',
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
			'inline-flex h-7.5 shrink-0 cursor-pointer items-center justify-center gap-1.5 rounded-sm px-3 text-ui whitespace-nowrap outline-offset-1 transition-colors select-none disabled:cursor-default pointer-coarse:min-h-11 data-focus:outline-2 data-focus:outline-focus',
			variant === 'primary' && 'bg-accent font-medium text-accent-ink',
			variant === 'inverse' && 'bg-text font-medium text-bg',
			variant === 'secondary' && 'border border-border-strong text-text hover:enabled:bg-surface-2 active:enabled:bg-border data-active:bg-border data-hover:bg-surface-2',
			variant === 'ghost' && 'text-text-2 hover:enabled:bg-surface-2 hover:enabled:text-text data-hover:bg-surface-2 data-hover:text-text',
			variant === 'danger' && 'border border-danger-border bg-danger-bg font-medium text-danger hover:enabled:border-danger data-hover:border-danger',
			lifts && 'lift',
			disabled && !loading && 'opacity-40',
		]}>{@render content()}</button
	>
{/if}

<style>
	/* Primary and Inverse lift on hover with a faint white edge and glow (pen: "Button / Primary /
	   Hover": stroke #FFFFFF33, inner shadow #FFFFFF26). It's a light, not a colour of its own. */
	.lift:not(:disabled):hover,
	.lift[data-hover] {
		box-shadow:
			inset 0 0 0 1px rgb(255 255 255 / 0.2),
			inset 0 0 8px rgb(255 255 255 / 0.15);
	}
</style>
