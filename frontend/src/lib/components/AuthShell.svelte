<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import type { Snippet } from 'svelte';
	import { onMount } from 'svelte';
	import { fly } from 'svelte/transition';
	import { backOut } from 'svelte/easing';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import MorphingShape from '$lib/components/m3/MorphingShape.svelte';

	// Shared frame for login/register. Left: a dark stage where the agent crew lives — a large
	// morphing Nomi, the other agents' shapes drifting at different depths (with pointer
	// parallax), and a ticker of the kind of things they do. Right: the form. Below 900px the
	// stage becomes a shorter band above the form. All motion stops under reduced motion.

	let {
		title,
		subtitle,
		switchPrompt,
		switchLabel,
		switchHref,
		children,
	}: {
		title: string;
		subtitle: string;
		switchPrompt: string;
		switchLabel: string;
		switchHref: string;
		children: Snippet;
	} = $props();

	const ACTIVITY = [
		{ agent: 'money', name: 'Money', text: m.auth_tick_money() },
		{ agent: 'planning', name: 'Planning', text: m.auth_tick_planning() },
		{ agent: 'coding', name: 'Coding', text: m.auth_tick_coding() },
		{ agent: 'personality', name: 'Personality', text: m.auth_tick_personality() },
	];

	let activityIndex = $state(0);
	let px = $state(0);
	let py = $state(0);
	let reducedMotion = $state(false);

	onMount(() => {
		reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
		if (reducedMotion) return;
		const handle = setInterval(() => {
			activityIndex = (activityIndex + 1) % ACTIVITY.length;
		}, 3400);
		return () => clearInterval(handle);
	});

	function onPointerMove(event: PointerEvent) {
		if (reducedMotion || event.pointerType !== 'mouse') return;
		px = (event.clientX / window.innerWidth) * 2 - 1;
		py = (event.clientY / window.innerHeight) * 2 - 1;
	}

	const current = $derived(ACTIVITY[activityIndex]);
</script>

