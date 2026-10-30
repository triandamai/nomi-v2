<script lang="ts">
	import { m } from '$lib/paraglide/messages';

	// Chat responses → Rename Field: the chat's name edited in place in the top bar. Enter saves
	// (not when empty), Escape cancels.
	type Props = { value: string; onsave: (name: string) => void; oncancel: () => void };
	let { value, onsave, oncancel }: Props = $props();

	const initial = () => value;
	let name = $state(initial());

	function focus(field: HTMLInputElement) {
		field.focus();
		field.select();
	}

	function onkeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' && name.trim()) {
			event.preventDefault();
			onsave(name.trim());
		} else if (event.key === 'Escape') {
			event.preventDefault();
			oncancel();
		}
	}
</script>

<span class="flex min-w-0 items-center gap-2.5">
	<input
		use:focus
		type="text"
		aria-label={m.rename_label()}
		bind:value={name}
		{onkeydown}
		class="h-7 w-75 max-w-full min-w-0 rounded-sm border border-accent bg-surface px-2 text-ui text-text outline-none"
	/>
	<span class="shrink-0 font-mono text-label whitespace-nowrap text-text-3">{m.rename_hint()}</span>
</span>
