<script lang="ts">
	import { ArrowUp, Mic, Plus } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import IconButton from '../actions/IconButton.svelte';
	import KeyHint from '../actions/KeyHint.svelte';
	import Chip from './Chip.svelte';
	import type { CrewId } from '../agents/crew';

	// Design System / Nomi → Composer: one component for chat, home and project; the agent chip and
	// placeholder change. Enter sends and Shift+Enter starts a new line (never mid-composition in
	// an input method); Send stays off until there's something to send.
	type Props = {
		placeholder: string;
		agent: { id: CrewId; name: string };
		model: string;
		value?: string;
		hints?: { keys: string; action: string }[];
		onsend?: (text: string) => void;
		onattach?: () => void;
		onrecord?: () => void;
		onpickagent?: () => void;
		onpickmodel?: () => void;
	};
	let {
		placeholder,
		agent,
		model,
		value = $bindable(''),
		hints = [
			{ keys: '↵', action: m.composer_hint_send() },
			{ keys: '⇧↵', action: m.composer_hint_newline() },
			{ keys: '@', action: m.composer_hint_mention() },
		],
		onsend,
		onattach,
		onrecord,
		onpickagent,
		onpickmodel,
	}: Props = $props();

	const canSend = $derived(value.trim().length > 0);

	function send() {
		if (!canSend) return;
		onsend?.(value.trim());
		value = '';
	}

	function onkeydown(event: KeyboardEvent) {
		if (event.key !== 'Enter' || event.shiftKey || event.isComposing) return;
		event.preventDefault();
		send();
	}

	/** The field grows with what's written, up to a point, then scrolls. */
	function grow(field: HTMLTextAreaElement) {
		$effect(() => {
			void value;
			field.style.height = 'auto';
			field.style.height = `${Math.min(field.scrollHeight, 240)}px`;
		});
	}
</script>

<div data-composer class="flex w-full flex-col gap-3 rounded-md border border-border-strong bg-surface px-2.75 pt-3.25 pb-2.25">
	<textarea
		{placeholder}
		aria-label={m.composer_field()}
		rows="1"
		bind:value
		{onkeydown}
		use:grow
		class="w-full resize-none bg-transparent px-1 text-body text-text outline-none placeholder:text-text-3"
	></textarea>
	<div class="flex items-center gap-1.5">
		<IconButton icon={Plus} label={m.composer_attach()} onclick={onattach} />
		<Chip kind="agent" label={agent.name} agent={agent.id} onclick={onpickagent} />
		<Chip kind="model" label={model} onclick={onpickmodel} />
		<span class="flex-1"></span>
		<span class="hidden md:block"><KeyHint {hints} /></span>
		<IconButton icon={Mic} label={m.composer_record()} variant="ghost" onclick={onrecord} />
		<IconButton icon={ArrowUp} label={m.composer_send()} variant="send" disabled={!canSend} onclick={send} />
	</div>
</div>
