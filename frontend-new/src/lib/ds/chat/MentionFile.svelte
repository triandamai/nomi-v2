<script lang="ts">
	import type { Component } from 'svelte';
	import { File, type LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Chat responses → Mention / File: a file named in a message, in mono, neutral (files aren't crew).
	let { name, icon = File }: { name: string; icon?: Component<LucideProps> } = $props();
</script>

<span class="inline-flex items-center gap-1 rounded-xs bg-surface-2 px-1.5 py-px">
	<span class="inline-flex text-text-2"><Icon {icon} size={12} /></span>
	<span class="font-mono text-caption text-text">{name}</span>
</span>
