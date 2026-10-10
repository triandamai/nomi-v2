<!-- Preferences → Navigation drawer. Home, Chats and Projects are fixed at the top; below them the
     pinned features, which can be dragged (or moved with the arrow keys / buttons) into any order
     and unpinned, then everything not pinned, ready to add. Each change saves straight away. -->
<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { flip } from 'svelte/animate';
	import IconDragHandle from '$lib/components/icons/IconDragHandle.svelte';
	import IconLock from '$lib/components/icons/IconLock.svelte';
	import IconClose from '$lib/components/icons/IconClose.svelte';
	import IconPlus from '$lib/components/icons/IconPlus.svelte';
	import IconChevronUp from '$lib/components/icons/IconChevronUp.svelte';
	import IconChevronDown from '$lib/components/icons/IconChevronDown.svelte';
	import Snackbar from '$lib/components/m3/Snackbar.svelte';
	import { FEATURES, PERMANENT, PINNABLE, DEFAULT_PINS, moveItem, type PinnableId } from '$lib/features';
	import { currentPins, savePins } from '$lib/drawerStore.svelte';

	let { savedPins }: { savedPins: string[] | null | undefined } = $props();

	const pins = $derived(currentPins(savedPins));
	const available = $derived(PINNABLE.filter((id) => !pins.includes(id)));

	// While dragging, the order being shown; null otherwise.
	let dragOrder = $state<PinnableId[] | null>(null);
	let dragging = $state<PinnableId | null>(null);
	let dragOffset = $state(0);
	let announcement = $state('');
	let toast = $state('');
	let toastOpen = $state(false);

	const shown = $derived(dragOrder ?? pins);

	async function commit(next: PinnableId[]) {
		if (next.join() === pins.join()) return;
		const ok = await savePins(next);
		if (!ok) {
			toast = m.drawer_save_failed();
			toastOpen = true;
		}
	}

	function move(id: PinnableId, by: number) {
		const from = pins.indexOf(id);
		const to = from + by;
		if (to < 0 || to >= pins.length) return;
		announcement = m.drawer_moved({ name: FEATURES[id].label(), position: to + 1 });
		commit(moveItem(pins, from, to));
	}

	// Pointer drag: the row follows the finger; crossing half of a neighbour swaps them.
	let startY = 0;
	let rowHeight = 56;
	function onPointerDown(event: PointerEvent, id: PinnableId) {
		if (event.button !== 0) return;
		event.preventDefault();
		(event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
		const row = (event.currentTarget as HTMLElement).closest('li');
		rowHeight = row?.getBoundingClientRect().height ?? 56;
		startY = event.clientY;
		dragging = id;
		dragOrder = [...pins];
		dragOffset = 0;
	}
	function onPointerMove(event: PointerEvent) {
		if (!dragging || !dragOrder) return;
		let offset = event.clientY - startY;
		const index = dragOrder.indexOf(dragging);
		// Swap with the neighbour once the row is more than half way over it.
		if (offset > rowHeight / 2 && index < dragOrder.length - 1) {
			dragOrder = moveItem(dragOrder, index, index + 1);
			startY += rowHeight;
			offset -= rowHeight;
		} else if (offset < -rowHeight / 2 && index > 0) {
			dragOrder = moveItem(dragOrder, index, index - 1);
			startY -= rowHeight;
			offset += rowHeight;
		}
		dragOffset = offset;
	}
	function onPointerUp() {
		if (!dragging || !dragOrder) return;
		const next = dragOrder;
		const id = dragging;
		dragging = null;
		dragOrder = null;
		dragOffset = 0;
		if (next.join() !== pins.join()) announcement = m.drawer_moved({ name: FEATURES[id].label(), position: next.indexOf(id) + 1 });
		commit(next);
	}
	function onHandleKey(event: KeyboardEvent, id: PinnableId) {
		if (event.key === 'ArrowUp') {
			event.preventDefault();
			move(id, -1);
		} else if (event.key === 'ArrowDown') {
			event.preventDefault();
			move(id, 1);
		}
	}
</script>

<div class="drawer-config">
	<h3 class="drawer-config__label">{m.drawer_always()}</h3>
	<ul class="rows">
		{#each PERMANENT as id (id)}
			{@const feature = FEATURES[id]}
			<li class="row row--fixed">
				<span class="row__lead" aria-hidden="true"><IconLock size={18} /></span>
				<span class="row__icon"><feature.icon size={22} /></span>
				<span class="row__name">{feature.label()}</span>
			</li>
		{/each}
	</ul>

	<h3 class="drawer-config__label">{m.drawer_pinned()}</h3>
	{#if shown.length === 0}
		<p class="drawer-config__empty">{m.drawer_none_pinned()}</p>
	{:else}
		<ol class="rows">
			{#each shown as id, i (id)}
				{@const feature = FEATURES[id]}
				<li
					class="row row--pinned"
					class:row--dragging={dragging === id}
					style:transform={dragging === id ? `translateY(${dragOffset}px)` : undefined}
					animate:flip={{ duration: dragging === id ? 0 : 180 }}
				>
					<button
						type="button"
						class="row__handle"
						aria-label={m.drawer_drag({ name: feature.label() })}
						title={m.drawer_drag({ name: feature.label() })}
						onpointerdown={(e) => onPointerDown(e, id)}
						onpointermove={onPointerMove}
						onpointerup={onPointerUp}
						onpointercancel={onPointerUp}
						onkeydown={(e) => onHandleKey(e, id)}
					>
						<IconDragHandle size={20} />
					</button>
					<span class="row__icon row__icon--pinned"><feature.icon size={22} /></span>
					<span class="row__name">{feature.label()}</span>
					<span class="row__actions">
						<button type="button" class="row__btn" disabled={i === 0} aria-label={m.drawer_move_up({ name: feature.label() })} onclick={() => move(id, -1)}>
							<IconChevronUp size={18} />
						</button>
						<button type="button" class="row__btn" disabled={i === shown.length - 1} aria-label={m.drawer_move_down({ name: feature.label() })} onclick={() => move(id, 1)}>
							<IconChevronDown size={18} />
						</button>
						<button type="button" class="row__btn" aria-label={m.drawer_remove({ name: feature.label() })} onclick={() => commit(pins.filter((p) => p !== id))}>
							<IconClose size={18} />
						</button>
					</span>
				</li>
			{/each}
		</ol>
	{/if}

	<h3 class="drawer-config__label">{m.drawer_available()}</h3>
	{#if available.length === 0}
		<p class="drawer-config__empty">{m.drawer_all_pinned()}</p>
	{:else}
		<ul class="rows">
			{#each available as id (id)}
				{@const feature = FEATURES[id]}
				<li class="row">
					<span class="row__icon"><feature.icon size={22} /></span>
					<span class="row__text">
						<span class="row__name">{feature.label()}</span>
						<span class="row__desc">{feature.description()}</span>
					</span>
					<button type="button" class="row__add" aria-label={m.drawer_add({ name: feature.label() })} onclick={() => commit([...pins, id])}>
						<IconPlus size={18} />
					</button>
				</li>
			{/each}
		</ul>
	{/if}

	<button type="button" class="drawer-config__reset" disabled={pins.join() === DEFAULT_PINS.join()} onclick={() => commit([...DEFAULT_PINS])}>
		{m.drawer_reset()}
	</button>
	<p class="sr-only" aria-live="polite">{announcement}</p>
</div>

<Snackbar bind:open={toastOpen} message={toast} />

<style>
	.drawer-config {
		display: flex;
		flex-direction: column;
		gap: 8px;
		max-width: 520px;
	}
	.drawer-config__label {
		margin: 12px 0 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
		font-weight: 600;
	}
	.drawer-config__empty {
		margin: 0;
		padding: 16px;
		border: 1.5px dashed var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-large);
		color: var(--md-sys-color-on-surface-variant);
	}
	.rows {
		display: flex;
		flex-direction: column;
		gap: 4px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.row {
		display: flex;
		align-items: center;
		gap: 12px;
		min-height: 56px;
		padding: 4px 8px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container);
		color: var(--md-sys-color-on-surface);
	}
	.row--fixed {
		background: var(--md-sys-color-surface-container-low);
		color: var(--md-sys-color-on-surface-variant);
	}
	.row--pinned {
		position: relative;
		background: var(--md-sys-color-surface-container-high);
		touch-action: none;
	}
	.row--dragging {
		z-index: 1;
		background: var(--md-sys-color-surface-container-highest);
		box-shadow: 0 6px 20px color-mix(in srgb, black 25%, transparent);
	}
	.row__lead {
		display: grid;
		place-items: center;
		width: 40px;
		opacity: 0.6;
	}
	.row__handle {
		display: grid;
		place-items: center;
		width: 40px;
		height: 48px;
		flex: none;
		border: none;
		border-radius: var(--md-sys-shape-corner-medium);
		background: none;
		color: var(--md-sys-color-on-surface-variant);
		cursor: grab;
		touch-action: none;
	}
	.row--dragging .row__handle {
		cursor: grabbing;
	}
	.row__icon {
		display: grid;
		place-items: center;
		width: 40px;
		height: 40px;
		flex: none;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-highest);
		color: var(--md-sys-color-on-surface-variant);
	}
	.row:not(.row--pinned):not(.row--fixed) .row__icon {
		margin-left: 4px;
	}
	.row__icon--pinned {
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}
	.row__text {
		display: flex;
		flex-direction: column;
		min-width: 0;
		flex: 1;
	}
	.row__name {
		flex: 1;
		min-width: 0;
		font-weight: 600;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.row__desc {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.row__actions {
		display: flex;
		flex: none;
	}
	.row__btn,
	.row__add {
		display: grid;
		place-items: center;
		width: 40px;
		height: 40px;
		flex: none;
		border: none;
		border-radius: var(--md-sys-shape-corner-full);
		background: none;
		color: var(--md-sys-color-on-surface-variant);
		cursor: pointer;
	}
	.row__add {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.row__btn:hover:not(:disabled),
	.row__handle:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
	.row__btn:disabled {
		opacity: 0.35;
		cursor: default;
	}
	.row__btn:focus-visible,
	.row__add:focus-visible,
	.row__handle:focus-visible,
	.drawer-config__reset:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 1px;
	}
	/* Up/down buttons are for when dragging isn't handy; on phones the handle is enough. */
	@media (max-width: 480px) {
		.row__btn:not(:last-child) {
			display: none;
		}
	}
	.drawer-config__reset {
		align-self: flex-start;
		min-height: 40px;
		margin-top: 8px;
		padding: 0 16px;
		border: 1px solid var(--md-sys-color-outline);
		border-radius: var(--md-sys-shape-corner-full);
		background: none;
		color: var(--md-sys-color-primary);
		font: inherit;
		font-weight: 600;
		cursor: pointer;
	}
	.drawer-config__reset:disabled {
		opacity: 0.4;
		cursor: default;
	}
</style>
