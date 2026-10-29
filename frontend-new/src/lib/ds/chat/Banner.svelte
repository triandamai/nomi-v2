<script lang="ts">
	import type { Component } from 'svelte';
	import { Activity, Gauge, KeyRound, Wifi, WifiOff, type LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Chat responses → Banners: above the thread, one at a time, about what's around the chat (the
	// connection, the provider, your key, your plan) rather than one message. Offline is a warning;
	// back online is good news; the rest sit on a quiet bordered surface.
	type Kind = 'offline' | 'online' | 'slow' | 'expiring' | 'quota';
	type Props = {
		kind: Kind;
		text: string;
		action?: { label: string; onclick?: () => void; href?: string; external?: boolean };
	};
	let { kind, text, action }: Props = $props();

	const ICONS: Record<Kind, Component<LucideProps>> = { offline: WifiOff, online: Wifi, slow: Activity, expiring: KeyRound, quota: Gauge };
</script>

<div
	role="status"
	class={[
		'flex w-full flex-wrap items-center gap-x-2.5 gap-y-1 rounded-sm px-3 py-2',
		kind === 'offline' && 'bg-warning-bg',
		kind === 'online' && 'bg-accent-bg',
		kind !== 'offline' && kind !== 'online' && 'border border-border bg-surface',
	]}
>
	<span class={['inline-flex shrink-0', kind === 'online' ? 'text-accent' : kind === 'quota' ? 'text-text-2' : 'text-warning']}><Icon icon={ICONS[kind]} size={14} /></span>
	<span class="min-w-0 flex-1 basis-48 text-caption text-text">{text}</span>
	{#if action?.href}
		<a
			href={action.href}
			target={action.external ? '_blank' : undefined}
			rel={action.external ? 'noopener noreferrer' : undefined}
			class="ml-auto rounded-xs text-caption text-text-2 outline-offset-1 hover:text-text hover:underline hover:decoration-border-strong hover:underline-offset-2">{action.label}</a
		>
	{:else if action}
		<button type="button" class="ml-auto cursor-pointer rounded-xs text-caption text-text-2 outline-offset-1 hover:text-text pointer-coarse:min-h-11" onclick={action.onclick}>{action.label}</button>
	{/if}
</div>
