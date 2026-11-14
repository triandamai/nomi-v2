<script lang="ts">
	import Dialog from '../overlays/Dialog.svelte';
	import Button from '../actions/Button.svelte';

	// Admin → Confirm: asks before something that can't be undone (deleting a plan, revoking a key,
	// terminating an app). The title names the thing; the body says what happens next, plainly; the
	// confirm button says the action ("Delete plan") in the danger style, and Cancel is the quiet way
	// out. Escape cancels. While the change is on its way the button shows it and can't be pressed
	// twice. Built on Dialog.
	type Props = {
		open?: boolean;
		title: string;
		body: string;
		confirmLabel: string;
		cancelLabel: string;
		busy?: boolean;
		onconfirm: () => void;
		/** Shows the box in place (the docs). */
		inline?: boolean;
	};
	let { open = $bindable(false), title, body, confirmLabel, cancelLabel, busy = false, onconfirm, inline = false }: Props = $props();
</script>

<Dialog bind:open {inline} {title} description={body}>
	{#snippet actions()}
		<Button label={cancelLabel} variant="ghost" onclick={() => (open = false)} />
		<Button label={confirmLabel} variant="danger" loading={busy} onclick={onconfirm} />
	{/snippet}
</Dialog>
