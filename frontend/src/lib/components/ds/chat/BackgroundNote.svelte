<script lang="ts">
	import { LoaderCircle } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Chat responses → Background Note: long work runs without you watching; the result comes back
	// here and to Inbox. The estimate is the crew's own, marked as rough (~).
	let { eta }: { eta?: string } = $props();
</script>

<p role="status" class="flex w-full items-center gap-2.5 rounded-sm bg-surface px-3 py-2">
	<span class="inline-flex shrink-0 animate-spin text-text-2 motion-reduce:animate-none"><Icon icon={LoaderCircle} size={14} /></span>
	<span class="min-w-0 flex-1 text-caption text-text-2">{m.background_note()}</span>
	{#if eta}<span class="font-mono text-label text-text-3">{eta}</span>{/if}
</p>
