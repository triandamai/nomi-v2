<script lang="ts">
	import { tick, type Component, type Snippet } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Design System / Nomi → Overlays: a menu off a button. It floats (surface, border-strong, the
	// one shadow) in the top layer, so nothing clips it. Keyboard like the platform's menus: arrows
	// and Home/End move, Enter or Space picks, Escape (or Tab, or a click elsewhere) closes.
	// `{ group }` starts a labelled group of the items after it; `footer` is a quiet note at the end.
	type Item = { label: string; icon?: Component<LucideProps>; shortcut?: string; meta?: string; danger?: boolean; onselect?: () => void };
	type Entry = Item | 'divider' | { group: string };
	type TriggerProps = { 'aria-haspopup': 'menu'; 'aria-expanded': boolean; 'aria-controls': string; onclick: () => void };
	type Props = { items: Entry[]; trigger: Snippet<[TriggerProps]>; title?: string; footer?: string; wide?: boolean; align?: 'start' | 'end' };
	let { items, trigger, title, footer, wide = false, align = 'start' }: Props = $props();

	const sections = $derived.by(() => {
		const out: { label?: string; entries: (Item | 'divider')[] }[] = [{ entries: [] }];
		for (const entry of items) {
			if (typeof entry === 'object' && 'group' in entry) out.push({ label: entry.group, entries: [] });
			else out[out.length - 1].entries.push(entry);
		}
		return out.filter((section) => section.label || section.entries.length > 0);
	});

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

{#snippet row(entry: Item | 'divider')}
	{#if entry === 'divider'}
		<hr class="border-border" />
	{:else}
		<button
			type="button"
			role="menuitem"
			tabindex="-1"
			onclick={() => pick(entry)}
			class={[
				'flex h-7.5 w-full cursor-pointer items-center gap-2 rounded-xs px-2 text-left text-ui outline-none hover:bg-surface-2 focus-visible:bg-surface-2 pointer-coarse:min-h-11',
				entry.danger ? 'text-danger' : 'text-text',
			]}
		>
			{#if entry.icon}<span class={['inline-flex', entry.danger ? 'text-danger' : 'text-text-2']}><Icon icon={entry.icon} size={14} /></span>{/if}
			<span class="min-w-0 flex-1 truncate">{entry.label}</span>
			{#if entry.meta}<span class="font-mono text-label text-text-3">{entry.meta}</span>{/if}
			{#if entry.shortcut}<span class="font-mono text-label text-text-3">{entry.shortcut}</span>{/if}
		</button>
	{/if}
{/snippet}

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
		class={['fixed m-0 flex flex-col gap-0.5 rounded-md border border-border-strong bg-surface p-1 text-text shadow-float', wide ? 'w-70' : 'w-59']}
	>
		{#if title}<p class="font-mono text-label tracking-label text-text-3 uppercase">{title}</p>{/if}
		{#each sections as section, at (at)}
			{#if section.label}
				<div role="group" aria-labelledby="{id}-group-{at}" class="flex flex-col gap-0.5">
					<p id="{id}-group-{at}" role="presentation" class="px-2 pt-1.5 pb-0.5 font-mono text-label tracking-label text-text-3 uppercase">{section.label}</p>
					{#each section.entries as entry, index (index)}{@render row(entry)}{/each}
				</div>
			{:else}
				{#each section.entries as entry, index (index)}{@render row(entry)}{/each}
			{/if}
		{/each}
		{#if footer}
			<hr class="border-border" />
			<p class="px-2 py-1.5 font-mono text-label text-text-3">{footer}</p>
		{/if}
	</div>
{/if}
