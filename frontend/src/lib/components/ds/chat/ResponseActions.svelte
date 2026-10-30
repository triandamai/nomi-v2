<script lang="ts">
	import { Check, Copy, RefreshCw, Share2, ThumbsDown, ThumbsUp, Trash2 } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import { createCopier } from '../actions/copier.svelte';

	// Chat responses → Response actions: the same quiet bar under every reply. Like and dislike
	// are pressed toggles (a like in mint); delete sits apart. What answered, how long it took and
	// how many tokens it cost stay visible on the right.
	type Props = {
		/** The reply as plain text, for Copy. */
		text: string;
		/** e.g. "deepseek-flash · 1.4s · 812 tokens". */
		meta?: string;
		feedback?: 'liked' | 'disliked' | null;
		onlike?: () => void;
		ondislike?: () => void;
		onshare?: () => void;
		onregenerate?: () => void;
		ondelete?: () => void;
	};
	let { text, meta, feedback = null, onlike, ondislike, onshare, onregenerate, ondelete }: Props = $props();

	const copier = createCopier();
</script>

<div role="group" aria-label={m.reply_actions()} class="flex w-full flex-wrap items-center gap-0.5">
	<button type="button" aria-label={copier.copied ? m.copy_done() : m.reply_copy()} class="inline-flex size-7 cursor-pointer items-center justify-center rounded-sm text-text-3 outline-offset-1 hover:bg-surface-2 hover:text-text pointer-coarse:size-11" onclick={() => copier.copy(text)}>
		<Icon icon={copier.copied ? Check : Copy} size={14} />
	</button>
	{#if onlike}
		<button
			type="button"
			aria-label={m.reply_like()}
			aria-pressed={feedback === 'liked'}
			class={['inline-flex size-7 cursor-pointer items-center justify-center rounded-sm outline-offset-1 hover:bg-surface-2 pointer-coarse:size-11', feedback === 'liked' ? 'bg-surface-2 text-accent' : 'text-text-3 hover:text-text']}
			onclick={onlike}><Icon icon={ThumbsUp} size={14} /></button
		>
	{/if}
	{#if ondislike}
		<button
			type="button"
			aria-label={m.reply_dislike()}
			aria-pressed={feedback === 'disliked'}
			class={['inline-flex size-7 cursor-pointer items-center justify-center rounded-sm outline-offset-1 hover:bg-surface-2 pointer-coarse:size-11', feedback === 'disliked' ? 'bg-surface-2 text-text' : 'text-text-3 hover:text-text']}
			onclick={ondislike}><Icon icon={ThumbsDown} size={14} /></button
		>
	{/if}
	{#if onshare}
		<button type="button" aria-label={m.reply_share()} class="inline-flex size-7 cursor-pointer items-center justify-center rounded-sm text-text-3 outline-offset-1 hover:bg-surface-2 hover:text-text pointer-coarse:size-11" onclick={onshare}>
			<Icon icon={Share2} size={14} />
		</button>
	{/if}
	{#if onregenerate}
		<button type="button" aria-label={m.reply_regenerate()} class="inline-flex size-7 cursor-pointer items-center justify-center rounded-sm text-text-3 outline-offset-1 hover:bg-surface-2 hover:text-text pointer-coarse:size-11" onclick={onregenerate}>
			<Icon icon={RefreshCw} size={14} />
		</button>
	{/if}
	{#if ondelete}
		<span aria-hidden="true" class="mx-0.5 h-3.5 w-px bg-border-strong"></span>
		<button type="button" aria-label={m.reply_delete()} class="inline-flex size-7 cursor-pointer items-center justify-center rounded-sm text-text-3 outline-offset-1 hover:bg-surface-2 hover:text-text pointer-coarse:size-11" onclick={ondelete}>
			<Icon icon={Trash2} size={14} />
		</button>
	{/if}
	{#if meta}<span class="ml-auto pl-3 font-mono text-label text-text-3">{meta}</span>{/if}
</div>
