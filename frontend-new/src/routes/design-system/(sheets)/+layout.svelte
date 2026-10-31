<script lang="ts">
	import { building } from '$app/environment';
	import { page } from '$app/state';
	import { docHref } from '$lib/docs/href';

	let { children } = $props();

	// ?theme=dark or ?theme=light shows the whole sheet in one theme, to compare with the pen file.
	// (Prerendering has no query string: the static gallery reads it once it runs in the browser.)
	// Laid out like the pen's sheets: 64 around, 64 between the header and each section.
	const theme = $derived(building ? null : page.url.searchParams.get('theme'));
	// The static gallery is plain files: its sheets are .html pages next to each other, reached
	// relative to wherever it's hosted.
	const sheets = [
		{ key: 'nomi', label: 'Nomi' },
		{ key: 'chat', label: 'Chat responses' },
		{ key: 'build', label: 'Build' },
	].map((sheet) => ({ ...sheet, href: docHref(`/design-system/${sheet.key}`) }));
</script>

<svelte:head><title>Design system · Nomi</title></svelte:head>

<div data-theme={theme === 'dark' || theme === 'light' ? theme : undefined} class="min-h-screen bg-bg">
	<div class="mx-auto flex max-w-400 flex-col gap-16 p-16 max-md:px-4">
		<nav class="flex items-center gap-4 text-ui" aria-label="Design sheets">
			<a href={docHref('/design-system')} data-sveltekit-reload={__DS_STATIC__ || undefined} class="rounded-sm px-2 py-1 text-text-3 hover:bg-surface-2 hover:text-text">Docs</a>
			{#each sheets as sheet (sheet.href)}
				<a
					href={sheet.href}
					data-sveltekit-reload={__DS_STATIC__ || undefined}
					class="rounded-sm px-2 py-1 text-text-2 hover:bg-surface-2 hover:text-text"
					aria-current={!building && page.url.pathname.replace(/\.html$/, '').endsWith(`/design-system/${sheet.key}`) ? 'page' : undefined}>{sheet.label}</a
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
