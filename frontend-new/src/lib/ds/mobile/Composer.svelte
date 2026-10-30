<script lang="ts">
	import type { Snippet } from 'svelte';
	import { ArrowUp, Mic, Plus, Square } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Mobile / Composer: above the keyboard. Attach on the left, the message in a round pill at
	// 16pt (so iOS doesn't zoom), the mic inside it, and send only once there's text; while a reply
	// streams, stop takes send's place. Attachments, a notice and a toolbar can sit around it.
	type Props = {
		value?: string;
		streaming?: boolean;
		/** e.g. "transcribed from 0:14 of audio". */
		notice?: string;
		onsend: (text: string) => void;
		onattach?: () => void;
		onrecord?: () => void;
		onstop?: () => void;
		attachments?: Snippet;
		toolbar?: Snippet;
	};
	let { value = $bindable(''), streaming = false, notice, onsend, onattach, onrecord, onstop, attachments, toolbar }: Props = $props();

	const ready = $derived(value.trim() !== '');

	function send() {
		if (!ready) return;
		onsend(value.trim());
		value = '';
	}
</script>

<div class="flex w-full flex-col gap-2 border-t border-border bg-bg px-3 pt-1.75 pb-2">
	{#if attachments}<div class="flex items-center gap-2 overflow-x-auto">{@render attachments()}</div>{/if}
	{#if notice}<p class="flex items-center gap-1.5 px-1 font-mono text-label text-text-3"><Icon icon={Mic} size={12} />{notice}</p>{/if}
	<div class="flex items-end gap-2">
		{#if onattach}
			<button type="button" aria-label={m.composer_attach()} class="inline-flex size-9.5 shrink-0 cursor-pointer items-center justify-center rounded-full bg-surface-2 text-text-2 outline-offset-1" onclick={onattach}>
				<Icon icon={Plus} size={20} />
			</button>
		{/if}
		<div class="flex min-w-0 flex-1 items-end gap-2 rounded-full border border-border-strong bg-surface py-2 pr-2.25 pl-3.25 focus-within:outline-2 focus-within:outline-offset-2 focus-within:outline-focus">
			<textarea
				bind:value
				rows="1"
				aria-label={m.composer_field()}
				placeholder={m.mobile_message()}
				onkeydown={(event) => {
					if (event.key === 'Enter' && !event.shiftKey && !event.isComposing) {
						event.preventDefault();
						send();
					}
				}}
				class="field-sizing-content max-h-32 min-w-0 flex-1 resize-none bg-transparent text-phone leading-tight text-text outline-none placeholder:text-text-3"
			></textarea>
			{#if onrecord}
				<button type="button" aria-label={m.composer_record()} class="inline-flex shrink-0 cursor-pointer text-text-2 outline-offset-2" onclick={onrecord}><Icon icon={Mic} size={20} /></button>
			{/if}
		</div>
		{#if streaming && onstop}
			<button type="button" aria-label={m.reply_stop()} class="inline-flex size-9.5 shrink-0 cursor-pointer items-center justify-center rounded-full bg-text text-bg outline-offset-1" onclick={onstop}>
				<Icon icon={Square} size={14} />
			</button>
		{:else if ready}
			<button type="button" aria-label={m.composer_send()} class="inline-flex size-9.5 shrink-0 cursor-pointer items-center justify-center rounded-full bg-accent text-accent-ink outline-offset-1" onclick={send}>
				<Icon icon={ArrowUp} size={20} />
			</button>
		{/if}
	</div>
	{#if toolbar}<div class="flex items-center gap-2 px-0.5">{@render toolbar()}</div>{/if}
</div>
