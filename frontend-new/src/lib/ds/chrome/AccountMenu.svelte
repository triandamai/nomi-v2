<script lang="ts" module>
	export type Theme = 'system' | 'light' | 'dark';
</script>

<script lang="ts">
	import { tick } from 'svelte';
	import type { Component } from 'svelte';
	import { Check, ChevronLeft, ChevronRight, CreditCard, Keyboard, Languages, LogOut, Monitor, Moon, Settings, Settings2, Sparkle, Sun, SunMoon, User, type LucideProps } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Avatar from '../identity/Avatar.svelte';
	import MenuRow from '../overlays/MenuRow.svelte';
	import type { MenuItem } from '../overlays/MenuPanel.svelte';
	import { menuItems, nextIndex, placePanel } from '../overlays/floating';

	// App / Account menu → Account Menu (LkJLt): opens from the user row at the foot of the sidebar,
	// 272 wide (surface, border-strong inside, the larger float shadow). The person and their plan
	// with Upgrade; Settings, Profile, Plan & usage; the theme's three-way switch (system, light,
	// dark) that changes in place; the language, picked from its own list; Keyboard shortcuts; What's
	// new with the version; Log out. Keyboard like the platform's menus: arrows, Home and End move,
	// Enter picks, Escape closes (or goes back from the language list), Tab or a click away closes.
	// Left out on purpose: "Help & docs" (Nomi has no help site to open yet), and What's new's "new"
	// dot (there are no release notes to be new); the row shows the version, as the pen does.
	// Its button is the user row (UserRow's look, drawn here: the gear only shows that it opens), and
	// its links are Menu Items as links; UserRow and MenuRow keep the old app's twins' props.
	// `inline` shows the panel open in place (the gallery); `view` opens it on the language list.
	type Language = { value: string; label: string };
	type Props = {
		name: string;
		avatar?: string;
		/** "Free · 21.8K / 1M tokens". */
		plan?: string;
		/** Shown only when a bigger plan is offered. */
		upgradeHref?: string;
		settingsHref: string;
		profileHref: string;
		planHref: string;
		theme: Theme;
		ontheme: (theme: Theme) => void;
		language: string;
		languages: Language[];
		onlanguage: (language: string) => void;
		version: string;
		/** Opens the shortcuts dialog; without it the row isn't shown. */
		onshortcuts?: () => void;
		onlogout: () => void;
		inline?: boolean;
		view?: 'main' | 'language';
	};
	let { name, avatar, plan, upgradeHref, settingsHref, profileHref, planHref, theme, ontheme, language, languages, onlanguage, version, onshortcuts, onlogout, inline = false, view: initialView = 'main' }: Props = $props();

	const id = $props.id();
	const startView = () => initialView;
	let view = $state<'main' | 'language'>(startView());
	let open = $state(false);
	let anchor: HTMLElement | undefined = $state();
	let menu: HTMLElement | undefined = $state();
	let position = $state({ top: 0, left: 0 });

	const themes: { value: Theme; label: () => string; icon: typeof Monitor }[] = [
		{ value: 'system', label: m.theme_system, icon: Monitor },
		{ value: 'light', label: m.theme_light, icon: Sun },
		{ value: 'dark', label: m.theme_dark, icon: Moon },
	];
	const current = $derived(languages.find((option) => option.value === language)?.label ?? language);

	const links: { href: string; label: string; icon: Component<LucideProps>; shortcut?: string }[] = $derived([
		{ href: settingsHref, label: m.nav_settings(), icon: Settings2, shortcut: '⌘ ,' },
		{ href: profileHref, label: m.account_profile(), icon: User },
		{ href: planHref, label: m.nav_plan_usage(), icon: CreditCard },
	]);

	function place() {
		const box = anchor?.getBoundingClientRect();
		if (!box || !menu) return;
		position = placePanel(box, { width: menu.offsetWidth, height: menu.offsetHeight }, { width: window.innerWidth, height: window.innerHeight });
	}

	async function focusItem(which: 'first' | string) {
		await tick();
		const items = menuItems(menu);
		(which === 'first' ? items[0] : items.find((item) => item.dataset.item === which))?.focus();
	}

	async function show() {
		view = 'main';
		open = true;
		await tick();
		menu?.showPopover();
		place();
		await focusItem('first');
	}

	function hide(returnFocus: boolean) {
		open = false;
		if (returnFocus) anchor?.querySelector<HTMLElement>('button')?.focus();
	}

	async function showView(next: 'main' | 'language') {
		view = next;
		await tick();
		if (!inline) place();
		await focusItem(next === 'language' ? `language-${language}` : 'language');
	}

	function pick(item: MenuItem) {
		if (!inline) hide(true);
		item.onselect?.();
	}

	function onkeydown(event: KeyboardEvent) {
		const items = menuItems(menu);
		const at = items.indexOf(document.activeElement as HTMLElement);
		const next = nextIndex(event.key, at, items.length);
		if (next !== null) {
			event.preventDefault();
			items[next]?.focus();
		} else if (event.key === 'Escape') {
			event.preventDefault();
			if (view === 'language') void showView('main');
			else if (!inline) hide(true);
		} else if (event.key === 'Tab' && !inline) {
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

{#snippet divider()}<hr class="border-border" />{/snippet}

{#snippet panel()}
	<div
		bind:this={menu}
		id="{id}-menu"
		role="menu"
		tabindex="-1"
		aria-label={m.nav_account()}
		popover={inline ? undefined : 'manual'}
		{onkeydown}
		style={inline ? undefined : `top: ${position.top}px; left: ${position.left}px`}
		class={['flex w-68 flex-col gap-0.5 rounded-md bg-surface p-1 text-text shadow-float-lg outline -outline-offset-1 outline-border-strong', !inline && 'fixed m-0']}
	>
		<div role="none" class="flex items-center gap-2.5 border-b border-border px-2.5 pt-2.5 pb-2.75">
			<Avatar {name} src={avatar} />
			<span class="flex min-w-0 flex-1 flex-col gap-0.5">
				<span class="truncate text-ui font-medium text-text">{name}</span>
				{#if plan}<span class="font-mono text-label text-text-3">{plan}</span>{/if}
			</span>
			{#if upgradeHref}
				<a href={upgradeHref} role="menuitem" tabindex="-1" onclick={() => !inline && hide(false)} class="rounded-xs text-caption font-medium text-accent outline-offset-2 hover:underline focus-visible:underline">{m.account_upgrade()}</a>
			{/if}
		</div>

		{#if view === 'main'}
			{#each links as link (link.href)}
				<a
					href={link.href}
					role="menuitem"
					tabindex="-1"
					onclick={() => !inline && hide(false)}
					class="flex h-7.5 w-full cursor-pointer items-center gap-2 rounded-xs px-2 text-ui text-text outline-none hover:bg-surface-2 focus-visible:bg-surface-2 pointer-coarse:min-h-11"
				>
					<span class="inline-flex text-text-2"><Icon icon={link.icon} size={14} /></span>
					<span class="min-w-0 flex-1 truncate">{link.label}</span>
					{#if link.shortcut}<span class="font-mono text-label text-text-3">{link.shortcut}</span>{/if}
				</a>
			{/each}
			{@render divider()}
			<div role="group" aria-label={m.account_theme()} class="flex h-9 items-center gap-2 px-2">
				<span class="inline-flex text-text-2"><Icon icon={SunMoon} size={14} /></span>
				<span aria-hidden="true" class="min-w-0 flex-1 text-ui text-text">{m.account_theme()}</span>
				<span class="flex gap-0.5 rounded-sm bg-surface-2 p-0.5">
					{#each themes as option (option.value)}
						{@const checked = option.value === theme}
						<button
							type="button"
							role="menuitemradio"
							tabindex="-1"
							aria-checked={checked}
							aria-label={option.label()}
							onclick={() => ontheme(option.value)}
							class={[
								'inline-flex h-5.5 w-6.5 cursor-pointer items-center justify-center rounded-xs border outline-none focus-visible:border-focus pointer-coarse:min-h-11',
								checked ? 'border-border-strong bg-bg text-text' : 'border-transparent text-text-3 hover:text-text-2',
							]}
						>
							<Icon icon={option.icon} size={12} />
						</button>
					{/each}
				</span>
			</div>
			<button
				type="button"
				role="menuitem"
				tabindex="-1"
				aria-haspopup="menu"
				data-item="language"
				onclick={() => showView('language')}
				class="flex h-8 w-full cursor-pointer items-center gap-2 rounded-xs px-2 text-left text-ui text-text outline-none hover:bg-surface-2 focus-visible:bg-surface-2 pointer-coarse:min-h-11"
			>
				<span class="inline-flex text-text-2"><Icon icon={Languages} size={14} /></span>
				<span class="min-w-0 flex-1">{m.account_language()}</span>
				<span class="font-mono text-label text-text-2">{current}</span>
				<span class="inline-flex text-text-3"><Icon icon={ChevronRight} size={12} /></span>
			</button>
			{@render divider()}
			{#if onshortcuts}<MenuRow item={{ label: m.account_shortcuts(), icon: Keyboard, shortcut: '?', onselect: onshortcuts }} onpick={pick} />{/if}
			<div role="none" class="flex h-7.5 items-center gap-2 px-2">
				<span class="inline-flex text-text-2"><Icon icon={Sparkle} size={14} /></span>
				<span class="flex-1 text-ui whitespace-nowrap text-text">{m.account_whats_new()}</span>
				<span class="min-w-0 truncate font-mono text-label text-text-3">{version}</span>
			</div>
			{@render divider()}
			<MenuRow item={{ label: m.settings_logout(), icon: LogOut, onselect: onlogout }} onpick={pick} />
		{:else}
			<button
				type="button"
				role="menuitem"
				tabindex="-1"
				onclick={() => showView('main')}
				class="flex h-8 w-full cursor-pointer items-center gap-2 rounded-xs px-2 text-left text-ui text-text outline-none hover:bg-surface-2 focus-visible:bg-surface-2 pointer-coarse:min-h-11"
			>
				<span class="inline-flex text-text-2"><Icon icon={ChevronLeft} size={14} /></span>
				<span class="min-w-0 flex-1">{m.account_language()}</span>
			</button>
			{@render divider()}
			{#each languages as option (option.value)}
				{@const checked = option.value === language}
				<button
					type="button"
					role="menuitemradio"
					tabindex="-1"
					aria-checked={checked}
					data-item="language-{option.value}"
					onclick={() => {
						if (!inline) hide(false);
						if (!checked) onlanguage(option.value);
					}}
					class="flex h-7.5 w-full cursor-pointer items-center gap-2 rounded-xs px-2 text-left text-ui text-text outline-none hover:bg-surface-2 focus-visible:bg-surface-2 pointer-coarse:min-h-11"
				>
					<span class="min-w-0 flex-1">{option.label}</span>
					{#if checked}<span class="inline-flex text-text"><Icon icon={Check} size={14} /></span>{/if}
				</button>
			{/each}
		{/if}
	</div>
{/snippet}

{#if inline}
	{@render panel()}
{:else}
	<span bind:this={anchor} class="flex w-full">
		<button
			type="button"
			aria-label={name}
			aria-haspopup="menu"
			aria-expanded={open}
			aria-controls="{id}-menu"
			onclick={() => (open ? hide(true) : show())}
			class="flex w-full cursor-pointer items-center gap-2.5 rounded-sm px-2 py-1 text-left outline-offset-1 hover:bg-surface-2 aria-expanded:bg-surface-2 pointer-coarse:min-h-11"
		>
			<Avatar {name} src={avatar} />
			<span class="min-w-0 flex-1 truncate text-ui text-text">{name}</span>
			<span aria-hidden="true" class="inline-flex text-text"><Icon icon={Settings} /></span>
		</button>
	</span>
	{#if open}{@render panel()}{/if}
{/if}
