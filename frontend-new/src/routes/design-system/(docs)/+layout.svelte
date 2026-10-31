<script lang="ts">
	import { onMount } from 'svelte';
	import { Menu } from '@lucide/svelte';
	import { afterNavigate } from '$app/navigation';
	import { page } from '$app/state';
	import Icon from '$lib/ds/foundations/Icon.svelte';
	import Mascot from '$lib/ds/agents/Mascot.svelte';
	import Segmented from '$lib/ds/inputs/Segmented.svelte';
	import DocsNav, { type DocsGroup } from '$lib/docs/ui/DocsNav.svelte';
	import DocsSearch, { type SearchEntry } from '$lib/docs/ui/DocsSearch.svelte';
	import { guides } from '$lib/docs/nav';
	import { CATEGORIES } from '$lib/docs/catalog';
	import { api, components } from '$lib/docs/components';
	import { componentDocs } from '$lib/docs/entries';
	import { PATTERNS, SHEETS } from '$lib/docs/patterns';
	import { docHref } from '$lib/docs/href';

	let { children } = $props();

	// The docs for everyone who makes Nomi: guides, foundations, every component and pattern, the
	// tokens. Built from the real components, so what it shows is what ships.
	const here = $derived(page.url.pathname.replace(/\.html$/, ''));
	const link = (title: string, path: string) => ({ title, href: docHref(path), active: here.endsWith(path) });
	const summary = (file: string) => componentDocs[file]?.summary ?? api[file]?.notes.split(/(?<=\.)\s/)[0] ?? '';

	const groups: DocsGroup[] = $derived([
		...guides.slice(0, 3).map((group) => ({ title: group.title, links: group.links.map((item) => link(item.title, item.path)) })),
		{
			title: 'Components',
			links: [link('Overview', '/design-system/components')],
			folders: CATEGORIES.map((category) => ({
				title: category.title,
				links: components.filter((item) => item.category === category.key).map((item) => link(item.name, `/design-system/components/${item.slug}`)),
			})),
		},
		{
			title: 'Patterns',
			links: [link('Overview', '/design-system/patterns')],
			folders: (['chat', 'build'] as const).map((sheet) => ({
				title: SHEETS[sheet],
				links: PATTERNS.filter((pattern) => pattern.sheet === sheet).map((pattern) => link(pattern.title, `/design-system/patterns/${sheet}/${pattern.id}`)),
			})),
		},
		...guides.slice(3).map((group) => ({ title: group.title, links: group.links.map((item) => link(item.title, item.path)) })),
		{ title: 'Pen sheets', links: [link('Nomi', '/design-system/nomi'), link('Chat responses', '/design-system/chat'), link('Build', '/design-system/build')] },
	]);

	const entries: SearchEntry[] = [
		...guides.flatMap((group) => group.links.map((item) => ({ title: item.title, group: group.title, href: docHref(item.path), blurb: item.blurb }))),
		...components.map((item) => ({
			title: item.name,
			group: CATEGORIES.find((category) => category.key === item.category)!.title,
			href: docHref(`/design-system/components/${item.slug}`),
			blurb: summary(item.file),
		})),
		...PATTERNS.map((pattern) => ({ title: pattern.title, group: `Pattern · ${SHEETS[pattern.sheet]}`, href: docHref(`/design-system/patterns/${pattern.sheet}/${pattern.id}`) })),
		...(['nomi', 'chat', 'build'] as const).map((sheet) => ({ title: `${sheet === 'nomi' ? 'Nomi' : SHEETS[sheet]} sheet`, group: 'Pen sheets', href: docHref(`/design-system/${sheet}`) })),
	];

	const top = [
		['Get started', '/design-system/get-started'],
		['Foundations', '/design-system/foundations/principles'],
		['Content', '/design-system/content/voice-and-tone'],
		['Components', '/design-system/components'],
		['Patterns', '/design-system/patterns'],
		['Tokens', '/design-system/tokens'],
	] as const;
	const section = (path: string) => here.includes(path.split('/').slice(0, 3).join('/'));

	// System, dark or light, remembered on this device. ?theme= (as on the sheets) sets it too.
	let menu = $state(false);
	afterNavigate(() => (menu = false));
	let theme = $state('system');
	const themes = [
		{ value: 'system', label: 'System' },
		{ value: 'dark', label: 'Dark' },
		{ value: 'light', label: 'Light' },
	];
	onMount(() => {
		document.querySelector('aside [aria-current="page"]')?.scrollIntoView({ block: 'center' });
		try {
			theme = new URL(location.href).searchParams.get('theme') ?? localStorage.getItem('nomi-ds-theme') ?? 'system';
		} catch {
			theme = 'system';
		}
	});
	$effect(() => {
		const root = document.documentElement;
		if (theme === 'dark' || theme === 'light') root.dataset.theme = theme;
		else delete root.dataset.theme;
		try {
			localStorage.setItem('nomi-ds-theme', theme);
		} catch {
			// Storage can be off (private windows, previews): the choice lasts for this page.
		}
		return () => delete root.dataset.theme;
	});
</script>

<div class="min-h-screen bg-bg text-text" data-sveltekit-reload={__DS_STATIC__ || undefined}>
	<header class="sticky top-0 z-20 flex h-13 items-center gap-4 border-b border-border bg-bg px-4 lg:px-6">
		<a href={docHref('/design-system')} class="flex shrink-0 items-center gap-2.5">
			<Mascot agent="nomi" size={20} />
			<span class="text-ui font-medium text-text">Nomi</span>
			<span class="font-mono text-label tracking-label text-text-3 max-sm:hidden">DESIGN SYSTEM</span>
		</a>
		<nav aria-label="Docs sections" class="ml-4 hidden items-center gap-1 xl:flex">
			{#each top as [title, path] (path)}
				<a href={docHref(path)} class={['rounded-sm px-2.5 py-1.5 text-ui hover:bg-surface-2 hover:text-text', section(path) ? 'text-text' : 'text-text-2']}>{title}</a>
			{/each}
		</nav>
		<div class="ml-auto flex items-center gap-3">
			<div class="w-72 max-md:hidden"><DocsSearch {entries} /></div>
			<span class="max-sm:hidden"><Segmented label="Theme" bind:value={theme} options={themes} /></span>
			<details class="relative lg:hidden" bind:open={menu}>
				<summary class="menu flex size-7.5 cursor-pointer list-none items-center justify-center rounded-sm border border-border-strong text-text-2" aria-label="Menu">
					<Icon icon={Menu} size={16} />
				</summary>
				{#if menu}
					<div class="menu-panel fixed inset-x-0 top-13 overflow-y-auto border-b border-border bg-bg px-4 pb-6">
						<div class="flex flex-col gap-3 py-3 md:hidden">
							<DocsSearch {entries} />
							<span class="sm:hidden"><Segmented label="Theme" bind:value={theme} options={themes} /></span>
						</div>
						<DocsNav {groups} />
					</div>
				{/if}
			</details>
		</div>
	</header>
	<div class="mx-auto flex max-w-400">
		<aside class="aside sticky top-13 hidden w-62 shrink-0 overflow-y-auto border-r border-border px-3 pt-3 pb-10 lg:block">
			<DocsNav {groups} />
		</aside>
		<main class="flex min-w-0 flex-1 justify-center px-4 pt-10 pb-24 md:px-10 xl:px-16">
			{@render children()}
		</main>
	</div>
</div>

<style>
	.aside {
		height: calc(100dvh - 52px);
	}
	.menu-panel {
		max-height: calc(100dvh - 52px);
	}
	.menu::-webkit-details-marker {
		display: none;
	}
</style>
