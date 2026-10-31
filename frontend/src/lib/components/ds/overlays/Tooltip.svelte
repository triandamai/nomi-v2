<script lang="ts">
	import { tick, type Snippet } from 'svelte';
	import TooltipBubble from './TooltipBubble.svelte';

	// Twin of frontend-new's Tooltip: an M3 plain tooltip with the shortcut, on hover (after a
	// moment) or keyboard focus. Escape hides it. It describes its trigger to screen readers.
	type Props = { label: string; shortcut?: string; children: Snippet<[{ 'aria-describedby': string }]> };
	let { label, shortcut, children }: Props = $props();

	const id = $props.id();
	let anchor: HTMLElement | undefined = $state();
	let tip: HTMLElement | undefined = $state();
	let open = $state(false);
	let position = $state({ top: 0, left: 0 });
	let timer: ReturnType<typeof setTimeout> | undefined;

	async function show() {
		clearTimeout(timer);
		open = true;
		await tick();
		tip?.showPopover();
		const box = anchor?.getBoundingClientRect();
		if (!box || !tip) return;
		const below = box.bottom + 6 + tip.offsetHeight <= window.innerHeight;
		position = {
			top: below ? box.bottom + 6 : box.top - 6 - tip.offsetHeight,
			left: Math.min(Math.max(8, box.left + box.width / 2 - tip.offsetWidth / 2), window.innerWidth - tip.offsetWidth - 8),
		};
	}

	function hide() {
		clearTimeout(timer);
		open = false;
	}

	$effect(() => {
		if (!open) return;
		const escape = (event: KeyboardEvent) => event.key === 'Escape' && hide();
		window.addEventListener('keydown', escape);
		return () => window.removeEventListener('keydown', escape);
	});
</script>

<span
	bind:this={anchor}
	role="presentation"
	class="anchor"
	onmouseenter={() => (timer = setTimeout(show, 300))}
	onmouseleave={hide}
	onfocusin={(event) => (event.target as HTMLElement).matches(':focus-visible') && show()}
	onfocusout={hide}
>
	{@render children({ 'aria-describedby': id })}
</span>
{#if open}
	<TooltipBubble bind:element={tip} {id} {label} {shortcut} popover="manual" style="position: fixed; margin: 0; top: {position.top}px; left: {position.left}px" />
{/if}

<style>
	.anchor {
		display: inline-flex;
	}
</style>
