<script lang="ts">
	// Flow / Mobile — You & Settings, step 16 (Memory & context) → Profile: what every crew member
	// reads about the person, as a card (surface, border, radius 14, 14 in, 10 apart): the text at
	// 14 (or what fills it, in text-3, when there's none), then when it was updated in mono and
	// Edit, a 44 target.
	type Props = { text: string; empty: string; updated?: string | null; edit: string; editLabel?: string; onedit: () => void };
	let { text, empty, updated, edit, editLabel, onedit }: Props = $props();
</script>

<div class="flex flex-col gap-2.5 rounded-xl bg-surface p-3.5 outline -outline-offset-1 outline-border">
	{#if text.trim()}
		<p class="text-phone-sm leading-firm whitespace-pre-line text-text">{text}</p>
	{:else}
		<p class="text-phone-sm leading-firm text-text-3">{empty}</p>
	{/if}
	<div class="-mb-2.5 flex items-center gap-3">
		<span class="min-w-0 flex-1 truncate font-mono text-label text-text-3">{updated ?? ''}</span>
		<button
			type="button"
			aria-label={editLabel}
			onclick={onedit}
			class="inline-flex h-11 shrink-0 cursor-pointer items-center text-phone-sm font-medium text-text outline-offset-1 focus-visible:outline-2 focus-visible:outline-focus">{edit}</button
		>
	</div>
</div>
