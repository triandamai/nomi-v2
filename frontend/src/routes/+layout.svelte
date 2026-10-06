<script lang="ts">
	import './layout.css';
	import { page } from '$app/state';
	import { pageTitle } from '$lib/pageTitle';

	let { children } = $props();

	// What a link to Nomi shows when it's shared in WhatsApp, Telegram, Slack, iMessage and the
	// like. Crawlers need absolute URLs; the origin is the deployment's own (ORIGIN on the server).
	const TITLE = 'Nomi';
	const DESCRIPTION = 'A small crew of agents that remembers you.';
	const origin = $derived(page.url.origin);
</script>

<svelte:head>
	<title>{pageTitle(page.route.id, page.data.title)}</title>
	<link rel="icon" href="/favicon.svg" type="image/svg+xml" />
	<link rel="icon" href="/favicon-48.png" type="image/png" sizes="48x48" />
	<link rel="apple-touch-icon" href="/apple-touch-icon.png" />
	<link rel="manifest" href="/site.webmanifest" />
	<meta name="theme-color" content="#f4f7f2" media="(prefers-color-scheme: light)" />
	<meta name="theme-color" content="#0d1511" media="(prefers-color-scheme: dark)" />
	<meta name="description" content={DESCRIPTION} />

	<meta property="og:type" content="website" />
	<meta property="og:site_name" content={TITLE} />
	<meta property="og:title" content={TITLE} />
	<meta property="og:description" content={DESCRIPTION} />
	<meta property="og:url" content="{origin}{page.url.pathname}" />
	<meta property="og:image" content="{origin}/og-image.png" />
	<meta property="og:image:type" content="image/png" />
	<meta property="og:image:width" content="1200" />
	<meta property="og:image:height" content="630" />
	<meta property="og:image:alt" content="Nomi's green shape with two eyes, beside the words: A small crew of agents that remembers you." />
	<meta name="twitter:card" content="summary_large_image" />
	<meta name="twitter:title" content={TITLE} />
	<meta name="twitter:description" content={DESCRIPTION} />
	<meta name="twitter:image" content="{origin}/og-image.png" />
</svelte:head>
{@render children()}
