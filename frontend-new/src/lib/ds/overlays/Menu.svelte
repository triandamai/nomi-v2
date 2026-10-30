<script lang="ts">
	import { tick, type Snippet } from 'svelte';
	import MenuPanel, { type MenuEntry, type MenuItem } from './MenuPanel.svelte';

	// Design System / Nomi → Overlays: a menu off a button. It floats (surface, border-strong, the
	// one shadow) in the top layer, so nothing clips it. Keyboard like the platform's menus: arrows
	// and Home/End move, Enter or Space picks, Escape (or Tab, or a click elsewhere) closes.
	// `{ group }` starts a labelled group of the items after it; `footer` is a quiet note at the end.
	type Item = MenuItem;
	type Entry = MenuEntry;
	type TriggerProps = { 'aria-haspopup': 'menu'; 'aria-expanded': boolean; 'aria-controls': string; onclick: () => void };
	type Props = { items: Entry[]; trigger: Snippet<[TriggerProps]>; title?: string; footer?: string; wide?: boolean; align?: 'start' | 'end' };
	let { items, trigger, title, footer, wide = false, align = 'start' }: Props = $props();

	const id = $props.id();
	let open = $state(false);
	let anchor: HTMLElement | undefined = $state();
	let menu: HTMLElement | undefined = $state();
	let position = $state({ top: 0, left: 0 });

	const buttons = $derived(() => [...(menu?.querySelectorAll<HTMLElement>('[role="menuitem"]') ?? [])]);

	function triggerElement() {
		return anchor?.querySelector<HTMLElement>('button, a') ?? undefined;
	}

	function place() {
		const box = anchor?.getBoundingClientRect();
		if (!box || !menu) return;
		const width = menu.offsetWidth;
		const height = menu.offsetHeight;
		const below = box.bottom + 4 + height <= window.innerHeight;
		position = {
			top: below ? box.bottom + 4 : Math.max(8, box.top - 4 - height),
			left: Math.min(Math.max(8, align === 'end' ? box.right - width : box.left), window.innerWidth - width - 8),
		};
	}

	async function show() {
		open = true;
		await tick();
		menu?.showPopover();
		place();
		buttons()[0]?.focus();
	}

	function hide(returnFocus: boolean) {
		open = false;
		if (returnFocus) triggerElement()?.focus();
	}

	function pick(item: Item) {
		hide(true);
		item.onselect?.();
	}

	function onkeydown(event: KeyboardEvent) {
		const all = buttons();
		const at = all.indexOf(document.activeElement as HTMLElement);
		const next = { ArrowDown: at + 1, ArrowUp: at - 1, Home: 0, End: all.length - 1 }[event.key];
		if (next !== undefined) {
			event.preventDefault();
			all[(next + all.length) % all.length]?.focus();
		} else if (event.key === 'Escape') {
			event.preventDefault();
			hide(true);
		} else if (event.key === 'Tab') {
			hide(false);
		}
	}

	$effect(() => {
		if (!open) return;
		const away = (event: PointerEvent) => {
			const target = event.target as Node;
			if (!menu?.contains(target) && !anchor?.contains(target)) hide(false);
		};
		document.addEventListener('pointerdown', away);
		window.addEventListener('resize', place);
		return () => {
			document.removeEventListener('pointerdown', away);
			window.removeEventListener('resize', place);
		};
	});
</script>

<span bind:this={anchor} class="inline-flex">
	{@render trigger({ 'aria-haspopup': 'menu', 'aria-expanded': open, 'aria-controls': `${id}-menu`, onclick: () => (open ? hide(true) : show()) })}
</span>

{#if open}
	<MenuPanel
		bind:element={menu}
		id="{id}-menu"
		{items}
		{title}
		{footer}
		{wide}
		onpick={pick}
		popover="manual"
		{onkeydown}
		style="top: {position.top}px; left: {position.left}px"
		class="fixed m-0"
	/>
{/if}
