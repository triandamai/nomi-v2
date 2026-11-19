<script lang="ts" module>
	export type Device = 'phone' | 'desktop';
</script>

<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { ArrowUpRight, Monitor, RotateCw, Smartphone } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Flow / Mobile Build → Code, terminal & phone preview, step 5 (PsC5F) → Preview Bar (GppXE): the
	// preview's own bar under Chat | Preview, 40 high with a bottom stroke. The app's address in mono,
	// Reload, Open in browser, and Phone or Desktop as two icons on a surface-2 track, the chosen one
	// raised (surface, border-strong), so the choice reads in dark and light alike. It replaces the
	// pen's earlier centred pill (G0ih1), which read as a second tab bar. Each control keeps its drawn
	// size; on a coarse pointer it takes a 44pt hit area around it.
	type Props = {
		address: string;
		device?: Device;
		/** Where Open in browser goes; left out when the address isn't one to open (the sandbox's holds a secret). */
		href?: string;
		onreload: () => void;
		ondevice?: (value: Device) => void;
	};
	let { address, device = $bindable('phone'), href, onreload, ondevice }: Props = $props();

	const devices = [
		{ value: 'phone' as const, label: () => m.workspace_phone(), icon: Smartphone },
		{ value: 'desktop' as const, label: () => m.workspace_desktop(), icon: Monitor },
	];
	let buttons: HTMLButtonElement[] = $state([]);

	function choose(next: Device, focus = false) {
		device = next;
		ondevice?.(next);
		if (focus) buttons[devices.findIndex((one) => one.value === next)]?.focus();
	}

	function onkeydown(event: KeyboardEvent) {
		if (!['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown'].includes(event.key)) return;
		event.preventDefault();
		choose(device === 'phone' ? 'desktop' : 'phone', true);
	}
</script>

<div class="flex h-10 shrink-0 items-center gap-0.5 border-b border-border bg-bg pr-2 pl-4">
	<p class="min-w-0 flex-1 truncate font-mono text-caption text-text-3">{address}</p>
	<button type="button" aria-label={m.workspace_reload()} class="relative inline-flex size-8 shrink-0 cursor-pointer items-center justify-center rounded-sm text-text-2 outline-offset-1 hover:bg-surface-2 focus-visible:outline-2 focus-visible:outline-focus pointer-coarse:after:absolute pointer-coarse:after:-inset-1.5" onclick={onreload}>
		<Icon icon={RotateCw} size={16} />
	</button>
	{#if href}
		<a {href} target="_blank" rel="noopener noreferrer" aria-label={m.workspace_open_tab()} class="relative inline-flex size-8 shrink-0 items-center justify-center rounded-sm text-text-2 outline-offset-1 hover:bg-surface-2 focus-visible:outline-2 focus-visible:outline-focus pointer-coarse:after:absolute pointer-coarse:after:-inset-1.5">
			<Icon icon={ArrowUpRight} size={16} />
		</a>
	{/if}
	<div role="radiogroup" aria-label={m.workspace_device()} tabindex="-1" {onkeydown} class="ml-1 inline-flex h-8 shrink-0 items-center gap-0.5 rounded-inner bg-surface-2 p-0.5">
		{#each devices as one, index (one.value)}
			{@const checked = one.value === device}
			<button
				bind:this={buttons[index]}
				type="button"
				role="radio"
				aria-checked={checked}
				aria-label={one.label()}
				tabindex={checked ? 0 : -1}
				class={[
					'relative inline-flex size-7 cursor-pointer items-center justify-center rounded-sm outline-offset-1 focus-visible:outline-2 focus-visible:outline-focus pointer-coarse:after:absolute pointer-coarse:after:-inset-2',
					checked ? 'bg-surface text-text outline -outline-offset-1 outline-border-strong' : 'text-text-3',
				]}
				onclick={() => choose(one.value)}
			>
				<Icon icon={one.icon} size={14} />
			</button>
		{/each}
	</div>
</div>
