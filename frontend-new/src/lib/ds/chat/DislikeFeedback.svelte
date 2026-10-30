<script lang="ts">
	import { Check, X } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import CheckMark from '../inputs/CheckMark.svelte';

	// Chat responses → Feedback / Dislike: what went wrong first (any number of reasons), then an
	// optional note. Only this reply goes with it unless you include the whole chat, and amounts
	// and names stay masked either way.
	export type DislikeReason = 'inaccurate' | 'ignored' | 'wrong_crew' | 'too_long' | 'outdated' | 'unsafe' | 'other';
	type Props = {
		onsend: (feedback: { reasons: DislikeReason[]; note: string; includeChat: boolean }) => void;
		oncancel: () => void;
	};
	let { onsend, oncancel }: Props = $props();

	const id = $props.id();
	const REASONS: { key: DislikeReason; label: () => string }[] = [
		{ key: 'inaccurate', label: m.dislike_inaccurate },
		{ key: 'ignored', label: m.dislike_ignored },
		{ key: 'wrong_crew', label: m.dislike_wrong_crew },
		{ key: 'too_long', label: m.dislike_too_long },
		{ key: 'outdated', label: m.dislike_outdated },
		{ key: 'unsafe', label: m.dislike_unsafe },
		{ key: 'other', label: m.dislike_other },
	];

	let reasons = $state<DislikeReason[]>([]);
	let note = $state('');
	let includeChat = $state(false);

	function toggle(key: DislikeReason) {
		reasons = reasons.includes(key) ? reasons.filter((reason) => reason !== key) : REASONS.map((reason) => reason.key).filter((at) => at === key || reasons.includes(at));
	}
</script>

<section aria-labelledby="{id}-title" class="flex w-full flex-col overflow-hidden rounded-md bg-surface outline -outline-offset-1 outline-border-strong">
	<header class="flex items-center gap-2.5 px-3.5 pt-3 pb-1">
		<p id="{id}-title" class="min-w-0 flex-1 text-ui font-medium text-text">{m.dislike_title()}</p>
		<button type="button" aria-label={m.action_close()} class="inline-flex size-6 cursor-pointer items-center justify-center rounded-xs text-text-3 outline-offset-1 hover:text-text pointer-coarse:size-11" onclick={oncancel}>
			<Icon icon={X} size={14} />
		</button>
	</header>
	<div class="flex flex-wrap gap-2 px-3.5 py-2">
		{#each REASONS as reason (reason.key)}
			{@const on = reasons.includes(reason.key)}
			<button
				type="button"
				aria-pressed={on}
				class={[
					'inline-flex h-7 cursor-pointer items-center gap-1.5 rounded-full border px-3 text-ui outline-offset-1 transition-colors pointer-coarse:min-h-11',
					on ? 'border-text bg-text text-bg' : 'border-border-strong text-text-2 hover:text-text',
				]}
				onclick={() => toggle(reason.key)}
			>
				{#if on}<Icon icon={Check} size={12} />{/if}{reason.label()}
			</button>
		{/each}
	</div>
	<div class="px-3.5 pt-1 pb-2.5">
		<textarea
			aria-label={m.dislike_title()}
			bind:value={note}
			class="h-18 w-full resize-none rounded-sm border border-border-strong bg-bg px-2.5 py-2 text-caption leading-normal text-text placeholder:text-text-3"
		></textarea>
	</div>
	<div class="flex gap-2 px-3.5 pb-3">
		<span class="pt-px"><CheckMark id="{id}-chat" bind:checked={includeChat} aria-describedby="{id}-chat-help" /></span>
		<span class="flex flex-col gap-0.5">
			<label for="{id}-chat" class="cursor-pointer text-caption text-text">{m.dislike_include_chat()}</label>
			<span id="{id}-chat-help" class="text-caption text-text-3">{m.dislike_include_chat_help()}</span>
		</span>
	</div>
	<footer class="flex items-center justify-end gap-2 border-t border-border px-3.5 pt-2.25 pb-2.5">
		<Button label={m.action_cancel()} variant="ghost" onclick={oncancel} />
		<Button
			label={m.dislike_send()}
			disabled={reasons.length === 0 && !note.trim()}
			onclick={() => onsend({ reasons: [...reasons], note: note.trim(), includeChat })}
		/>
	</footer>
</section>
