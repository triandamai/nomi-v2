<script lang="ts">
	import { page } from '$app/state';

	let { children } = $props();

	// ?theme=dark or ?theme=light shows the whole sheet in one theme, to compare with the pen file.
	const theme = $derived(page.url.searchParams.get('theme'));
	const sheets = [
		{ href: '/design-system/nomi', label: 'Nomi' },
		{ href: '/design-system/chat', label: 'Chat responses' },
		{ href: '/design-system/build', label: 'Build' },
	];
</script>

<svelte:head><title>Design system · Nomi</title></svelte:head>

<div data-theme={theme === 'dark' || theme === 'light' ? theme : undefined} class="min-h-screen bg-bg">
	<div class="mx-auto flex max-w-7xl flex-col px-12 py-12">
		<nav class="flex items-center gap-4 text-ui" aria-label="Design sheets">
			{#each sheets as sheet (sheet.href)}
				<a
					href={sheet.href}
					class="rounded-sm px-2 py-1 text-text-2 hover:bg-surface-2 hover:text-text"
					aria-current={page.url.pathname === sheet.href ? 'page' : undefined}>{sheet.label}</a
				>
			{/each}
			<span class="ml-auto flex gap-2 font-mono text-label text-text-3">
				<a class="hover:text-text" href="?theme=dark">dark</a>
				<a class="hover:text-text" href="?theme=light">light</a>
				<a class="hover:text-text" href="?">system</a>
			</span>
		</nav>
		{@render children()}
	</div>
</div>
