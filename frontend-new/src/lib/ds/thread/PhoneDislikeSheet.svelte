<script lang="ts">
	import { Check } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Toggle from '../inputs/Toggle.svelte';
	import BottomSheet from '../overlays/BottomSheet.svelte';
	import type { DislikeReason } from '../chat/DislikeFeedback.svelte';

	// Flow / Mobile Chat — Like & dislike → Dislike sheet (step 3, D4O2g): what went wrong as round
	// chips at 15 (any number; a chosen one filled with text and checked, the others text-2), the
	// body 14 apart, a note at 16 on bg in a 12-round box 84 tall, 12 in, "Include the whole chat"
	// at 15 (off: only this reply and the note, masked), and Send feedback across the width in accent.
	// The pen's six reasons (ZxLos): the desktop's DislikeFeedback also offers "Unsafe or harmful",
	// which the phone's sheet doesn't draw (the backend takes any of them).
	type Props = { open?: boolean; onsend: (feedback: { reasons: DislikeReason[]; note: string; includeChat: boolean }) => void; onclose?: () => void };
	let { open = $bindable(false), onsend, onclose }: Props = $props();

	const REASONS: { key: DislikeReason; label: () => string }[] = [
		{ key: 'inaccurate', label: m.dislike_inaccurate },
		{ key: 'wrong_crew', label: m.dislike_wrong_crew },
		{ key: 'ignored', label: m.dislike_ignored },
		{ key: 'too_long', label: m.dislike_too_long },
		{ key: 'outdated', label: m.dislike_outdated },
		{ key: 'other', label: m.dislike_other },
	];
	let reasons = $state<DislikeReason[]>([]);
	let note = $state('');
	let includeChat = $state(false);
	$effect(() => {
		if (!open) {
			reasons = [];
			note = '';
			includeChat = false;
		}
	});
	const toggle = (key: DislikeReason) => (reasons = reasons.includes(key) ? reasons.filter((reason) => reason !== key) : [...reasons, key]);
</script>

<BottomSheet bind:open title={m.dislike_title()} subtitle={m.dislike_pick_any()} {onclose}>
	<div class="flex flex-col gap-3.5 px-4 pb-2">
		<div role="group" aria-label={m.dislike_title()} class="flex flex-wrap gap-2">
			{#each REASONS as reason (reason.key)}
				{@const chosen = reasons.includes(reason.key)}
				<button
					type="button"
					aria-pressed={chosen}
					class={['inline-flex h-9 cursor-pointer items-center gap-1.5 rounded-full px-3.5 text-body outline-offset-2', chosen ? 'bg-text text-bg' : 'border border-border-strong px-3.25 text-text-2']}
					onclick={() => toggle(reason.key)}
				>
					{#if chosen}<Icon icon={Check} size={14} />{/if}{reason.label()}
				</button>
			{/each}
		</div>
		<textarea
			bind:value={note}
			rows="3"
			aria-label={m.dislike_note_label()}
			placeholder={m.dislike_note_placeholder()}
			class="h-21 w-full resize-none rounded-lg border border-border-strong bg-bg p-2.75 text-phone text-text outline-none placeholder:text-text-3 focus:border-accent"
		></textarea>
		<div class="flex items-center gap-3">
			<span class="flex min-w-0 flex-1 flex-col gap-0.5">
				<span class="text-body text-text">{m.dislike_include_chat()}</span>
				<span class="text-ui text-text-3">{m.dislike_include_chat_phone()}</span>
			</span>
			<Toggle label={m.dislike_include_chat()} bind:checked={includeChat} />
		</div>
		<button
			type="button"
			class="inline-flex h-11.5 w-full cursor-pointer items-center justify-center rounded-lg bg-accent text-phone font-medium text-accent-ink outline-offset-2"
			onclick={() => {
				onsend({ reasons, note: note.trim(), includeChat });
				open = false;
			}}>{m.dislike_send()}</button
		>
	</div>
</BottomSheet>
