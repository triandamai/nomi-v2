<script lang="ts">
	import type { Component } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	/** A plain key with a machine value: "Code   23 files · main@a3f9c21". */
	let { icon, label, value }: { icon?: Component<LucideProps>; label: string; value: string } = $props();
</script>

<div class="flex w-full items-center gap-2 py-1.75">
	{#if icon}<span class="inline-flex text-text-3"><Icon {icon} size={14} /></span>{/if}
	<span class="w-18 shrink-0 text-ui text-text-2">{label}</span>
	<span class="min-w-0 flex-1 truncate font-mono text-label text-text">{value}</span>
</div>
