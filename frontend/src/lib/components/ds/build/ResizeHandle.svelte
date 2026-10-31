<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { SPLIT, chatWidth, nudge } from './split';

	// Build → Workspace / Resize handle: the split between chat and preview. Drag it, use the arrow
	// keys (Shift for bigger steps), or double-click to reset; it turns mint while dragging.
	type Props = {
		width: number;
		/** The room the chat and preview share, so the preview keeps its 480px. */
		available: number;
		onresize: (width: number) => void;
		/** Show the dragging state without a pointer (the gallery). */
		'data-dragging'?: boolean;
		'data-hover'?: boolean;
	};
	let { width, available, onresize, 'data-dragging': showDragging = false, 'data-hover': showHover = false }: Props = $props();

	let dragging = $state(false);
	let start = { x: 0, width: 0 };
	const active = $derived(dragging || showDragging);

	function onpointerdown(event: PointerEvent) {
		dragging = true;
		start = { x: event.clientX, width };
		(event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
	}

	function onpointermove(event: PointerEvent) {
		if (dragging) onresize(chatWidth(start.width + event.clientX - start.x, available));
	}

	function onkeydown(event: KeyboardEvent) {
		const next = nudge(width, event.key, event.shiftKey, available);
		if (next === width && !['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) return;
		event.preventDefault();
		onresize(next);
	}
</script>

<!-- A focusable separator with a value is the WAI-ARIA window splitter: interactive by design. -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
<div
	role="separator"
	aria-orientation="vertical"
	aria-label={m.ws_resize()}
	aria-valuenow={width}
	aria-valuemin={SPLIT.min}
	aria-valuemax={chatWidth(SPLIT.max, available)}
	tabindex="0"
	data-hover={showHover || undefined}
	class={[
		'flex h-full min-h-10 w-2.25 shrink-0 cursor-col-resize touch-none items-center justify-center outline-offset-0 select-none',
		active ? 'border-l-2 border-accent' : 'border-l border-border hover:bg-surface data-hover:bg-surface',
	]}
	{onpointerdown}
	{onpointermove}
	onpointerup={() => (dragging = false)}
	onpointercancel={() => (dragging = false)}
	ondblclick={() => onresize(chatWidth(SPLIT.reset, available))}
	{onkeydown}
>
	<span data-grip class={['flex flex-col gap-0.75 rounded-xs px-0.5 py-1.5', active ? 'bg-accent' : 'bg-surface-2']}>
		{#each [0, 1, 2] as dot (dot)}<span class={['size-0.75 rounded-full', active ? 'bg-accent-ink' : 'bg-text-3']}></span>{/each}
	</span>
</div>
