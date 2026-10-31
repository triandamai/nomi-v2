<!-- Confirm a delete: press the button to open it. Escape or Cancel closes it. -->
<script lang="ts">
	import { Download, Trash2 } from '@lucide/svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import Dialog from '$lib/ds/overlays/Dialog.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';

	let open = $state(false);
</script>

<Button label="Delete project" variant="danger" iconStart={Trash2} onclick={() => (open = true)} />
<Dialog
	bind:open
	title="Delete salah-daily-tracker?"
	description="The live URL stops working and the database is removed. Your code stays in your account for 30 days; export it first if you want a copy."
>
	<TextField label="Type the project name to confirm" placeholder="salah-daily-tracker" mono />
	{#snippet actions()}
		<Button label="Export repo first" variant="ghost" iconStart={Download} />
		<Button label="Cancel" variant="secondary" onclick={() => (open = false)} />
		<Button label="Delete project" variant="danger" iconStart={Trash2} onclick={() => (open = false)} />
	{/snippet}
</Dialog>
