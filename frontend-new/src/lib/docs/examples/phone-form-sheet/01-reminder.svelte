<!-- A reminder form: a date that has passed. -->
<script lang="ts">
	import PhoneFormSheet from '$lib/ds/thread/PhoneFormSheet.svelte';
	import Button from '$lib/ds/actions/Button.svelte';

	let open = $state(false);
</script>

<Button variant="secondary" label="Edit the reminder" onclick={() => (open = true)} />
<PhoneFormSheet
	bind:open
	title="New reminder"
	subtitle="Kala · prefilled from chat"
	today="2026-11-17"
	fields={[
		{ key: 'what', label: 'What', type: 'text', required: true, options: [] },
		{ key: 'date', label: 'Date', type: 'date', required: true, options: [] },
		{ key: 'time', label: 'Time', type: 'time', required: false, options: [] },
		{ key: 'repeat', label: 'Repeat', type: 'select', required: false, options: ['Once', 'Weekly', 'Monthly', 'Yearly'] },
	]}
	values={{ what: 'Pay kos rent to Bu Sri', date: '2026-10-25', time: '09:00', repeat: 'Monthly' }}
	onsubmit={() => (open = false)}
/>
