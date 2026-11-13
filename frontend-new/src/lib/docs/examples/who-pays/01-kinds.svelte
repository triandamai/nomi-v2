<!-- Who pays for each kind. -->
<script lang="ts">
	import WhoPays from '$lib/ds/settings/WhoPays.svelte';
</script>

<div class="w-58">
	<WhoPays
		label="Who pays"
		kinds={[
			{ tag: 'Included', text: 'Counts toward your plan’s 1M tokens.', accent: true },
			{ tag: 'Your key', text: 'Billed by that provider. Doesn’t touch your plan.' },
			{ tag: 'Local', text: 'Runs on your machine. Free, works offline.' },
		]}
	/>
</div>