<div class="auth" style="--px: {px}; --py: {py}" onpointermove={onPointerMove} role="presentation">
	<aside class="stage" aria-hidden="true">
		<div class="stage__field">
			<span class="stage__shape stage__hero" style="--depth: 10">
				<MorphingShape
					size={560}
					tone="glow"
					face
					stepMs={2600}
					sequence={['cookie9', 'flower5', 'clover4', 'sunny8']}
					class="stage__hero-svg"
				/>
			</span>
			<span class="stage__shape stage__sky" style="--depth: 26"><AgentShape agent="planning" size={168} /></span>
			<span class="stage__shape stage__ember" style="--depth: 34"><AgentShape agent="money" size={112} /></span>
			<span class="stage__shape stage__bloom" style="--depth: 44"><AgentShape agent="personality" size={76} /></span>
			<span class="stage__shape stage__tide" style="--depth: 20"><AgentShape agent="coding" size={132} /></span>
		</div>

		<div class="stage__content">
			<a href="/login" class="stage__mark" tabindex="-1">
				<AgentShape size={44} face />
				<span class="stage__word">nomi</span>
			</a>
			<p class="stage__line">{m.auth_line()}</p>

			<div class="ticker">
				{#key activityIndex}
					<div class="ticker__card" in:fly={{ y: 24, duration: 520, easing: backOut }} out:fly={{ y: -16, duration: 220 }}>
						<AgentShape agent={current.agent} size={36} working />
						<span class="ticker__text">
							<span class="ticker__name">{current.name}</span>
							<span class="ticker__what">{current.text}</span>
						</span>
						<span class="ticker__time">{m.time_just_now()}</span>
					</div>
				{/key}
			</div>
		</div>
	</aside>

	<main class="panel">
		<div class="panel__top">
			<span class="panel__prompt">{switchPrompt}</span>
			<a href={switchHref} class="panel__switch">{switchLabel}</a>
		</div>

		<div class="panel__card">
			<div class="panel__heading">
				<h1 class="panel__title">{title}</h1>
				<p class="panel__subtitle">{subtitle}</p>
			</div>
			{@render children()}
		</div>

		<footer class="panel__footer">
			<span>© 2026</span>
			<span aria-hidden="true">·</span>
			<span class="panel__made">
				{m.auth_made_with()}
				<svg width="14" height="14" viewBox="0 0 24 24" aria-label={m.auth_love()} role="img"><path fill="currentColor" d="M12 21s-7.5-4.6-9.6-9.2C.9 8.4 3 4.5 6.8 4.5c2.1 0 3.6 1.1 5.2 3 1.6-1.9 3.1-3 5.2-3 3.8 0 5.9 3.9 4.4 7.3C19.5 16.4 12 21 12 21z" /></svg>
				{m.auth_by()} <a href="https://trian.space" target="_blank" rel="noopener">Trian</a>
			</span>
		</footer>
	</main>
</div>

<style>
	.auth {
		min-height: 100vh;
		min-height: 100dvh;
		display: flex;
		flex-wrap: wrap;
		background: var(--md-sys-color-surface);
	}

	/* ---- Stage ---- */
	.stage {
		position: relative;
		flex: 1 1 520px;
		min-height: calc(100vh - 24px);
		min-height: calc(100dvh - 24px);
		margin: 12px;
		border-radius: 40px 40px 40px 12px;
		background: var(--nomi-color-stage);
		color: var(--nomi-color-on-stage);
		overflow: hidden;
		isolation: isolate;
	}
	.stage__field {
		position: absolute;
		inset: 0;
		z-index: -1;
	}
	.stage__shape {
		position: absolute;
		display: block;
		/* Pointer parallax: nearer shapes (higher --depth) travel further. */
		translate: calc(var(--px) * var(--depth) * -1px) calc(var(--py) * var(--depth) * -1px);
		transition: translate 900ms cubic-bezier(0.22, 1, 0.36, 1);
	}
	.stage__shape > :global(svg) {
		display: block;
		animation: stage-float var(--float, 7s) ease-in-out infinite alternate;
	}
	.stage__hero {
		right: -14%;
		bottom: -16%;
		width: min(560px, 78%);
	}
	.stage__hero :global(.stage__hero-svg) {
		width: 100%;
		height: auto;
		animation: stage-turn 60s linear infinite;
	}
	.stage__sky {
		top: 5%;
		right: 4%;
		--float: 8s;
	}
	.stage__ember {
		left: 9%;
		top: 54%;
		--float: 6s;
	}
	.stage__bloom {
		left: 40%;
		top: 40%;
		--float: 5s;
	}
	.stage__tide {
		left: -3%;
		bottom: -4%;
		--float: 9s;
	}
	@keyframes stage-float {
		from {
			transform: translateY(-10px) rotate(-8deg);
		}
		to {
			transform: translateY(12px) rotate(10deg);
		}
	}
	@keyframes stage-turn {
		to {
			rotate: 360deg;
		}
	}

	.stage__content {
		position: relative;
		height: 100%;
		min-height: inherit;
		box-sizing: border-box;
		display: flex;
		flex-direction: column;
		gap: 28px;
		padding: clamp(28px, 4.5vw, 56px);
	}
	.stage__mark {
		display: flex;
		align-items: center;
		gap: 12px;
		color: inherit;
		text-decoration: none;
	}
	.stage__word {
		font-family: var(--md-ref-typeface-brand);
		font-size: 2.25rem;
		line-height: 1;
		font-weight: 800;
		letter-spacing: -0.045em;
	}
	.stage__line {
		margin: 0;
		max-width: 11ch;
		font-family: var(--md-ref-typeface-brand);
		font-size: clamp(2.5rem, 4.4vw, 4.5rem);
		line-height: 0.98;
		font-weight: 800;
		letter-spacing: -0.045em;
		text-wrap: balance;
	}

	/* Ticker: one agent update at a time, springing in from below. */
	.ticker {
		position: relative;
		margin-top: auto;
		height: 92px;
		max-width: 440px;
	}
	.ticker__card {
		position: absolute;
		inset: 0 auto auto 0;
		display: flex;
		align-items: center;
		gap: 12px;
		width: 100%;
		box-sizing: border-box;
		padding: 14px 18px 14px 14px;
		border-radius: 24px 24px 24px 8px;
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
		box-shadow: 0 18px 40px -20px rgba(0, 0, 0, 0.55);
	}
	.ticker__text {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-width: 0;
		line-height: 1.3;
	}
	.ticker__name {
		font-weight: 700;
		font-size: 0.9375rem;
	}
	.ticker__what {
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}
	.ticker__time {
		align-self: flex-start;
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.6875rem;
		letter-spacing: 0.04em;
		color: var(--md-sys-color-on-surface-variant);
		white-space: nowrap;
	}

	/* ---- Form panel ---- */
	.panel {
		flex: 1 1 440px;
		display: flex;
		flex-direction: column;
		padding: 20px clamp(16px, 4vw, 48px) 40px;
		box-sizing: border-box;
	}
	.panel__footer {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: center;
		gap: 6px;
		margin-top: auto;
		padding-top: 32px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.panel__made {
		display: inline-flex;
		align-items: center;
		gap: 4px;
	}
	.panel__made svg {
		color: var(--md-sys-color-tertiary);
	}
	.panel__footer a {
		color: var(--md-sys-color-primary);
		font-weight: 600;
		text-decoration: none;
	}
	.panel__footer a:hover {
		text-decoration: underline;
	}
	.panel__top {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 12px;
		min-height: 48px;
	}
	.panel__prompt {
		font-size: 0.9375rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.panel__switch {
		display: inline-flex;
		align-items: center;
		height: 44px;
		padding: 0 20px;
		border-radius: 22px;
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
		font-weight: 600;
		font-size: 0.9375rem;
		text-decoration: none;
		transition: border-radius var(--nomi-motion-spatial-fast);
	}
	.panel__switch:hover {
		border-radius: var(--md-sys-shape-corner-medium);
	}
	.panel__card {
		width: 100%;
		max-width: 420px;
		margin: auto;
		padding: 32px 0;
		display: flex;
		flex-direction: column;
		gap: 28px;
	}
	.panel__heading {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}
	.panel__title {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: clamp(2.5rem, 4vw, 3.25rem);
		line-height: 1;
		font-weight: 800;
		letter-spacing: -0.04em;
		color: var(--md-sys-color-on-surface);
	}
	.panel__subtitle {
		margin: 0;
		font-size: 1.0625rem;
		line-height: 1.5;
		color: var(--md-sys-color-on-surface-variant);
	}

	/* ---- Phone / narrow: the stage becomes a band above the form ---- */
	@media (max-width: 900px) {
		.stage {
			flex-basis: 100%;
			min-height: 320px;
		}
		.stage__hero {
			width: 300px;
			right: -70px;
			bottom: -90px;
		}
		.stage__sky :global(svg) {
			width: 88px;
			height: 88px;
		}
		.stage__ember :global(svg) {
			width: 60px;
			height: 60px;
		}
		.stage__bloom,
		.stage__tide,
		.stage__ember {
			display: none;
		}
		.stage__line {
			font-size: 2rem;
			max-width: 12ch;
		}
		.ticker {
			max-width: 100%;
		}
		.panel__card {
			margin-top: 8px;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.stage__shape,
		.stage__shape > :global(svg),
		.stage__hero :global(.stage__hero-svg) {
			animation: none;
			transition: none;
			translate: none;
		}
	}
</style>
