<script lang="ts">
	import { Download } from '@lucide/svelte';
	import Dialog from './Dialog.svelte';
	import Button from '../actions/Button.svelte';
	import TextField from '../inputs/TextField.svelte';

	let { onclose, ondelete }: { onclose?: () => void; ondelete?: () => void } = $props();
	let open = $state(false);
</script>

<Button label="Delete project" variant="danger" onclick={() => (open = true)} />
<Dialog
	bind:open
	title="Delete salah-daily-tracker?"
	description="The live URL stops working and the database is removed. Your code stays in your account for 30 days; export it first if you want a copy."
	{onclose}
>
	<TextField label="Type the project name to confirm" placeholder="salah-daily-tracker" mono />
	{#snippet actions()}
		<Button label="Export repo first" variant="ghost" iconStart={Download} />
		<Button label="Cancel" variant="secondary" onclick={() => (open = false)} />
		<Button label="Delete project" variant="danger" onclick={ondelete} />
	{/snippet}
</Dialog>
