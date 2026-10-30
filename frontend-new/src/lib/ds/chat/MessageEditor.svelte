<script lang="ts">
	import { ArrowUp } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Button from '../actions/Button.svelte';

	// Chat responses → User / Editing: a sent message edited in place. Saving sends it as a new
	// version; the old one (and its reply) stays, reachable from the version switcher. Escape
	// cancels; ⌘/Ctrl+Enter saves.
	type Props = {
		value: string;
		/** Where the old reply stays, e.g. "1 / 2". */
		version?: string;
		onsave: (text: string) => void;
		oncancel: () => void;
	};
	let { value, version, onsave, oncancel }: Props = $props();

	const initial = () => value;
	let text = $state(initial());
	const changed = $derived(text.trim() !== '' && text.trim() !== value.trim());

	function focus(field: HTMLTextAreaElement) {
		field.focus();
		field.setSelectionRange(field.value.length, field.value.length);
	}

	function onkeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			event.preventDefault();
			oncancel();
		} else if (event.key === 'Enter' && (event.metaKey || event.ctrlKey) && changed) {
			event.preventDefault();
			onsave(text.trim());
		}
	}
</script>

<div class="flex w-full justify-end">
	<div class="flex w-140 max-w-full flex-col gap-3 rounded-md bg-surface p-3 outline -outline-offset-1 outline-accent">
		<textarea
			use:focus
			bind:value={text}
			{onkeydown}
			aria-label={m.msg_edit()}
			rows="1"
			class="field-sizing-content w-full resize-none bg-transparent text-body leading-normal text-text outline-none"
		></textarea>
		<div class="flex flex-wrap items-center gap-2">
			<span class="min-w-0 flex-1 basis-48 font-mono text-label text-text-3">{version ? m.edit_hint({ version }) : m.edit_hint_new()}</span>
			<span class="ml-auto flex items-center gap-2">
				<Button label={m.action_cancel()} variant="ghost" onclick={oncancel} />
				<Button label={m.edit_save()} iconEnd={ArrowUp} disabled={!changed} onclick={() => onsave(text.trim())} />
			</span>
		</div>
	</div>
</div>
