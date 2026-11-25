<script lang="ts">
	import { onMount } from 'svelte';
	import { initViewport } from '$lib/viewport';
	import '../app.css';
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { shareMeta } from '$lib/seo/seo';

	let { children } = $props();

	// What a shared link shows, on every page: the landing page's title and line with the crew's
	// image. Each page names its own tab (<title>); the icons are in app.html.
	const tags = $derived(
		shareMeta({
			origin: page.url.origin,
			pathname: page.url.pathname,
			title: m.landing_title(),
			description: m.landing_description(),
			siteName: m.app_name(),
			imageAlt: m.meta_image_alt(),
			locale: getLocale(),
		}),
	);

	onMount(() => {
		return initViewport();
	});
</script>

<svelte:head>
	{#each tags as tag (tag.name ?? tag.property)}
		<meta name={tag.name} property={tag.property} content={tag.content} />
	{/each}
</svelte:head>

{@render children()}
