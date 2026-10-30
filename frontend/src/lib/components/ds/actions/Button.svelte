<script lang="ts">
	import type { Component } from 'svelte';
	import { LoaderCircle, type LucideProps } from '@lucide/svelte';
	import M3Button from '../../m3/Button.svelte';
	import Icon from '../foundations/Icon.svelte';

	// Twin of frontend-new's Button on the M3 button: primary is the hero (action gradient, one
	// per view), inverse a filled button for a panel's main action, secondary outlined, ghost a text
	// button; the danger pair in the error role. Loading shows real progress in its label.
	type Props = {
		label: string;
		/** ghost-danger: a quiet stop ("Stop all" in the crew-working banner). */
		variant?: 'primary' | 'inverse' | 'secondary' | 'ghost' | 'ghost-danger' | 'danger';
		iconStart?: Component<LucideProps>;
		iconEnd?: Component<LucideProps>;
		loading?: boolean;
		disabled?: boolean;
		/** Makes it a link. */
		href?: string;
		/** A link to another site: opens in a new tab, without access to this page. */
		external?: boolean;
		type?: 'button' | 'submit' | 'reset';
		onclick?: (event: MouseEvent) => void;
		[attribute: `data-${string}`]: unknown;
		[attribute: `aria-${string}`]: unknown;
	};
	let { label, variant = 'primary', iconStart, iconEnd, loading = false, disabled = false, href, external = false, type = 'button', onclick, ...rest }: Props = $props();

	const M3_VARIANT = {
		primary: 'gradient',
		inverse: 'filled',
		secondary: 'outlined',
		ghost: 'text',
		'ghost-danger': 'text',
		danger: 'outlined',
	} as const;
	const leading = $derived(loading ? LoaderCircle : iconStart);
	const danger = $derived(variant === 'danger' || variant === 'ghost-danger');
</script>

{#snippet content()}
	{#if leading}<span class="icon" class:icon--spin={loading}><Icon icon={leading} size={16} /></span>{/if}
	<span>{label}</span>
	{#if iconEnd && !loading}<span class="icon"><Icon icon={iconEnd} size={16} /></span>{/if}
{/snippet}

{#if href}
	<M3Button
		{href}
		variant={M3_VARIANT[variant]}
		target={external ? '_blank' : undefined}
		rel={external ? 'noopener noreferrer' : undefined}
		class={danger ? 'ds-button--danger' : ''}
		{...rest}>{@render content()}</M3Button
	>
{:else}
	<M3Button
		{type}
		{onclick}
		variant={M3_VARIANT[variant]}
		disabled={disabled || loading}
		aria-busy={loading || undefined}
		class={danger ? 'ds-button--danger' : ''}
		{...rest}>{@render content()}</M3Button
	>
{/if}

<style>
	.icon {
		display: inline-flex;
	}
	.icon--spin {
		animation: spin 1s linear infinite;
	}
	@media (prefers-reduced-motion: reduce) {
		.icon--spin {
			animation: none;
		}
	}
	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
	:global(.ds-button--danger) {
		color: var(--md-sys-color-error) !important;
	}
	:global(.ds-button--danger.m3-button--outlined) {
		border-color: var(--md-sys-color-error) !important;
	}
</style>
