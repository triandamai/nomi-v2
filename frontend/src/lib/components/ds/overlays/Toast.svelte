<script lang="ts">
	import type { Component } from 'svelte';
	import { Check, CircleAlert, Info, X, type LucideProps } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Twin of frontend-new's Toast as an M3 snackbar: what just happened, its detail in mono, one
	// action and a close. Errors interrupt (role alert); everything else waits its turn (role status).
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

<div role={kind === 'error' ? 'alert' : 'status'} class="toast toast--{kind}">
	<span class="icon"><Icon icon={glyph} size={20} /></span>
	<span class="text">
		<span class="title">{title}</span>
		{#if detail}<span class="detail">{detail}</span>{/if}
	</span>
	{#if action}
		<button type="button" onclick={action.onclick} class="action">{action.label}</button>
	{/if}
	{#if ondismiss}
		<button type="button" aria-label={m.toast_dismiss()} onclick={ondismiss} class="close">
			<Icon icon={X} size={20} />
		</button>
	{/if}
</div>

<style>
	/* The M3 snackbar: inverse surface, one action in inverse-primary, a close. */
	.toast {
		display: flex;
		width: 400px;
		max-width: 100%;
		min-height: 48px;
		align-items: center;
		gap: 12px;
		padding: 8px 8px 8px 16px;
		border-radius: var(--md-sys-shape-corner-extra-small);
		background: var(--md-sys-color-inverse-surface);
		color: var(--md-sys-color-inverse-on-surface);
		box-shadow: var(--md-sys-elevation-shadow-level3);
	}
	.icon {
		display: inline-flex;
	}
	.toast--success .icon {
		color: var(--md-sys-color-inverse-primary);
	}
	.toast--error .icon {
		color: var(--md-sys-color-error-container);
	}
	.text {
		display: flex;
		min-width: 0;
		flex: 1;
		flex-direction: column;
		gap: 2px;
		padding: 6px 0;
	}
	.title {
		font: var(--md-sys-typescale-body-medium-weight) var(--md-sys-typescale-body-medium-size) / var(--md-sys-typescale-body-medium-line-height) var(--md-sys-typescale-body-medium-font);
	}
	.detail {
		font-family: var(--md-ref-typeface-mono);
		font-size: var(--md-sys-typescale-label-small-size);
		opacity: 0.8;
	}
	.action,
	.close {
		display: inline-flex;
		min-height: 40px;
		align-items: center;
		border: 0;
		border-radius: var(--md-sys-shape-corner-full);
		background: none;
		cursor: pointer;
		color: var(--md-sys-color-inverse-primary);
	}
	.action {
		padding: 0 12px;
		font: var(--md-sys-typescale-label-large-weight) var(--md-sys-typescale-label-large-size) / var(--md-sys-typescale-label-large-line-height) var(--md-sys-typescale-label-large-font);
	}
	.close {
		width: 40px;
		justify-content: center;
		color: var(--md-sys-color-inverse-on-surface);
	}
	.action:hover,
	.close:hover {
		background: color-mix(in srgb, currentColor 8%, transparent);
	}
	.action:focus-visible,
	.close:focus-visible {
		outline: 2px solid var(--md-sys-color-inverse-primary);
	}
</style>
