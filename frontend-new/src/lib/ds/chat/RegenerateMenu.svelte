<script lang="ts">
	import type { Component, ComponentProps } from 'svelte';
	import { List, Maximize2, Minimize2, RefreshCw, type LucideProps } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Menu from '../overlays/Menu.svelte';

	// Chat responses → Regenerate Menu, off the reply's Regenerate button: the same again, shorter,
	// with more detail, as a list, or with another model (and what it's paid with).
	type Model = { name: string; source: string; icon?: Component<LucideProps> };
	type Choice = { style: 'again' | 'shorter' | 'longer' | 'list' } | { model: string };
	type Props = { models?: Model[]; onpick: (choice: Choice) => void; trigger?: ComponentProps<typeof Menu>['trigger']; inline?: boolean };
	let { models = [], onpick, trigger, inline = false }: Props = $props();
</script>

<Menu
	wide
	{trigger}
	{inline}
	items={[
		{ label: m.regen_again(), icon: RefreshCw, shortcut: '⌘R', onselect: () => onpick({ style: 'again' }) },
		{ label: m.regen_shorter(), icon: Minimize2, onselect: () => onpick({ style: 'shorter' }) },
		{ label: m.regen_longer(), icon: Maximize2, onselect: () => onpick({ style: 'longer' }) },
		{ label: m.regen_list(), icon: List, onselect: () => onpick({ style: 'list' }) },
		...(models.length ? ['divider' as const, { group: m.regen_models() }] : []),
		...models.map((model) => ({ label: model.name, icon: model.icon, meta: model.source, onselect: () => onpick({ model: model.name }) })),
	]}
/>
