<script lang="ts">
	import { tick, type Component, type Snippet } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Design System / Nomi → Overlays: a menu off a button. It floats (surface, border-strong, the
	// one shadow) in the top layer, so nothing clips it. Keyboard like the platform's menus: arrows
	// and Home/End move, Enter or Space picks, Escape (or Tab, or a click elsewhere) closes.
	type Item = { label: string; icon?: Component<LucideProps>; shortcut?: string; danger?: boolean; onselect?: () => void };
	type TriggerProps = { 'aria-haspopup': 'menu'; 'aria-expanded': boolean; 'aria-controls': string; onclick: () => void };
	type Props = { items: (Item | 'divider')[]; trigger: Snippet<[TriggerProps]>; title?: string; align?: 'start' | 'end' };
	let { items, trigger, title, align = 'start' }: Props = $props();

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
	<div
		bind:this={menu}
		id="{id}-menu"
		role="menu"
		tabindex="-1"
		popover="manual"
		{onkeydown}
		style="top: {position.top}px; left: {position.left}px"
		class="fixed m-0 flex w-59 flex-col gap-0.5 rounded-md border border-border-strong bg-surface p-1 text-text shadow-float"
	>
		{#if title}<p class="px-2 pt-1.5 pb-1 font-mono text-label tracking-label text-text-3 uppercase">{title}</p>{/if}
		{#each items as item, index (index)}
			{#if item === 'divider'}
				<hr class="my-0.5 border-border" />
			{:else}
				<button
					type="button"
					role="menuitem"
					tabindex="-1"
					onclick={() => pick(item)}
					class={[
						'flex h-7.5 w-full cursor-pointer items-center gap-2 rounded-xs px-2 text-left text-ui outline-none hover:bg-surface-2 focus-visible:bg-surface-2 pointer-coarse:min-h-11',
						item.danger ? 'text-danger' : 'text-text',
					]}
				>
					{#if item.icon}<span class={['inline-flex', item.danger ? 'text-danger' : 'text-text-2']}><Icon icon={item.icon} size={14} /></span>{/if}
					<span class="min-w-0 flex-1 truncate">{item.label}</span>
					{#if item.shortcut}<span class="font-mono text-label text-text-3">{item.shortcut}</span>{/if}
				</button>
			{/if}
		{/each}
	</div>
{/if}
