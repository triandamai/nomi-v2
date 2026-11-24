<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { ArrowUp } from '@lucide/svelte';
	import type { CrewId } from '../agents/crew';
	import Mascot from '../agents/Mascot.svelte';
	import Icon from '../foundations/Icon.svelte';

	// Flow / Mobile — Home, Inbox & You, step 1 (dl13j) → Ask (UIQTM): Home's way to start a chat on a
	// phone. A card (surface, border-strong, radius 18, inset 14, 12 apart): the field at 16pt, then who
	// it goes to (a 32 pill with their mascot), the model in mono, and Send (36, round), the accent once
	// there's something to send. Each control keeps a 44 touch area drawn outside it, so the card stays the
	// pen's 98 high.
	type Props = {
		value?: string;
		field?: HTMLTextAreaElement;
		placeholder: string;
		to: { agent: CrewId; name: string };
		model?: string;
		busy?: boolean;
		onsend: (text: string) => void;
		onto?: () => void;
		onmodel?: () => void;
	};
	let { value = $bindable(''), field = $bindable(), placeholder, to, model, busy = false, onsend, onto, onmodel }: Props = $props();
	const ready = $derived(value.trim().length > 0 && !busy);

	function send() {
		if (ready) onsend(value.trim());
	}
</script>

<div class="flex flex-col gap-3 rounded-3xl bg-surface p-3.5 outline -outline-offset-1 outline-border-strong">
	<textarea
		bind:this={field}
		bind:value
		{placeholder}
		aria-label={placeholder}
		rows="1"
		class="max-h-40 w-full resize-none bg-transparent field-sizing-content text-phone leading-cozy text-text outline-none placeholder:text-text-3"
		onkeydown={(event) => {
			if (event.key === 'Enter' && !event.shiftKey && !event.isComposing) {
				event.preventDefault();
				send();
			}
		}}
	></textarea>
	<div class="flex items-center gap-2">
		<button type="button" aria-label={m.phone_home_to({ name: to.name })} class="inline-flex h-8 shrink-0 cursor-pointer items-center gap-1.5 rounded-full bg-surface-2 px-2.5 text-phone-sm text-text outline-offset-2 focus-visible:outline-2 focus-visible:outline-focus relative before:absolute before:-inset-y-1.5 before:inset-x-0" onclick={onto}>
			<Mascot agent={to.agent} size={20} />
			{to.name}
		</button>
		{#if model}
			<button type="button" class="min-w-0 flex-1 cursor-pointer truncate text-left font-mono text-label text-text-3 outline-offset-2 focus-visible:outline-2 focus-visible:outline-focus relative py-1 before:absolute before:-inset-y-2.5 before:inset-x-0" onclick={onmodel}>{model}</button>
		{:else}
			<span class="flex-1"></span>
		{/if}
		<button type="button" aria-label={m.composer_send()} disabled={!ready} class={['inline-flex size-9 shrink-0 cursor-pointer items-center justify-center rounded-full outline-offset-2 focus-visible:outline-2 focus-visible:outline-focus disabled:cursor-default relative before:absolute before:-inset-1', ready ? 'bg-accent text-accent-ink' : 'bg-surface-2 text-text-3']} onclick={send}>
			<Icon icon={ArrowUp} size={18} />
		</button>
	</div>
</div>
