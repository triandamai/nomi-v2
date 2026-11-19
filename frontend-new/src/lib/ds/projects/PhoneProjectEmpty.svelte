<script lang="ts">
	import type { Component } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import PhonePill from '../actions/PhonePill.svelte';

	// Mobile / Projects Empty / No match (M9JDSg), Empty / Tab (uDnnd) and Mobile / Projects Load
	// Error (UULGe), Flow / Mobile Projects — Find & filter, Loading, offline & errors: the phone's
	// list when there's nothing to show. No box: [40,24], everything centred 10 apart. A 48 tile with
	// 14 corners (surface-2, its 22 glyph in text-2; danger-bg and danger when the list failed), the
	// title at 17 / 600, the line at 14 / 1.45 in text-3, what failed in mono at 12 where there's a
	// reason, and the one thing to do as a 40 pill (border-strong, 14 / 500, its icon after).
	type Props = {
		icon: Component<LucideProps>;
		title: string;
		body: string;
		/** What failed and when, in mono: "503 from Nomi’s server · 09:14". */
		code?: string;
		tone?: 'plain' | 'danger';
		action?: { label: string; icon?: Component<LucideProps>; onclick: () => void };
	};
	let { icon, title, body, code, tone = 'plain', action }: Props = $props();
</script>

<div role={tone === 'danger' ? 'alert' : undefined} class="flex w-full flex-col items-center gap-2.5 px-6 py-10 text-center">
	<span class={['inline-flex size-12 items-center justify-center rounded-xl', tone === 'danger' ? 'bg-danger-bg text-danger' : 'bg-surface-2 text-text-2']}><Icon {icon} size={22} /></span>
	<p class="w-full text-phone-title font-semibold text-text">{title}</p>
	<p class="w-full text-phone-sm leading-comfy text-text-3">{body}</p>
	{#if code}<p class="font-mono text-caption text-text-3">{code}</p>{/if}
	{#if action}<PhonePill label={action.label} icon={action.icon} onclick={action.onclick} />{/if}
</div>
