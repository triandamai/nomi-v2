<script lang="ts">
	import Switch from '../../m3/Switch.svelte';

	// Twin of frontend-new's Toggle on the M3 switch.
	type Props = { label: string; checked?: boolean; disabled?: boolean; onchange?: (checked: boolean) => void };
	let { label, checked = $bindable(false), disabled = false, onchange }: Props = $props();
</script>

<Switch bind:checked {disabled} aria-label={label} {onchange} />
