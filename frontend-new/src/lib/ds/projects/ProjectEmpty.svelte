<script lang="ts">
	import type { Component } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';

	// Design System / Projects → Project / Empty / No match (F4dhk9) and Empty / Tab (eI37L): the
	// grid when a search or a tab has nothing in it. 240 tall, 24 in, everything centred 10 apart: a
	// 20px icon, the title, a 300 wide line at 1.5 in text-3, and, when there's something to do
	// about it, one secondary button 6 further down.
	type Props = {
		icon: Component<LucideProps>;
		title: string;
		body: string;
		action?: { label: string; icon?: Component<LucideProps>; onclick: () => void };
	};
	let { icon, title, body, action }: Props = $props();
</script>

<div class="flex h-60 w-full flex-col items-center justify-center gap-2.5 rounded-md p-6 text-center outline -outline-offset-1 outline-border">
	<span class="inline-flex text-text-3"><Icon {icon} size={20} /></span>
	<p class="text-ui font-medium text-text">{title}</p>
	<p class="w-75 max-w-full text-ui leading-normal text-text-3">{body}</p>
	{#if action}
		<span class="inline-flex pt-1.5"><Button variant="secondary" label={action.label} iconEnd={action.icon} onclick={action.onclick} /></span>
	{/if}
</div>
