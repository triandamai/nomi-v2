<script lang="ts">
	import Toggle from '../inputs/Toggle.svelte';

	// App / Inbox (BFoiZ) → Delivery → Setting: one way things reach the person, what it means
	// under it, and the switch at the end, 12 apart.
	type Props = { title: string; description: string; checked?: boolean; disabled?: boolean; onchange?: (checked: boolean) => void };
	let { title, description, checked = $bindable(false), disabled = false, onchange }: Props = $props();
</script>

<div class="flex items-center gap-3">
	<div class="flex min-w-0 flex-1 flex-col gap-0.5">
		<span class="text-ui font-medium text-text">{title}</span>
		<span class="text-caption text-text-3">{description}</span>
	</div>
	<Toggle label={title} bind:checked {disabled} {onchange} />
</div>
