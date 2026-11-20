<script lang="ts">
	import { Monitor, Moon, Sun, SunMoon } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import type { Theme } from '$lib/theme';
	import { getThemeContext } from './themeContext';
	import IconButton from '../actions/IconButton.svelte';
	import BottomSheet from '../overlays/BottomSheet.svelte';
	import SheetOption from '../overlays/SheetOption.svelte';

	// Chrome → Theme Picker Button: opens a phone bottom sheet with the theme's three-way switch
	// (System, Light, Dark). Designed for mobile viewports where the sidebar and desktop account menu
	// are hidden.
	type Props = {
		theme?: Theme;
		ontheme?: (theme: Theme) => void;
		variant?: 'outline' | 'ghost';
		class?: string;
	};
	let { theme: propTheme, ontheme: propOntheme, variant = 'ghost', class: className = '' }: Props = $props();

	const ctx = getThemeContext();
	const activeTheme = $derived(propTheme ?? ctx?.theme ?? 'system');

	let open = $state(false);

	const icon = $derived(activeTheme === 'dark' ? Moon : activeTheme === 'light' ? Sun : SunMoon);

	const options: { value: Theme; label: () => string; icon: typeof Monitor }[] = [
		{ value: 'system', label: m.theme_system, icon: Monitor },
		{ value: 'light', label: m.theme_light, icon: Sun },
		{ value: 'dark', label: m.theme_dark, icon: Moon },
	];

	function pick(value: Theme) {
		if (propOntheme) propOntheme(value);
		else ctx?.changeTheme(value);
		open = false;
	}
</script>

<span class={className}>
	<IconButton
		{icon}
		label={m.account_theme()}
		{variant}
		onclick={() => (open = true)}
		aria-haspopup="dialog"
		aria-expanded={open}
	/>
</span>

<BottomSheet bind:open title={m.account_theme()} onclose={() => (open = false)}>
	{#each options as option (option.value)}
		{@const chosen = option.value === activeTheme}
		<SheetOption
			icon={option.icon}
			label={option.label()}
			{chosen}
			onclick={() => pick(option.value)}
		/>
	{/each}
</BottomSheet>
