<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Check } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Button from '../actions/Button.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// Chat responses → Form: appears inline, prefilled from the chat, as a real form. Validation
	// sits under each field (the fields own it), never in a toast.
	type Props = {
		agent: CrewId;
		title: string;
		/** e.g. "prefilled from chat". */
		meta?: string;
		/** What happens after saving, e.g. "Kala pings you in Inbox and on your phone". */
		note?: string;
		submitLabel: string;
		onsubmit: () => void;
		oncancel?: () => void;
		children: Snippet;
	};
	let { agent, title, meta, note, submitLabel, onsubmit, oncancel, children }: Props = $props();
</script>

<form
	class="flex w-full flex-col overflow-hidden rounded-md bg-bg outline -outline-offset-1 outline-border-strong"
	onsubmit={(event) => {
		event.preventDefault();
		onsubmit();
	}}
>
	<header class="flex items-center gap-2.5 border-b border-border bg-surface px-3.5 pt-2.5 pb-2.25">
		<Mascot {agent} size={20} />
		<p class="min-w-0 flex-1 text-ui font-medium text-text">{title}</p>
		{#if meta}<span class="font-mono text-label text-text-3">{meta}</span>{/if}
	</header>
	<div class="flex flex-col gap-3.5 p-3.5">{@render children()}</div>
	<footer class="flex flex-wrap items-center gap-2 border-t border-border px-3.5 pt-2.25 pb-2.5">
		<span class="min-w-0 flex-1 basis-48 text-caption text-text-3">{note}</span>
		<span class="ml-auto flex items-center gap-2">
			{#if oncancel}<Button label={m.action_cancel()} variant="ghost" onclick={oncancel} />{/if}
			<Button label={submitLabel} type="submit" iconEnd={Check} />
		</span>
	</footer>
</form>
