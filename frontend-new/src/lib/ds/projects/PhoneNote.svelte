<script lang="ts">
	import type { Component } from 'svelte';
	import { Lock, type LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Flow / Mobile Build → a sheet's note (Database jyEKQ "Read-only here…", Code C0kOpT "Read-only…"):
	// a 14 icon and the line at 13, on surface-2 in a 12-round box [10,12] (`boxed`), or plain under a list.
	type Props = { text: string; icon?: Component<LucideProps>; boxed?: boolean };
	let { text, icon = Lock, boxed = false }: Props = $props();
</script>

<p class={['flex items-center gap-2 text-ui', boxed ? 'rounded-lg bg-surface-2 px-3 py-2.5 text-text-2' : 'text-text-3']}>
	<span class="inline-flex shrink-0"><Icon {icon} size={14} /></span>{text}
</p>
