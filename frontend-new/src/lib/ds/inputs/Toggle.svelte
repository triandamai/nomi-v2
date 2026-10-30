<script lang="ts">
	// Design System / Nomi → Toggle: a switch, mint with the knob right when on.
	type Props = { label: string; checked?: boolean; disabled?: boolean; onchange?: (checked: boolean) => void };
	let { label, checked = $bindable(false), disabled = false, onchange }: Props = $props();

	function flip() {
		checked = !checked;
		onchange?.(checked);
	}
</script>

<button
	type="button"
	role="switch"
	aria-checked={checked}
	aria-label={label}
	{disabled}
	onclick={flip}
	class={[
		'inline-flex h-4.5 w-8 shrink-0 cursor-pointer items-center rounded-full border p-px outline-offset-2 disabled:cursor-default disabled:opacity-40',
		checked ? 'justify-end border-accent bg-accent' : 'justify-start border-border-strong bg-surface-2',
	]}
>
	<span class={['size-3.5 rounded-full', checked ? 'bg-accent-ink' : 'bg-text-3']}></span>
</button>
