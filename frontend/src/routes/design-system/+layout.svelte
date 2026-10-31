<script lang="ts">
	import { page } from '$app/state';

	let { children } = $props();

	// The sheets match frontend-new's /design-system: the primitives drawn in M3 here, then chat
	// responses and build with the twins copied as they are. Documentation, not app text.
	const sheets = [
		{ href: '/design-system', label: 'Primitives' },
		{ href: '/design-system/chat', label: 'Chat responses' },
		{ href: '/design-system/build', label: 'Build' },
	];
</script>

<svelte:head><title>Design system · Nomi</title></svelte:head>

<nav aria-label="Design sheets">
	{#each sheets as sheet (sheet.href)}
		<a href={sheet.href} aria-current={page.url.pathname === sheet.href ? 'page' : undefined}>{sheet.label}</a>
	{/each}
</nav>
{@render children()}

<style>
	:global(body) {
		background: var(--nomi-color-ground);
	}
	nav {
		display: flex;
		max-width: 1080px;
		flex-wrap: wrap;
		gap: 4px;
		margin: 0 auto;
		padding: 24px 16px 0;
	}
	a {
		display: inline-flex;
		min-height: 40px;
		align-items: center;
		padding: 0 14px;
		border-radius: var(--md-sys-shape-corner-full);
		color: var(--md-sys-color-on-surface-variant);
		font: var(--md-sys-typescale-label-large-weight) var(--md-sys-typescale-label-large-size) / var(--md-sys-typescale-label-large-line-height) var(--md-sys-typescale-label-large-font);
		text-decoration: none;
	}
	a:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
		color: var(--md-sys-color-on-surface);
	}
	a:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	a[aria-current='page'] {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	@media (pointer: coarse) {
		a {
			min-height: 44px;
		}
	}
</style>
