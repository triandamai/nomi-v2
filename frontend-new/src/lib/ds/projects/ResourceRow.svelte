<script lang="ts">
	import type { Component } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Flow / Build — Workspace tools (U16wU) → Screen / Monitor (EQNtT) → KV Row: one resource of the
	// running app, [7,0] and 8 apart: its icon, its name (72 wide) and the value in mono.
	type Props = { icon: Component<LucideProps>; name: string; value: string };
	let { icon, name, value }: Props = $props();
</script>

<div class="flex items-center gap-2 py-1.75">
	<span class="inline-flex shrink-0 text-text-3"><Icon {icon} size={14} /></span>
	<span class="w-18 shrink-0 text-ui text-text-2">{name}</span>
	<span class="min-w-0 flex-1 truncate font-mono text-label text-text">{value}</span>
</div>
