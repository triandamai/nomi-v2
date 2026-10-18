<script lang="ts">
	// A provider's monochrome logo, painted in the text colour through a CSS mask so it follows
	// the theme (and dark mode) without inlining someone else's SVG. Providers without a logo,
	// and custom endpoints, get a generic mark.
	import IconSparkle from '$lib/components/icons/IconSparkle.svelte';
	import { CUSTOM_PROVIDER_ID, logoUrl } from '$lib/providers';

	let { id, hasLogo = true, size = 20 }: { id: string; hasLogo?: boolean; size?: number } = $props();
</script>

{#if hasLogo && id !== CUSTOM_PROVIDER_ID}
	<span
		class="logo"
		style:width="{size}px"
		style:height="{size}px"
		style:--logo-url="url('{logoUrl(encodeURIComponent(id))}')"
		aria-hidden="true"
		data-logo={id}
	></span>
{:else}
	<span class="logo-fallback" aria-hidden="true"><IconSparkle {size} /></span>
{/if}

<style>
	.logo {
		display: inline-block;
		flex: none;
		background: currentColor;
		-webkit-mask: var(--logo-url) center / contain no-repeat;
		mask: var(--logo-url) center / contain no-repeat;
	}
	.logo-fallback {
		display: inline-flex;
		flex: none;
	}
</style>
