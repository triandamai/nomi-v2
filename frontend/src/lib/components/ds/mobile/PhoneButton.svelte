<script lang="ts">
	import type { Component } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Mobile / Button: full width, 46pt tall, 16pt label, 12pt corners; primary (mint), secondary
	// (outlined) or danger. The icon follows the label.
	type Props = {
		label: string;
		variant?: 'primary' | 'secondary' | 'danger';
		icon?: Component<LucideProps>;
		href?: string;
		external?: boolean;
		disabled?: boolean;
		onclick?: () => void;
	};
	let { label, variant = 'primary', icon, href, external = false, disabled = false, onclick }: Props = $props();
</script>

{#snippet content()}
	{label}{#if icon}<Icon {icon} />{/if}
{/snippet}

{#if href}
	<a
		{href}
		target={external ? '_blank' : undefined}
		rel={external ? 'noopener noreferrer' : undefined}
		class={[
			'flex h-11.5 w-full items-center justify-center gap-2 rounded-lg text-phone font-medium outline-offset-2',
			variant === 'primary' && 'bg-accent text-accent-ink',
			variant === 'secondary' && 'border border-border-strong text-text',
			variant === 'danger' && 'border border-danger-border bg-danger-bg text-danger',
		]}>{@render content()}</a
	>
{:else}
	<button
		type="button"
		{disabled}
		{onclick}
		class={[
			'flex h-11.5 w-full cursor-pointer items-center justify-center gap-2 rounded-lg text-phone font-medium outline-offset-2 disabled:cursor-default disabled:opacity-40',
			variant === 'primary' && 'bg-accent text-accent-ink',
			variant === 'secondary' && 'border border-border-strong text-text',
			variant === 'danger' && 'border border-danger-border bg-danger-bg text-danger',
		]}>{@render content()}</button
	>
{/if}
