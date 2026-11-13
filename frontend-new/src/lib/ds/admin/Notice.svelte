<script lang="ts">
	import type { Component } from 'svelte';
	import { Check, CircleAlert, Info, TriangleAlert, type LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Admin → Notice: what a form or a test just found, said where it happened (under the form, not
	// in a toast that goes away): saved, failed with the backend's own reason quoted in mono, a test
	// that passed, something to know first. An icon and words, never colour alone; a failure
	// interrupts (alert), the rest wait their turn (status). Built like Banner.
	type Props = { kind: 'success' | 'error' | 'info' | 'warning'; text: string; detail?: string };
	let { kind, text, detail }: Props = $props();

	const ICONS: Record<Props['kind'], Component<LucideProps>> = { success: Check, error: CircleAlert, info: Info, warning: TriangleAlert };
</script>

<div
	role={kind === 'error' ? 'alert' : 'status'}
	class={[
		'flex w-full items-start gap-2.5 rounded-sm px-3 py-2',
		kind === 'success' && 'bg-accent-bg',
		kind === 'error' && 'bg-danger-bg',
		kind === 'warning' && 'bg-warning-bg',
		kind === 'info' && 'bg-surface outline -outline-offset-1 outline-border',
	]}
>
	<span
		class={[
			'inline-flex shrink-0 pt-px',
			kind === 'success' && 'text-accent',
			kind === 'error' && 'text-danger',
			kind === 'warning' && 'text-warning',
			kind === 'info' && 'text-text-2',
		]}><Icon icon={ICONS[kind]} size={14} /></span
	>
	<span class="flex min-w-0 flex-1 flex-col gap-0.5">
		<span class="text-caption text-text">{text}</span>
		{#if detail}<span class="font-mono text-label break-words whitespace-pre-line text-text-2">{detail}</span>{/if}
	</span>
</div>
