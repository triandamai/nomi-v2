<script lang="ts">
	import CheckMark from './CheckMark.svelte';

	// Design System / Nomi → Check: a real checkbox, 16px, filled in text colour when checked.
	type Props = { label: string; checked?: boolean; disabled?: boolean; name?: string; onchange?: (checked: boolean) => void };
	let { label, checked = $bindable(false), disabled = false, name, onchange }: Props = $props();
</script>

<label class={['inline-flex cursor-pointer items-center gap-2 text-ui text-text', disabled && 'cursor-default opacity-40']}>
	<CheckMark bind:checked {disabled} {name} {onchange} />
	{label}
</label>
