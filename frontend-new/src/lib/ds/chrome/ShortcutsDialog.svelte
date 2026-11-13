<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import Button from '../actions/Button.svelte';
	import Kbd from '../actions/Kbd.svelte';
	import Dialog from '../overlays/Dialog.svelte';

	// The account menu's "Keyboard shortcuts" (App / Account menu, LkJLt): the keys the app really
	// answers to, each beside what it does, in a Dialog. The pen draws the menu row, not the dialog,
	// so this is the Dialog with a list in it. ⌘ is written as the design writes it; the
	// description tells Windows and Linux to press Ctrl.
	type Shortcut = { label: string; keys: string };
	type Props = { open?: boolean; shortcuts: Shortcut[]; inline?: boolean };
	let { open = $bindable(false), shortcuts, inline = false }: Props = $props();
</script>

<Dialog bind:open {inline} title={m.account_shortcuts()} description={m.shortcuts_body()}>
	<dl class="flex flex-col">
		{#each shortcuts as shortcut (shortcut.label)}
			<div class="flex h-9 items-center justify-between gap-3 border-b border-border last:border-b-0">
				<dt class="text-ui text-text">{shortcut.label}</dt>
				<dd><Kbd keys={shortcut.keys} /></dd>
			</div>
		{/each}
	</dl>
	{#snippet actions()}
		<Button label={m.shortcuts_done()} variant="inverse" onclick={() => (open = false)} />
	{/snippet}
</Dialog>
