<script lang="ts">
	import { CircleAlert } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Flow / Mobile Chat — Start & answer → your message (Answer yr0gh, nltkF): your message at 16pt
	// in an 18pt-round bubble on the right, its text at most 250 wide ([10,14], so the bubble at most
	// 278); if it wasn't sent, says so with a retry. The twin Mobile / User Message (`mobile/
	// UserMessage`, 280 wide: the old app's copy, which keeps its sizes) drawn to the chat's width.
	// `lifted` is the bubble held over a sheet's scrim (Message controls nzT7f): a border-strong
	// stroke and the lift shadow (0 6 18).
	type Props = { text: string; status?: 'sent' | 'failed'; onretry?: () => void; lifted?: boolean };
	let { text, status = 'sent', onretry, lifted = false }: Props = $props();
</script>

<div class="flex w-full flex-col items-end gap-1">
	<p class={['max-w-69.5 rounded-3xl bg-surface-2 px-3.5 py-2.5 text-phone leading-cozy whitespace-pre-wrap text-text', lifted && 'shadow-lift outline -outline-offset-1 outline-border-strong']}>{text}</p>
	{#if status === 'failed'}
		<p class="flex items-center gap-1.5 px-1">
			<span class="inline-flex items-center gap-1.5 font-mono text-label text-danger"><Icon icon={CircleAlert} size={12} />{m.msg_not_sent()}</span>
			{#if onretry}<button type="button" class="cursor-pointer rounded-xs text-ui font-medium text-text outline-offset-1 pointer-coarse:min-h-11" onclick={onretry}>{m.action_retry()}</button>{/if}
		</p>
	{/if}
</div>
