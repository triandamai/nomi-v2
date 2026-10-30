<script lang="ts">
	import { CircleAlert, Hourglass, LoaderCircle } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// What the person said: on the right, in a surface-2 bubble. Until it's sent it's a little
	// faded and says where it is: sending, waiting in the outbox (offline), or not sent with a way
	// to retry or edit it.
	type Props = {
		text: string;
		status?: 'sent' | 'sending' | 'queued' | 'failed';
		oncancel?: () => void;
		onretry?: () => void;
		onedit?: () => void;
	};
	let { text, status = 'sent', oncancel, onretry, onedit }: Props = $props();
</script>

<div class="flex w-full flex-col items-end gap-1.5">
	<p
		class={[
			'max-w-xl rounded-md bg-surface-2 px-3.5 py-2 text-body whitespace-pre-wrap text-text',
			status !== 'sent' && 'opacity-85',
			status === 'failed' && 'border border-danger-border',
		]}
	>
		{text}
	</p>
	{#if status !== 'sent'}
		<p class="flex flex-wrap items-center justify-end gap-1.5">
			{#if status === 'sending'}
				<span class="inline-flex items-center gap-1.5 font-mono text-label text-text-3">
					<span class="inline-flex animate-spin motion-reduce:animate-none"><Icon icon={LoaderCircle} size={12} /></span>{m.msg_sending()}
				</span>
			{:else if status === 'queued'}
				<span class="inline-flex items-center gap-1.5 font-mono text-label text-text-3"><Icon icon={Hourglass} size={12} />{m.msg_queued()}</span>
				{#if oncancel}<button type="button" class="cursor-pointer rounded-xs text-caption text-text-2 outline-offset-1 hover:text-text pointer-coarse:min-h-11" onclick={oncancel}>{m.action_cancel()}</button>{/if}
			{:else}
				<span class="inline-flex items-center gap-1.5 font-mono text-label text-danger"><Icon icon={CircleAlert} size={12} />{m.msg_not_sent()}</span>
				{#if onretry}<button type="button" class="cursor-pointer rounded-xs text-caption text-text-2 outline-offset-1 hover:text-text pointer-coarse:min-h-11" onclick={onretry}>{m.action_retry()}</button>{/if}
				{#if onedit}<button type="button" class="cursor-pointer rounded-xs text-caption text-text-2 outline-offset-1 hover:text-text pointer-coarse:min-h-11" onclick={onedit}>{m.action_edit()}</button>{/if}
			{/if}
		</p>
	{/if}
</div>
