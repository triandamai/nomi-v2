<script lang="ts">
	// The showcase: live apps people built with Nomi and chose to show (from their project's
	// settings). Public, so it works signed in or not. Each card frames the live app itself, small
	// and inert, and opens it in a new tab.
	import { m } from '$lib/paraglide/messages';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import IconOpenInNew from '$lib/components/icons/IconOpenInNew.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	const host = (url: string) => {
		try {
			return new URL(url).host;
		} catch {
			return url;
		}
	};
	const STACKS: Record<string, string> = { sveltekit: 'SvelteKit', nextjs: 'Next.js', react: 'React', vue: 'Vue', static: 'Static site' };
	const stack = (id: string) => STACKS[id] ?? id;
</script>

<svelte:head>
	<title>{m.showcase_title()} · Nomi</title>
	<meta name="description" content={m.showcase_lede()} />
</svelte:head>

<div class="showcase">
	<header class="top">
		<a href="/" class="brand" aria-label={m.nav_nomi_home()}>
			<AgentShape size={32} face />
			<span class="brand__word">nomi</span>
		</a>
		{#if data.signedIn}
			<Button variant="tonal" size="s" href="/">{m.showcase_open_nomi()}</Button>
		{:else}
			<Button variant="text" size="s" href="/login">{m.showcase_log_in()}</Button>
		{/if}
	</header>

	<main class="main">
		<section class="hero">
			<h1 class="hero__title">{m.showcase_title()}</h1>
			<p class="hero__lede">{m.showcase_lede()}</p>
		</section>

		{#if data.apps.length === 0}
			<div class="empty">
				<AgentShape agent="coding" size={72} />
				<p class="empty__title">{data.failed ? m.showcase_failed() : m.showcase_empty()}</p>
				<p class="empty__hint">{m.showcase_empty_hint()}</p>
			</div>
		{:else}
			<ul class="grid">
				{#each data.apps as app (app.id)}
					<li>
						<a class="card" href={app.url} target="_blank" rel="noopener noreferrer" aria-label={m.showcase_open_app({ name: app.name })}>
							<div class="card__screen" aria-hidden="true">
								<iframe src={app.url} title="" tabindex="-1" loading="lazy" sandbox="allow-scripts" referrerpolicy="no-referrer"></iframe>
							</div>
							<div class="card__body">
								<span class="card__title">
									<span class="card__name">{app.name}</span>
									<IconOpenInNew size={18} />
								</span>
								{#if app.blurb}<span class="card__blurb">{app.blurb}</span>{/if}
								<span class="card__meta">{host(app.url)} · {app.team ? `${app.team} · ` : ''}{stack(app.stack)}</span>
							</div>
						</a>
					</li>
				{/each}
			</ul>
		{/if}

		<section class="cta">
			<p class="cta__line">{m.showcase_cta_line()}</p>
			<Button variant="gradient" size="m" href={data.signedIn ? '/' : '/register'}>{m.showcase_cta()}</Button>
		</section>
	</main>
</div>

<style>
	.showcase {
		min-height: 100dvh;
		background: var(--nomi-color-ground);
		color: var(--md-sys-color-on-surface);
	}
	.top {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		max-width: 1200px;
		margin: 0 auto;
		padding: 16px;
	}
	.brand {
		display: inline-flex;
		align-items: center;
		gap: 10px;
		min-height: 44px;
		border-radius: var(--md-sys-shape-corner-full);
		color: inherit;
		text-decoration: none;
	}
	.brand:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 4px;
	}
	.brand__word {
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.625rem;
		font-weight: 800;
		letter-spacing: -0.04em;
	}
	.main {
		display: flex;
		flex-direction: column;
		gap: 32px;
		max-width: 1200px;
		margin: 0 auto;
		padding: 8px 16px 48px;
	}
	.hero {
		display: flex;
		flex-direction: column;
		gap: 10px;
		max-width: 60ch;
		padding-top: 16px;
	}
	.hero__title {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: clamp(2.25rem, 7vw, 3.5rem);
		font-weight: 800;
		line-height: 1.05;
		letter-spacing: -0.04em;
	}
	.hero__lede {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 1.0625rem;
		line-height: 1.55;
	}
	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(min(100%, 300px), 1fr));
		gap: 16px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.card {
		display: flex;
		flex-direction: column;
		height: 100%;
		overflow: hidden;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
		color: inherit;
		text-decoration: none;
		transition: transform var(--nomi-motion-spatial-fast, 200ms) ease, box-shadow 200ms ease;
	}
	.card:hover {
		transform: translateY(-2px);
		box-shadow: var(--md-sys-elevation-level2);
	}
	.card:focus-visible {
		outline: 3px solid var(--md-sys-color-primary);
		outline-offset: 3px;
	}
	/* The live app at a quarter size: framed four times the card's width, then scaled down. */
	.card__screen {
		position: relative;
		aspect-ratio: 16 / 10;
		overflow: hidden;
		background: var(--md-sys-color-surface-container-low);
	}
	.card__screen iframe {
		position: absolute;
		top: 0;
		left: 0;
		width: 400%;
		height: 400%;
		border: 0;
		transform: scale(0.25);
		transform-origin: 0 0;
		pointer-events: none;
	}
	.card__body {
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: 14px 18px 18px;
	}
	.card__title {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		color: var(--md-sys-color-on-surface-variant);
	}
	.card__name {
		overflow: hidden;
		color: var(--md-sys-color-on-surface);
		font-size: 1.0625rem;
		font-weight: 700;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.card__blurb {
		display: -webkit-box;
		overflow: hidden;
		color: var(--md-sys-color-on-surface);
		font-size: 0.9375rem;
		line-height: 1.45;
		-webkit-box-orient: vertical;
		-webkit-line-clamp: 2;
		line-clamp: 2;
	}
	.card__meta {
		overflow: hidden;
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		padding: 48px 16px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
		text-align: center;
	}
	.empty__title {
		margin: 8px 0 0;
		font-size: 1.125rem;
		font-weight: 700;
	}
	.empty__hint {
		margin: 0;
		max-width: 48ch;
		color: var(--md-sys-color-on-surface-variant);
	}
	.cta {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: 12px 24px;
		padding: 24px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
	}
	.cta__line {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.375rem;
		font-weight: 800;
		letter-spacing: -0.02em;
	}
	@media (prefers-reduced-motion: reduce) {
		.card,
		.card:hover {
			transition: none;
			transform: none;
		}
	}
</style>
