<script lang="ts">
	import type { Component } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Flow / Mobile Chat — Clarify → plan (step 6 "Plan ready · nothing runs yet", step 9 "Plan v2
	// ready", and while questions wait) and Plan → approve → act (step 1 "3 steps · Maya, Dana,
	// Kala"): the bar that sits over the composer while something waits on the person. Surface, a
	// top stroke, [10,16], what waits at 14 in text-2, and the one thing to do as a mint pill 36
	// tall (15 / 500, an icon before it where the pen draws one). The pill keeps the pen's 36 and
	// takes a 44 target.
	type Props = { label: string; action: string; icon?: Component<LucideProps>; onclick: () => void; button?: HTMLButtonElement };
	let { label, action, icon, onclick, button = $bindable() }: Props = $props();
</script>

<div data-flow-bar class="flex w-full items-center gap-2 border-t border-border bg-surface px-4 pt-2.25 pb-2.5">
	<p class="min-w-0 flex-1 truncate text-phone-sm text-text-2">{label}</p>
	<button bind:this={button} type="button" class="relative inline-flex h-9 shrink-0 cursor-pointer items-center gap-1.5 rounded-full bg-accent px-3.5 text-body font-medium text-accent-ink outline-offset-2" {onclick}>
		<span aria-hidden="true" class="absolute -inset-y-1 inset-x-0"></span>
		{#if icon}<Icon {icon} size={14} />{/if}
		{action}
	</button>
</div>
