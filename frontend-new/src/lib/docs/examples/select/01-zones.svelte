<!-- Chosen, and nothing chosen yet. -->
<script lang="ts">
	import Select from '$lib/ds/inputs/Select.svelte';

	const zones = [
		{ value: 'Asia/Jakarta', label: 'Asia/Jakarta · WIB' },
		{ value: 'Asia/Makassar', label: 'Asia/Makassar · WITA' },
		{ value: 'Asia/Jayapura', label: 'Asia/Jayapura · WIT' },
	];
	let chosen = $state('Asia/Jakarta');
	let none = $state('');
</script>

<div class="flex items-center gap-3">
	<Select label="Time zone" options={zones} bind:value={chosen} />
	<Select label="Time zone" options={zones} bind:value={none} placeholder="Choose your time zone" />
</div>
