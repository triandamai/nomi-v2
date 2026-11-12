<script lang="ts">
	import { tick } from 'svelte';
	import { ArrowDown } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import SortMenu, { sortLabel, type ProjectSort } from './SortMenu.svelte';
	import { menuItems, nextIndex, placePanel } from '../overlays/floating';

	// Design System / Projects → Project / Sort Button (MYatL): the order the list is in ("Updated
	// ↓", newest first), 30 tall, 10 in, a quiet stroke. It opens the Sort Menu (MLYM5) below its
	// end, focus on the chosen order; arrows, Home/End, Enter and Escape as in any menu.
	type Props = { value?: ProjectSort; onchange?: (sort: ProjectSort) => void };
	let { value = $bindable('updated'), onchange }: Props = $props();

	const id = $props.id();
	let open = $state(false);
	let button: HTMLButtonElement | undefined = $state();
	let menu: HTMLElement | undefined = $state();
	let position = $state({ top: 0, left: 0 });

	function place() {
		if (!button || !menu) return;
		position = placePanel(button.getBoundingClientRect(), { width: menu.offsetWidth, height: menu.offsetHeight }, { width: window.innerWidth, height: window.innerHeight }, 'end');
	}

	async function show() {
		open = true;
		await tick();
		menu?.showPopover();
		place();
		const items = menuItems(menu);
		(items.find((item) => item.getAttribute('aria-checked') === 'true') ?? items[0])?.focus();
	}

	function hide(returnFocus: boolean) {
		open = false;
		if (returnFocus) button?.focus();
	}

	function choose(sort: ProjectSort) {
		hide(true);
		if (sort === value) return;
		value = sort;
		onchange?.(sort);
	}

	function onkeydown(event: KeyboardEvent) {
		const items = menuItems(menu);
		const next = nextIndex(event.key, items.indexOf(document.activeElement as HTMLElement), items.length);
		if (next !== null) {
			event.preventDefault();
			items[next]?.focus();
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
			if (!menu?.contains(target) && !button?.contains(target)) hide(false);
		};
		document.addEventListener('pointerdown', away);
		window.addEventListener('resize', place);
		return () => {
			document.removeEventListener('pointerdown', away);
			window.removeEventListener('resize', place);
		};
	});
</script>

<button
	bind:this={button}
	type="button"
	aria-label={m.projects_sort_button({ label: sortLabel(value) })}
	aria-haspopup="menu"
	aria-expanded={open}
	aria-controls="{id}-menu"
	onclick={() => (open ? hide(true) : show())}
	class="inline-flex h-7.5 shrink-0 cursor-pointer items-center gap-1.5 rounded-sm border border-border px-2.25 text-ui whitespace-nowrap text-text-2 outline-offset-1 hover:bg-surface-2 hover:text-text aria-expanded:bg-surface-2 pointer-coarse:min-h-11"
>
	{sortLabel(value)}
	<span class="inline-flex text-text-3"><Icon icon={ArrowDown} size={12} /></span>
</button>
{#if open}
	<SortMenu
		bind:element={menu}
		id="{id}-menu"
		{value}
		onchoose={choose}
		popover="manual"
		{onkeydown}
		style="top: {position.top}px; left: {position.left}px"
		class="fixed m-0"
	/>
{/if}
