<script lang="ts">
	import { onMount } from 'svelte';
	import { ArrowUp } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import PhonePill from '../actions/PhonePill.svelte';

	// Flow / Mobile Chat — Message controls → Editing (step 4): your message, edited where it stood,
	// in a box on the surface with 18 corners and an accent stroke ([12,12,12,14]), the text at 16;
	// under it which version it saves as (mono) and Cancel (ghost) and Send (accent) as pills. Enter
	// sends, Shift+Enter makes a new line, Escape cancels.
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

<div class="flex w-full flex-col gap-2.5 rounded-3xl bg-surface px-3.5 py-3 outline -outline-offset-1 outline-accent">
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
		<PhonePill variant="ghost" label={m.action_cancel()} onclick={oncancel} />
		<button type="button" disabled={!ready} class="inline-flex h-10 shrink-0 cursor-pointer items-center gap-1.5 rounded-full bg-accent px-4 text-phone-sm font-medium text-accent-ink outline-offset-2 disabled:cursor-default disabled:opacity-50" onclick={save}>
			{m.composer_send()}<ArrowUp size={16} strokeWidth={1.5} aria-hidden="true" />
		</button>
	</div>
</div>
