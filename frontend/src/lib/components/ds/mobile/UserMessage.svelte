<script lang="ts">
	import { CircleAlert } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Mobile / User Message: your message at 16pt in an 18pt-round bubble on the right; if it
	// wasn't sent, says so with a retry.
	type Props = { text: string; status?: 'sent' | 'failed'; onretry?: () => void };
	let { text, status = 'sent', onretry }: Props = $props();
</script>

<div class="flex w-full flex-col items-end gap-1">
	<p class="max-w-70 rounded-3xl bg-surface-2 px-3.5 py-2.5 text-phone leading-cozy whitespace-pre-wrap text-text">{text}</p>
	{#if status === 'failed'}
		<p class="flex items-center gap-1.5 px-1">
			<span class="inline-flex items-center gap-1.5 font-mono text-label text-danger"><Icon icon={CircleAlert} size={12} />{m.msg_not_sent()}</span>
			{#if onretry}<button type="button" class="cursor-pointer rounded-xs text-ui font-medium text-text outline-offset-1 pointer-coarse:min-h-11" onclick={onretry}>{m.action_retry()}</button>{/if}
		</p>
	{/if}
</div>
