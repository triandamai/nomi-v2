<script lang="ts">
	import { CircleAlert } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Chat responses → Composer Limit: a message over the limit, by how much (in characters, as
	// counted), and the way round it.
	type Props = { count: string; max: string; onfile?: () => void };
	let { count, max, onfile }: Props = $props();
</script>

<p role="alert" class="flex w-full flex-wrap items-center gap-x-2 gap-y-1">
	<span class="inline-flex items-center gap-2 font-mono text-label text-danger"><Icon icon={CircleAlert} size={12} />{count} / {max}</span>
	<span class="min-w-0 flex-1 basis-40 text-caption text-text-2">{m.limit_too_long()}</span>
	{#if onfile}
		<button type="button" class="ml-auto cursor-pointer rounded-xs text-caption text-text outline-offset-1 hover:underline hover:decoration-border-strong hover:underline-offset-2 pointer-coarse:min-h-11" onclick={onfile}>{m.limit_send_file()}</button>
	{/if}
</p>
