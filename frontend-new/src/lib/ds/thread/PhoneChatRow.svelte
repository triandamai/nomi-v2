<script lang="ts">
	import { MessageSquare } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// Flow / Mobile Chat — Start & answer → Chats → Chat: a chat in the phone's list, [12,20], 12
	// apart: a 40 round avatar, the title at 16 (500) with its time in mono, the last line at 14.
	// The pen's avatar is who last answered, in their tint; where that isn't known (the list doesn't
	// say: BACKEND_TODO.md) it's the chat glyph on surface-2. Mobile / Chat Row is the twin this
	// follows; this one takes no agent for granted, says when the crew is at work, and long-pressing
	// it (or its More, for the keyboard) opens its sheet.
	type Props = {
		href: string;
		title: string;
		time?: string;
		preview?: string;
		agent?: CrewId;
		/** "working" in place of the time while the crew is at it. */
		working?: string;
		/** The label of More, the way to the chat's sheet without a long press. */
		more?: string;
		onmore?: () => void;
	};
	let { href, title, time, preview, agent, working, more, onmore }: Props = $props();

	let timer: ReturnType<typeof setTimeout> | undefined;
	let pressed = false;
	function press() {
		pressed = false;
		timer = setTimeout(() => {
			pressed = true;
			onmore?.();
		}, 500);
	}
	const release = () => clearTimeout(timer);
</script>

<div class="relative">
	<a
		{href}
		class="flex w-full items-center gap-3 px-5 py-3 -outline-offset-2 touch-callout-none select-none"
		onpointerdown={press}
		onpointerup={release}
		onpointerleave={release}
		onpointercancel={release}
		oncontextmenu={(event) => {
			event.preventDefault();
			if (!pressed) onmore?.();
		}}
		onclick={(event) => {
			// A long press opened the sheet: it's not also a tap on the chat.
			if (pressed) event.preventDefault();
			pressed = false;
		}}
	>
		{#if agent}
			<span class="inline-flex size-10 shrink-0 items-center justify-center rounded-full" style="background: var(--crew-{agent}-bg)"><Mascot {agent} size={32} /></span>
		{:else}
			<span class="inline-flex size-10 shrink-0 items-center justify-center rounded-full bg-surface-2 text-text-2"><Icon icon={MessageSquare} size={18} /></span>
		{/if}
		<span class="flex min-w-0 flex-1 flex-col gap-0.75">
			<span class="flex items-center gap-2">
				<span class="min-w-0 flex-1 truncate text-phone font-medium text-text">{title}</span>
				{#if working}<span class="font-mono text-label text-accent">{working}</span>{:else if time}<span class="font-mono text-label text-text-3">{time}</span>{/if}
			</span>
			{#if preview}<span class="line-clamp-1 text-phone-sm text-text-3">{preview}</span>{/if}
		</span>
	</a>
	{#if onmore && more}
		<button type="button" class="sr-only focus:not-sr-only focus:absolute focus:top-3 focus:right-3 focus:rounded-sm focus:bg-surface-2 focus:px-2 focus:py-1 focus:text-caption" onclick={onmore}>{more}</button>
	{/if}
</div>
