<!-- A page worth opening, as a preview: its site, title, description and image (read from the
     page on the server when the agent didn't give them). The whole card opens it. -->
<script lang="ts">
	import type { LinkView, ViewProps } from '$lib/views/types';

	let { view }: ViewProps<LinkView> = $props();

	const host = $derived.by(() => {
		try {
			return new URL(view.url).hostname.replace(/^www\./, '');
		} catch {
			return view.url;
		}
	});
	let imageFailed = $state(false);
</script>

<a class="link" href={view.url} target="_blank" rel="noopener noreferrer">
	<span class="link__text">
		<span class="link__site">{view.site ?? host}</span>
		<span class="link__title">{view.title ?? view.url}</span>
		{#if view.description}<span class="link__description">{view.description}</span>{/if}
	</span>
	{#if view.image && !imageFailed}
		<img class="link__image" src={view.image} alt="" loading="lazy" referrerpolicy="no-referrer" onerror={() => (imageFailed = true)} />
	{/if}
</a>

<style>
	.link {
		display: flex;
		align-items: stretch;
		gap: 0;
		min-width: 0;
		overflow: hidden;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
		box-shadow: inset 0 0 0 1px var(--md-sys-color-outline-variant);
		color: var(--md-sys-color-on-surface);
		text-decoration: none;
		transition: background-color var(--nomi-motion-effects-fast), border-radius var(--nomi-motion-spatial-fast);
	}
	.link:hover {
		background: var(--md-sys-color-surface-container-low);
		border-radius: var(--md-sys-shape-corner-large);
	}
	.link:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.link__text {
		display: flex;
		flex-direction: column;
		gap: 4px;
		flex: 1;
		min-width: 0;
		padding: 14px 16px;
	}
	.link__site {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.link__title {
		color: var(--md-sys-color-primary);
		font-weight: 700;
		line-height: 1.35;
		overflow-wrap: anywhere;
	}
	.link__description {
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
		line-height: 1.4;
	}
	.link__image {
		flex: none;
		width: 112px;
		object-fit: cover;
		background: var(--md-sys-color-surface-container-high);
	}
	@media (max-width: 420px) {
		.link__image {
			width: 84px;
		}
	}
</style>
