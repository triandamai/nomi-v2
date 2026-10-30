<script lang="ts">
	import { Trash2 } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Button from '../actions/Button.svelte';
	import Dialog from '../overlays/Dialog.svelte';

	// Chat responses → Delete Dialog: what leaves (the reply and its cards), what stays (your
	// message) and what can't be undone (things the crew already did).
	type Props = { open?: boolean; name: string; ondelete: () => void };
	let { open = $bindable(false), name, ondelete }: Props = $props();
</script>

<Dialog bind:open title={m.delete_reply_title()} description={m.delete_reply_body({ name })}>
	{#snippet actions()}
		<Button label={m.action_cancel()} variant="secondary" onclick={() => (open = false)} />
		<Button
			label={m.reply_delete()}
			variant="danger"
			iconStart={Trash2}
			onclick={() => {
				ondelete();
				open = false;
			}}
		/>
	{/snippet}
</Dialog>
