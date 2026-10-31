<!-- Settings rows: the words sit beside the switch, and the same name goes in its label. -->
<script lang="ts">
	import Toggle from '$lib/ds/inputs/Toggle.svelte';

	let mask = $state(true);
	let anyone = $state(false);
</script>

<div class="flex w-full max-w-100 flex-col gap-3">
	<div class="flex items-center gap-3">
		<span class="flex min-w-0 flex-1 flex-col gap-0.5">
			<span class="text-ui text-text">Hide amounts and names</span>
			<span class="text-caption text-text-3">Shows Rp ••• and initials. Your records never leave Nomi.</span>
		</span>
		<Toggle label="Hide amounts and names" bind:checked={mask} />
	</div>
	<div class="flex items-center gap-3">
		<span class="flex min-w-0 flex-1 flex-col gap-0.5">
			<span class="text-ui text-text">Anyone with the link can view</span>
			<span class="text-caption text-text-3">Read-only. No sign-in, no replies.</span>
		</span>
		<Toggle label="Anyone with the link can view" bind:checked={anyone} />
	</div>
</div>
