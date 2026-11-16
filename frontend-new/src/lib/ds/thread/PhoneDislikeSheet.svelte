<script lang="ts">
	import { Check } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Toggle from '../inputs/Toggle.svelte';
	import BottomSheet from '../overlays/BottomSheet.svelte';
	import type { DislikeReason } from '../chat/DislikeFeedback.svelte';

	// Flow / Mobile Chat — Like & dislike → Dislike sheet (step 3): what went wrong as round chips
	// (any number; a chosen one filled with text and checked), a note at 16 in a 14-round box,
	// "Include the whole chat" (off: only this reply and the note, masked), and Send feedback across
	// the width in accent. The same reasons and words as the desktop's DislikeFeedback.
	type Props = { open?: boolean; onsend: (feedback: { reasons: DislikeReason[]; note: string; includeChat: boolean }) => void; onclose?: () => void };
	let { open = $bindable(false), onsend, onclose }: Props = $props();

	const REASONS: { key: DislikeReason; label: () => string }[] = [
		{ key: 'inaccurate', label: m.dislike_inaccurate },
		{ key: 'wrong_crew', label: m.dislike_wrong_crew },
		{ key: 'ignored', label: m.dislike_ignored },
		{ key: 'too_long', label: m.dislike_too_long },
		{ key: 'outdated', label: m.dislike_outdated },
		{ key: 'unsafe', label: m.dislike_unsafe },
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
	<div class="flex flex-col gap-4 px-4 pb-2">
		<div role="group" aria-label={m.dislike_title()} class="flex flex-wrap gap-2">
			{#each REASONS as reason (reason.key)}
				{@const chosen = reasons.includes(reason.key)}
				<button
					type="button"
					aria-pressed={chosen}
					class={['inline-flex h-9 cursor-pointer items-center gap-1.5 rounded-full px-3.5 text-phone-sm outline-offset-2', chosen ? 'bg-text text-bg' : 'border border-border-strong px-3.25 text-text']}
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
			class="w-full resize-none rounded-xl border border-border-strong bg-surface px-3.25 py-2.75 text-phone text-text outline-none placeholder:text-text-3 focus:border-accent"
		></textarea>
		<div class="flex items-center gap-3">
			<span class="flex min-w-0 flex-1 flex-col gap-0.5">
				<span class="text-phone-sm text-text">{m.dislike_include_chat()}</span>
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
