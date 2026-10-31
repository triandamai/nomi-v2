<script lang="ts">
	import { AlarmClock, Bell, BellRing, Check, Pencil } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import IconButton from '../actions/IconButton.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// Chat responses → Reminder: Kala's (the tile wears Kala's tint). Scheduled, it says when and who
	// keeps it, with a way to edit; due, it turns to warning and offers snooze and done.
	type Props = {
		title: string;
		/** e.g. "25 Nov · 09:00 · every month". */
		when: string;
		due?: boolean;
		owner?: { agent: CrewId; name: string };
		/** What "done" means for this one, e.g. "Mark paid". */
		doneLabel?: string;
		onedit?: () => void;
		onsnooze?: () => void;
		ondone?: () => void;
	};
	let { title, when, due = false, owner, doneLabel, onedit, onsnooze, ondone }: Props = $props();
</script>

<section class={['flex w-full flex-wrap items-center gap-3 rounded-md bg-bg px-3.5 py-3 outline -outline-offset-1', due ? 'outline-warning' : 'outline-border-strong']}>
	<span class="inline-flex size-9 shrink-0 items-center justify-center rounded-sm bg-crew-kala-bg text-crew-kala-ink"><Icon icon={due ? BellRing : Bell} size={16} /></span>
	<span class="flex min-w-0 flex-1 basis-40 flex-col gap-0.75">
		<span class="text-ui font-medium text-text">{title}</span>
		<span class={['font-mono text-label', due ? 'text-warning' : 'text-text-3']}>{due ? `${m.reminder_due()} · ${when}` : when}</span>
	</span>
	<span class="flex items-center gap-2">
		{#if due}
			{#if onsnooze}<Button label={m.reminder_snooze()} variant="ghost" iconStart={AlarmClock} onclick={onsnooze} />{/if}
			{#if ondone}<Button label={doneLabel ?? m.reminder_done()} variant="secondary" iconEnd={Check} onclick={ondone} />{/if}
		{:else}
			{#if owner}
				<span class="inline-flex items-center gap-1.5">
					<Mascot agent={owner.agent} size={20} />
					<span class="text-caption text-text-3">{owner.name}</span>
				</span>
			{/if}
			{#if onedit}<IconButton icon={Pencil} label={m.reminder_edit()} variant="ghost" iconSize={14} onclick={onedit} />{/if}
		{/if}
	</span>
</section>
