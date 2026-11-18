<!-- Run this plan?: the recommended way first. -->
<script lang="ts">
	import PhoneRunSheet from '$lib/ds/thread/PhoneRunSheet.svelte';
	import Button from '$lib/ds/actions/Button.svelte';

	let open = $state(false);
</script>

<Button variant="secondary" label="Review & run" onclick={() => (open = true)} />
<PhoneRunSheet
	bind:open
	subtitle="5 steps · Nomi, Dana, Tara"
	options={[{ title: 'Run it, ask before anything leaves Nomi', recommended: true }, { title: 'Research only' }, { title: 'Keep planning', detail: 'Tell Nomi what to change' }]}
	onrun={() => (open = false)}
/>
