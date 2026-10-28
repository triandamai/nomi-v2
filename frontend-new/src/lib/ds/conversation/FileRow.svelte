<script lang="ts">
	import { FileCode } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	/** A file an agent touched, with its change ("+214"), in mono. */
	let { path, change }: { path: string; change?: string } = $props();
</script>

<span class="flex w-full items-center gap-2 px-3 py-1.5">
	<span class="inline-flex text-text-3"><Icon icon={FileCode} size={12} /></span>
	<span class="min-w-0 flex-1 truncate font-mono text-label text-text-2">{path}</span>
	{#if change}<span class="font-mono text-label text-text-3">{change}</span>{/if}
</span>
