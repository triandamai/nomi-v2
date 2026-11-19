<script lang="ts" module>
	export type Device = 'phone' | 'desktop';
</script>

<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { Monitor, Smartphone } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Flow / Mobile Build → Code, terminal & phone preview, step 5 (PsC5F) → Device Toggle (G0ih1):
	// whether the phone's preview shows the app as a phone sees it or the desktop layout, scaled to
	// fit. 40 high on surface-2 (radius 10, inset 2), each side its icon and word at 13, the chosen
	// one on bg in text at 500. A radio group: the arrow keys move the choice.
	type Props = { value?: Device; onchange?: (value: Device) => void };
	let { value = $bindable('phone'), onchange }: Props = $props();

	const options = [
		{ value: 'phone' as const, label: () => m.workspace_phone(), icon: Smartphone },
		{ value: 'desktop' as const, label: () => m.workspace_desktop(), icon: Monitor },
	];
	let buttons: HTMLButtonElement[] = $state([]);

	function choose(next: Device, focus = false) {
		value = next;
		onchange?.(next);
		if (focus) buttons[options.findIndex((option) => option.value === next)]?.focus();
	}

	function onkeydown(event: KeyboardEvent) {
		if (!['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown'].includes(event.key)) return;
		event.preventDefault();
		choose(value === 'phone' ? 'desktop' : 'phone', true);
	}
</script>

<div role="radiogroup" aria-label={m.workspace_device()} tabindex="-1" {onkeydown} class="inline-flex h-10 gap-0.5 rounded-md bg-surface-2 p-0.5">
	{#each options as option, index (option.value)}
		{@const checked = option.value === value}
		<button
			bind:this={buttons[index]}
			type="button"
			role="radio"
			aria-checked={checked}
			tabindex={checked ? 0 : -1}
			class={[
				'inline-flex cursor-pointer items-center gap-1.5 rounded-inner px-3 text-ui outline-offset-1 focus-visible:outline-2 focus-visible:outline-focus',
				checked ? 'bg-bg font-medium text-text' : 'text-text-2',
			]}
			onclick={() => choose(option.value)}
		>
			<Icon icon={option.icon} size={14} />
			{option.label()}
		</button>
	{/each}
</div>
