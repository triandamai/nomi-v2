<script lang="ts">
	import type { Component } from 'svelte';
	import { Activity, Bell, Bot, Brain, ChartColumn, Construction, Cpu, CreditCard, LayoutDashboard, LifeBuoy, Plug, Ship, SlidersHorizontal, Users, Wrench, type LucideProps } from '@lucide/svelte';
	import { navigating, page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { VERSION_LABEL } from '$lib/version';
	import type { Theme } from '$lib/theme';
	import { navGroups, pageForPath, type PageKey } from '$lib/admin/access';
	import { GROUP_LABELS, PAGE_LABELS, levelLabel } from '$lib/admin/labels';
	import AppShell from '$lib/ds/shells/AppShell.svelte';
	import FocusShell from '$lib/ds/shells/FocusShell.svelte';
	import AdminNav from '$lib/ds/admin/AdminNav.svelte';
	import AccountMenu from '$lib/ds/chrome/AccountMenu.svelte';
	import TopBar from '$lib/ds/chrome/TopBar.svelte';
	import ToastRegion from '$lib/ds/overlays/ToastRegion.svelte';
	import SkeletonRow from '$lib/ds/lists/SkeletonRow.svelte';
	import { toasts } from '$lib/ds/overlays/toasts.svelte';

	// The admin console's shell. The pen draws no admin screens, so it is the design system's App
	// shell (AGENTS.md §2) with a SettingsNav-style sidebar of the areas the person may open (AdminNav),
	// the v2 brand marked "admin", a way back to the app and the person's account menu at its foot;
	// the top bar names the page and the person's level there. Someone with no admin area at all
	// sees the gate's no-access page on its own, in the focus shell.
	let { data, children } = $props();

	const ICONS: Record<PageKey, Component<LucideProps>> = {
		overview: LayoutDashboard,
		live: Activity,
		usage: ChartColumn,
		users: Users,
		plans: CreditCard,
		notifications: Bell,
		models: Cpu,
		providers: Plug,
		embedding: Brain,
		tools: Wrench,
		dynamicAgents: Bot,
		support: LifeBuoy,
		maintenance: Construction,
		integrations: Ship,
		configuration: SlidersHorizontal,
	};

	const groups = $derived(
		navGroups(data.access).map((group) => ({
			label: GROUP_LABELS[group.key](),
			items: group.pages.map((each) => ({ href: each.href, icon: ICONS[each.key], label: PAGE_LABELS[each.key]() })),
		})),
	);
	const current = $derived(pageForPath(page.url.pathname));
	const level = $derived(current ? data.access[current.area] : undefined);
	// A page inside another names itself in its load ({ title }): a plan, a person.
	const title = $derived((page.data as { title?: string }).title ?? (current ? PAGE_LABELS[current.key]() : m.admin_title()));

	// Loading another page: after a moment, say which one is on its way in place of this one.
	const going = $derived(navigating.to && navigating.to.url.pathname !== page.url.pathname ? pageForPath(navigating.to.url.pathname) : null);
	let slow = $state(false);
	$effect(() => {
		if (!going) {
			slow = false;
			return;
		}
		const timer = setTimeout(() => (slow = true), 250);
		return () => clearTimeout(timer);
	});

	let theme = $derived<Theme>(data.theme);
	let logout: HTMLFormElement | undefined = $state();
	const language = getLocale();
	const languages = [
		{ value: 'en', label: m.language_en() },
		{ value: 'id', label: m.language_id() },
	];

	async function send(path: string, body: unknown): Promise<boolean> {
		const response = await fetch(path, { method: 'PUT', headers: { 'content-type': 'application/json' }, body: JSON.stringify(body) }).catch(() => null);
		return Boolean(response?.ok && !response.redirected);
	}

	function applyTheme(next: Theme) {
		if (next === 'system') delete document.documentElement.dataset.theme;
		else document.documentElement.dataset.theme = next;
	}

	async function changeTheme(next: Theme) {
		const before = theme;
		theme = next;
		applyTheme(next);
		if (await send('/preferences', { theme: next })) return;
		theme = before;
		applyTheme(before);
		toasts.show({ kind: 'error', title: m.theme_failed() });
	}

	async function changeLanguage(next: string) {
		if (await send('/preferences', { language: next })) location.reload();
		else toasts.show({ kind: 'error', title: m.language_failed() });
	}
</script>

<svelte:head><title>{title} · {m.admin_title()} · {m.app_name()}</title></svelte:head>

{#snippet sidebar()}
	<AdminNav label={m.admin_nav()} mark={m.admin_mark()} homeHref="/admin" backHref="/" backLabel={m.admin_back_to_app()} {groups} active={current?.href}>
		{#snippet footer()}
			<AccountMenu
				name={data.person.name}
				avatar={data.person.avatar ?? undefined}
				settingsHref="/settings"
				profileHref="/settings/profile"
				planHref="/settings/plan"
				{theme}
				ontheme={changeTheme}
				{language}
				{languages}
				onlanguage={changeLanguage}
				version={VERSION_LABEL}
				onlogout={() => logout?.requestSubmit()}
			/>
		{/snippet}
	</AdminNav>
{/snippet}

{#snippet topbar()}
	<TopBar {title} meta={level ? m.admin_level_meta({ level: levelLabel(level).toLowerCase() }) : undefined} />
{/snippet}

{#if groups.length > 0}
	<AppShell {sidebar} {topbar}>
		<div class="mx-auto flex w-full max-w-300 flex-col gap-6 pb-8">
			{#if going && slow}
				<SkeletonRow label={m.admin_loading_page({ page: PAGE_LABELS[going.key]().toLowerCase() })} />
			{:else}
				{@render children()}
			{/if}
		</div>
	</AppShell>
{:else}
	<FocusShell>{@render children()}</FocusShell>
{/if}

<form bind:this={logout} method="POST" action="/sign-out" hidden></form>

<ToastRegion />
