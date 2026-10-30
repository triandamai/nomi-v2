<script lang="ts">
	import { ArrowRight, RotateCcw, Square, Unplug } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';

	// Chat responses → Streaming, Stopped, Interrupted: a reply's prose and whether it's still
	// coming (a mint caret), was stopped by you (and after how long), or was cut off (the reason,
	// in the provider's terms, and what to do next).
	type Props = {
		text: string;
		status?: 'done' | 'streaming' | 'stopped' | 'interrupted';
		/** How long it ran before you stopped it, e.g. "00:03". */
		elapsed?: string;
		/** Why it was cut off, e.g. "Reply cut off after 18s. DeepSeek closed the stream." */
		reason?: string;
		oncontinue?: () => void;
		onrestart?: () => void;
	};
	let { text, status = 'done', elapsed = '', reason = '', oncontinue, onrestart }: Props = $props();
</script>

<div class="flex w-full flex-col gap-2" aria-busy={status === 'streaming'}>
	<p class="text-body whitespace-pre-wrap text-text-2">
		{text}{#if status === 'streaming'}<span data-caret aria-hidden="true" class="ml-0.5 inline-block h-4 w-2 animate-pulse bg-accent align-middle motion-reduce:animate-none"></span><span class="sr-only">{m.reply_writing()}</span>{/if}
	</p>
	{#if status === 'stopped'}
		<p class="flex flex-wrap items-center gap-2">
			<span class="inline-flex text-text-3"><Icon icon={Square} size={12} /></span>
			<span class="font-mono text-label text-text-3">{m.reply_stopped({ elapsed })}</span>
			{#if oncontinue}<button type="button" class="cursor-pointer rounded-xs text-caption text-text-2 outline-offset-1 hover:text-text pointer-coarse:min-h-11" onclick={oncontinue}>{m.reply_continue()}</button>{/if}
		</p>
	{:else if status === 'interrupted'}
		<div class="flex flex-wrap items-center gap-2.5 rounded-sm bg-warning-bg py-2 pr-2 pl-3">
			<span class="inline-flex text-warning"><Icon icon={Unplug} size={14} /></span>
			<span class="min-w-0 flex-1 basis-48 text-caption text-text">{reason}</span>
			<span class="ml-auto flex items-center gap-2">
				{#if onrestart}<Button label={m.reply_start_over()} variant="ghost" iconStart={RotateCcw} onclick={onrestart} />{/if}
				{#if oncontinue}<Button label={m.reply_continue()} variant="secondary" iconEnd={ArrowRight} onclick={oncontinue} />{/if}
			</span>
		</div>
	{/if}
</div>
