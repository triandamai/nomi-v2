<script lang="ts">
	import { onMount } from 'svelte';
	import { ArrowUp } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';

	// Flow / Mobile Chat — Message controls → Editing (step 4, lnxaH): your message, edited where it
	// stood, in a box on the surface with 16 corners and an accent stroke ([12,12,12,14]), the text at
	// 16; under it which version it saves as (mono) and Cancel and Send as 34-tall pills, 14 in
	// (t80BL2, uEd5M): Cancel in a border-strong stroke at 15 / 400 in text, Send in accent at 15 / 500
	// with a 14 arrow. Enter sends, Shift+Enter makes a new line, Escape cancels.
	type Props = { value: string; version: string; onsave: (text: string) => void; oncancel: () => void };
	let { value, version, onsave, oncancel }: Props = $props();
	const initial = () => value;
	let draft = $state(initial());
	let field: HTMLTextAreaElement | undefined = $state();
	const ready = $derived(draft.trim() !== '' && draft.trim() !== value.trim());
	onMount(() => {
		field?.focus();
		field?.setSelectionRange(draft.length, draft.length);
	});
	function save() {
		if (ready) onsave(draft.trim());
	}
</script>

<div data-editor class="flex w-full flex-col gap-2.5 rounded-2xl bg-surface px-3.5 py-3 outline -outline-offset-1 outline-accent">
	<textarea
		bind:this={field}
		bind:value={draft}
		aria-label={m.message_edit_field()}
		rows="2"
		onkeydown={(event) => {
			if (event.key === 'Escape') oncancel();
			else if (event.key === 'Enter' && !event.shiftKey && !event.isComposing) {
				event.preventDefault();
				save();
			}
		}}
		class="field-sizing-content max-h-48 w-full resize-none bg-transparent text-phone text-text outline-none"
	></textarea>
	<div class="flex items-center gap-2">
		<span class="min-w-0 flex-1 truncate font-mono text-label text-text-3">{version}</span>
		<button type="button" class="inline-flex h-8.5 shrink-0 cursor-pointer items-center rounded-full border border-border-strong px-3.25 text-body text-text outline-offset-2" onclick={oncancel}>{m.action_cancel()}</button>
		<button type="button" disabled={!ready} class="inline-flex h-8.5 shrink-0 cursor-pointer items-center gap-1.5 rounded-full bg-accent px-3.5 text-body font-medium text-accent-ink outline-offset-2 disabled:cursor-default disabled:opacity-50" onclick={save}>
			{m.composer_send()}<ArrowUp size={14} strokeWidth={1.5} aria-hidden="true" />
		</button>
	</div>
</div>
