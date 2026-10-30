<script lang="ts">
	import { Check, Copy, Ellipsis, Share, ThumbsDown, ThumbsUp } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import { createCopier } from '../actions/copier.svelte';

	// Mobile / Action Bar: the reply's actions on 38pt round targets; the rest (regenerate,
	// delete) live in the sheet behind More.
	type Props = {
		text: string;
		feedback?: 'liked' | 'disliked' | null;
		onlike?: () => void;
		ondislike?: () => void;
		onshare?: () => void;
		onmore?: () => void;
	};
	let { text, feedback = null, onlike, ondislike, onshare, onmore }: Props = $props();

	const copier = createCopier();
</script>

<div role="group" aria-label={m.reply_actions()} class="flex items-center gap-0.5">
	<button type="button" aria-label={copier.copied ? m.copy_done() : m.reply_copy()} class="inline-flex size-9.5 cursor-pointer items-center justify-center rounded-full text-text-3 outline-offset-1" onclick={() => copier.copy(text)}>
		<Icon icon={copier.copied ? Check : Copy} size={18} />
	</button>
	{#if onlike}
		<button type="button" aria-label={m.reply_like()} aria-pressed={feedback === 'liked'} class={['inline-flex size-9.5 cursor-pointer items-center justify-center rounded-full outline-offset-1', feedback === 'liked' ? 'bg-surface-2 text-accent' : 'text-text-3']} onclick={onlike}>
			<Icon icon={ThumbsUp} size={18} />
		</button>
	{/if}
	{#if ondislike}
		<button type="button" aria-label={m.reply_dislike()} aria-pressed={feedback === 'disliked'} class={['inline-flex size-9.5 cursor-pointer items-center justify-center rounded-full outline-offset-1', feedback === 'disliked' ? 'bg-surface-2 text-text' : 'text-text-3']} onclick={ondislike}>
			<Icon icon={ThumbsDown} size={18} />
		</button>
	{/if}
	{#if onshare}
		<button type="button" aria-label={m.reply_share()} class="inline-flex size-9.5 cursor-pointer items-center justify-center rounded-full text-text-3 outline-offset-1" onclick={onshare}>
			<Icon icon={Share} size={18} />
		</button>
	{/if}
	{#if onmore}
		<button type="button" aria-label={m.mobile_more()} class="inline-flex size-9.5 cursor-pointer items-center justify-center rounded-full text-text-3 outline-offset-1" onclick={onmore}>
			<Icon icon={Ellipsis} size={18} />
		</button>
	{/if}
</div>
