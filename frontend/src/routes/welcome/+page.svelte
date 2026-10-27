<script lang="ts">
	// The landing page: what someone sees the first time they open Nomi. The hero turns through
	// three scenes — Nomi's crew building an app, remembering what you told it, and getting
	// everyday work done — each a headline and a short chat that plays on the dark stage. Picking
	// a scene stops the turning. Then how it works, the crew, a few live apps and a last way in.
	// Under reduced motion the first scene shows whole and only changes when picked.
	import { m } from '$lib/paraglide/messages';
	import { onMount } from 'svelte';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import MorphingShape from '$lib/components/m3/MorphingShape.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import IconOpenInNew from '$lib/components/icons/IconOpenInNew.svelte';
	import { crewName } from '$lib/crewNames';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	// Time between one chat line and the next while a scene plays.
	const STEP_MS = 1100;
	// A line that shows its work first ("Building it…") stays that way this long.
	const BUSY_MS = 1800;
	// How long a finished scene stays before the next one starts.
	const HOLD_MS = 3200;

	type Line = { from: string; text: () => string; busy?: () => string };
	type Scene = { name: () => string; title: () => string; result: () => string; resultAgent: string; chat: Line[] };
	const SCENES: Scene[] = [
		{
			name: m.landing_scene_build,
			title: m.landing_title,
			result: m.landing_chat_live,
			resultAgent: 'coding',
			chat: [
				{ from: 'you', text: m.landing_chat_you_ask },
				{ from: 'planning', text: m.landing_chat_rena_plan },
				{ from: 'you', text: m.landing_chat_you_go },
				{ from: 'coding', text: m.landing_chat_koda_built, busy: m.landing_chat_koda_building },
				{ from: 'inspector', text: m.landing_chat_iris_checked },
			],
		},
		{
			name: m.landing_scene_remember,
			title: m.landing_title_remember,
			result: m.landing_remember_result,
			resultAgent: 'chitchat',
			chat: [
				{ from: 'you', text: m.landing_remember_you_tell },
				{ from: 'chitchat', text: m.landing_remember_nomi },
				{ from: 'you', text: m.landing_remember_you_ask },
				{ from: 'planning', text: m.landing_remember_rena, busy: m.landing_remember_rena_busy },
				{ from: 'reminders', text: m.landing_remember_kala },
			],
		},
		{
			name: m.landing_scene_work,
			title: m.landing_title_work,
			result: m.landing_work_result,
			resultAgent: 'money',
			chat: [
				{ from: 'you', text: m.landing_work_you_ask },
				{ from: 'money', text: m.landing_work_dana, busy: m.landing_work_dana_busy },
				{ from: 'you', text: m.landing_work_you_next },
				{ from: 'money', text: m.landing_work_dana_budget },
				{ from: 'workspace', text: m.landing_work_tara },
			],
		},
	];

	// The scene showing (and the one before, rolling out), how many of its lines are in, whether
	// the newest is still busy, and whether its result has landed. Server-rendered and without
	// JavaScript, the first scene shows whole.
	let scene = $state(0);
	let leaving = $state<number | null>(null);
	let shown = $state(SCENES[0].chat.length);
	let busy = $state(false);
	let finished = $state(true);
	let turning = true;
	let reducedMotion = false;
	let timers: ReturnType<typeof setTimeout>[] = [];

	function stop() {
		timers.forEach(clearTimeout);
		timers = [];
	}

	function play(index: number) {
		stop();
		if (index !== scene) leaving = scene;
		scene = index;
		const lines = SCENES[index].chat;
		if (reducedMotion) {
			shown = lines.length;
			busy = false;
			finished = true;
			return;
		}
		shown = 0;
		busy = false;
		finished = false;
		let at = STEP_MS / 2;
		lines.forEach((line, i) => {
			timers.push(setTimeout(() => {
				shown = i + 1;
				busy = Boolean(line.busy);
			}, at));
			if (line.busy) {
				at += BUSY_MS;
				timers.push(setTimeout(() => (busy = false), at));
			}
			at += STEP_MS;
		});
		timers.push(setTimeout(() => (finished = true), at - STEP_MS / 2));
		if (turning) timers.push(setTimeout(() => play((index + 1) % SCENES.length), at + HOLD_MS));
	}

	function pick(index: number) {
		turning = false;
		play(index);
	}

	onMount(() => {
		reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
		if (reducedMotion) turning = false;
		else play(0);
		return stop;
	});

	const current = $derived(SCENES[scene]);

	// A headline's sentences, each starting its own line ("Tell Nomi once." / "It remembers.").
	const sentences = (title: string) => title.split(/(?<=[.!?])\s+/);

	const STEPS = [
		{ title: m.landing_step_describe, body: m.landing_step_describe_body },
		{ title: m.landing_step_plan, body: m.landing_step_plan_body },
		{ title: m.landing_step_build, body: m.landing_step_build_body },
		{ title: m.landing_step_publish, body: m.landing_step_publish_body },
	];

	const CREW = [
		{ agent: 'chitchat', does: m.landing_crew_nomi },
		{ agent: 'planning', does: m.landing_crew_rena },
		{ agent: 'coding', does: m.landing_crew_koda },
		{ agent: 'inspector', does: m.landing_crew_iris },
		{ agent: 'money', does: m.landing_crew_dana },
		{ agent: 'reminders', does: m.landing_crew_kala },
		{ agent: 'workspace', does: m.landing_crew_tara },
	];

	const host = (url: string) => {
		try {
			return new URL(url).host;
		} catch {
			return url;
		}
	};
</script>

{#snippet lines(chat: Line[], count: number, live: boolean)}
	<ol class="chat__lines">
		{#each chat.slice(0, count) as line, i (`${scene}-${i}`)}
			{#if line.from === 'you'}
				<li class="bubble bubble--you">{line.text()}</li>
			{:else}
				{@const working = live && busy && i === count - 1}
				<li class="bubble bubble--agent">
					<AgentShape agent={line.from} size={28} face {working} />
					<span class="bubble__body">
						<span class="bubble__name">{crewName(line.from)}</span>
						<span>{working && line.busy ? line.busy() : line.text()}</span>
					</span>
				</li>
			{/if}
		{/each}
	</ol>
{/snippet}

<div class="page">
	<header class="top">
		<a href="/welcome" class="brand" aria-label={m.nav_nomi_home()}>
			<AgentShape size={32} face />
			<span class="brand__word">nomi</span>
		</a>
		<nav class="top__nav" aria-label={m.landing_nav()}>
			<a class="top__link" href="/showcase">{m.nav_showcase()}</a>
			{#if data.signedIn}
				<Button variant="tonal" size="s" href="/">{m.showcase_open_nomi()}</Button>
			{:else}
				<a class="top__link" href="/login">{m.landing_sign_in()}</a>
				<Button variant="tonal" size="s" href="/register">{m.landing_create_account()}</Button>
			{/if}
		</nav>
	</header>

	<main>
		<section class="stage" aria-labelledby="landing-title">
			<div class="stage__field" aria-hidden="true">
				<span class="stage__nomi">
					<MorphingShape size={520} tone="glow" stepMs={2600} sequence={['cookie9', 'flower5', 'clover4', 'sunny8']} class="stage__nomi-svg" />
				</span>
			</div>

			<div class="stage__copy">
				<h1 id="landing-title" class="stage__title">
					{#each SCENES as each, i (i)}
						<span class="stage__line" class:stage__line--on={i === scene} class:stage__line--out={i === leaving && i !== scene} aria-hidden={i !== scene}>
							{#each sentences(each.title()) as sentence, j (j)}<span class="stage__sentence">{sentence}</span>{' '}{/each}
						</span>
					{/each}
				</h1>
				<p class="stage__lede">{m.landing_lede()}</p>
				<div class="stage__actions">
					{#if data.signedIn}
						<Button variant="gradient" size="m" href="/">{m.showcase_open_nomi()}</Button>
					{:else}
						<Button variant="gradient" size="m" href="/register">{m.landing_create_account()}</Button>
						<a class="stage__signin" href="/login">{m.landing_have_account()}</a>
					{/if}
				</div>
			</div>

			<figure class="chat" aria-label={m.landing_chat_label()}>
				<div class="scenes" role="group" aria-label={m.landing_scenes_label()}>
					{#each SCENES as each, i (i)}
						<button type="button" class="scenes__pick" aria-pressed={i === scene} onclick={() => pick(i)}>{each.name()}</button>
					{/each}
				</div>
				<!-- Every scene's whole chat sits hidden in the same cell as the one playing, so the panel
				     is as tall as the longest at any width and nothing below it moves. -->
				<div class="chat__body">
					{#each SCENES as each, i (i)}
						<div class="chat__play chat__play--sizer" aria-hidden="true" inert>
							{@render lines(each.chat, each.chat.length, false)}
							<p class="chat__result">{each.result()}</p>
						</div>
					{/each}
					<div class="chat__play">
						{@render lines(current.chat, shown, true)}
						<p class="chat__result" class:chat__result--on={finished} aria-hidden={!finished}>
							<AgentShape agent={current.resultAgent} size={20} />
							<span>{current.result()}</span>
						</p>
					</div>
				</div>
			</figure>
		</section>

		<section class="section" aria-labelledby="how-title">
			<h2 id="how-title" class="section__title">{m.landing_how_title()}</h2>
			<ol class="steps">
				{#each STEPS as step, i (i)}
					<li class="step">
						<span class="step__number">{i + 1}</span>
						<h3 class="step__title">{step.title()}</h3>
						<p class="step__body">{step.body()}</p>
					</li>
				{/each}
			</ol>
		</section>

		<section class="section" aria-labelledby="crew-title">
			<h2 id="crew-title" class="section__title">{m.landing_crew_title()}</h2>
			<p class="section__lede">{m.landing_crew_lede()}</p>
			<ul class="crew">
				{#each CREW as member (member.agent)}
					<li class="crew__row">
						<AgentShape agent={member.agent} size={44} face />
						<span class="crew__text">
							<span class="crew__name">{crewName(member.agent)}</span>
							<span class="crew__does">{member.does()}</span>
						</span>
					</li>
				{/each}
			</ul>
		</section>

		{#if data.apps.length > 0}
			<section class="section" aria-labelledby="built-title">
				<div class="section__head">
					<h2 id="built-title" class="section__title">{m.showcase_title()}</h2>
					<a class="section__more" href="/showcase">{m.landing_see_all()}</a>
				</div>
				<ul class="apps">
					{#each data.apps as app (app.id)}
						<li>
							<a class="app" href={app.url} target="_blank" rel="noopener noreferrer" aria-label={m.showcase_open_app({ name: app.name })}>
								<span class="app__name">{app.name}<IconOpenInNew size={16} /></span>
								{#if app.blurb}<span class="app__blurb">{app.blurb}</span>{/if}
								<span class="app__host">{host(app.url)}</span>
							</a>
						</li>
					{/each}
				</ul>
			</section>
		{/if}

		<section class="closing" aria-labelledby="closing-title">
			<AgentShape size={64} face />
			<h2 id="closing-title" class="closing__title">{m.landing_closing_title()}</h2>
			<p class="closing__lede">{m.landing_closing_lede()}</p>
			{#if data.signedIn}
				<Button variant="filled" size="m" href="/">{m.showcase_open_nomi()}</Button>
			{:else}
				<Button variant="filled" size="m" href="/register">{m.landing_create_account()}</Button>
			{/if}
		</section>
	</main>

	<footer class="footer">
		<span>© 2026 Nomi</span>
		<a href="/showcase">{m.nav_showcase()}</a>
		{#if !data.signedIn}<a href="/login">{m.landing_sign_in()}</a>{/if}
	</footer>
</div>

<style>
	.page {
		min-height: 100dvh;
		padding: 0 12px;
		background: var(--nomi-color-ground);
		color: var(--md-sys-color-on-surface);
	}

	/* ---- Top bar ---- */
	.top {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		max-width: 1200px;
		margin: 0 auto;
		padding: 12px 4px;
	}
	.brand {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		min-height: 44px;
		border-radius: 12px;
		color: inherit;
		text-decoration: none;
	}
	.brand__word {
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.5rem;
		font-weight: 800;
		letter-spacing: -0.03em;
	}
	.top__nav {
		display: flex;
		align-items: center;
		gap: 4px;
	}
	.top__link {
		display: inline-flex;
		align-items: center;
		min-height: 44px;
		padding: 0 12px;
		border-radius: 999px;
		color: var(--md-sys-color-on-surface);
		font-weight: 600;
		text-decoration: none;
	}
	.top__link:hover,
	.stage__signin:hover,
	.section__more:hover,
	.footer a:hover {
		background: color-mix(in srgb, currentColor 8%, transparent);
	}
	a:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}

	/* ---- Stage: the hero ---- */
	.stage {
		position: relative;
		display: grid;
		grid-template-columns: minmax(0, 1.05fr) minmax(0, 1fr);
		align-items: center;
		gap: 40px;
		max-width: 1200px;
		margin: 0 auto;
		padding: clamp(32px, 6vw, 72px);
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
	.stage__nomi {
		position: absolute;
		left: -12%;
		bottom: -38%;
		width: min(520px, 70%);
		opacity: 0.22;
	}
	.stage__nomi :global(.stage__nomi-svg) {
		width: 100%;
		height: auto;
		animation: stage-turn 60s linear infinite;
	}
	@keyframes stage-turn {
		to {
			rotate: 360deg;
		}
	}
	/* Every scene's headline sits in one grid cell, so the hero is as tall as the longest and
	   never jumps; the one showing rolls up into place and the one before rolls up and away. */
	.stage__title {
		display: grid;
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: clamp(2.5rem, 6.2vw, 4.75rem);
		font-weight: 800;
		line-height: 0.98;
		letter-spacing: -0.035em;
		font-variation-settings: 'wdth' 85;
	}
	.stage__line {
		grid-area: 1 / 1;
		max-width: 12ch;
		opacity: 0;
		translate: 0 0.5em;
		visibility: hidden;
	}
	.stage__sentence {
		display: block;
	}
	.stage__line--on,
	.stage__line--out {
		visibility: visible;
		transition:
			opacity 360ms ease,
			translate 620ms cubic-bezier(0.38, 1.21, 0.22, 1);
	}
	.stage__line--on {
		opacity: 1;
		translate: 0 0;
	}
	.stage__line--out {
		translate: 0 -0.5em;
		transition-duration: 240ms, 360ms;
	}
	.stage__lede {
		margin: 20px 0 0;
		max-width: 46ch;
		font-size: 1.125rem;
		line-height: 1.55;
		opacity: 0.86;
	}
	.stage__actions {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 12px 20px;
		margin-top: 32px;
	}
	.stage__signin {
		display: inline-flex;
		align-items: center;
		min-height: 44px;
		padding: 0 12px;
		border-radius: 999px;
		color: inherit;
		font-weight: 600;
	}

	/* ---- The chat the hero plays ---- */
	.chat {
		margin: 0;
		padding: 20px;
		display: flex;
		flex-direction: column;
		gap: 14px;
		border-radius: 28px;
		background: color-mix(in srgb, var(--nomi-color-on-stage) 7%, transparent);
	}
	.scenes {
		display: flex;
		gap: 4px;
		align-self: flex-start;
		padding: 4px;
		border-radius: 999px;
		background: color-mix(in srgb, var(--nomi-color-on-stage) 8%, transparent);
	}
	.scenes__pick {
		min-height: 36px;
		padding: 0 14px;
		border: none;
		border-radius: 999px;
		background: transparent;
		color: inherit;
		font: inherit;
		font-weight: 600;
		font-size: 0.875rem;
		cursor: pointer;
		transition: background 160ms ease;
	}
	.scenes__pick:hover {
		background: color-mix(in srgb, var(--nomi-color-on-stage) 10%, transparent);
	}
	.scenes__pick[aria-pressed='true'] {
		background: var(--nomi-color-on-stage);
		color: var(--nomi-color-stage);
	}
	.scenes__pick:focus-visible {
		outline: 2px solid var(--nomi-color-on-stage);
		outline-offset: 2px;
	}
	.chat__body {
		display: grid;
	}
	.chat__play {
		grid-area: 1 / 1;
		display: flex;
		flex-direction: column;
		justify-content: flex-end;
		gap: 14px;
	}
	.chat__play--sizer {
		visibility: hidden;
	}
	.chat__lines {
		display: flex;
		flex-direction: column;
		gap: 10px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.bubble {
		max-width: 88%;
		padding: 10px 14px;
		line-height: 1.45;
		animation: bubble-in 420ms cubic-bezier(0.38, 1.21, 0.22, 1) both;
	}
	.bubble--you {
		align-self: flex-end;
		border-radius: 20px 20px 6px 20px;
		background: var(--nomi-color-on-stage);
		color: var(--nomi-color-stage);
	}
	.bubble--agent {
		align-self: flex-start;
		display: flex;
		align-items: flex-start;
		gap: 10px;
		border-radius: 20px 20px 20px 6px;
		background: color-mix(in srgb, var(--nomi-color-on-stage) 10%, transparent);
	}
	.bubble__body {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.bubble__name {
		font-size: 0.8125rem;
		font-weight: 700;
		opacity: 0.75;
	}
	@keyframes bubble-in {
		from {
			opacity: 0;
			translate: 0 12px;
			scale: 0.96;
		}
	}
	.chat__result {
		align-self: flex-start;
		display: inline-flex;
		align-items: center;
		gap: 8px;
		margin: 0;
		padding: 8px 14px 8px 10px;
		border-radius: 999px;
		background: var(--nomi-gradient-action);
		color: var(--nomi-on-action);
		font-weight: 700;
		opacity: 0;
		scale: 0.9;
		transition:
			opacity 300ms ease,
			scale 420ms cubic-bezier(0.38, 1.21, 0.22, 1);
	}
	.chat__result--on {
		opacity: 1;
		scale: 1;
	}

	/* ---- Sections ---- */
	/* Sections line up with the hero's text: the stage's width and inner padding. */
	.section {
		max-width: 1200px;
		margin: 0 auto;
		padding: clamp(56px, 9vw, 96px) clamp(32px, 6vw, 72px) 0;
	}
	.section__head {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 12px;
	}
	.section__title {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: clamp(1.75rem, 3.6vw, 2.5rem);
		font-weight: 800;
		letter-spacing: -0.025em;
		line-height: 1.1;
	}
	.section__lede {
		margin: 10px 0 0;
		max-width: 60ch;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 1.0625rem;
		line-height: 1.55;
	}
	.section__more {
		display: inline-flex;
		align-items: center;
		min-height: 44px;
		padding: 0 12px;
		border-radius: 999px;
		color: var(--md-sys-color-primary);
		font-weight: 600;
		white-space: nowrap;
	}

	.steps {
		display: grid;
		grid-template-columns: repeat(4, minmax(0, 1fr));
		gap: 28px;
		margin: 32px 0 0;
		padding: 0;
		list-style: none;
	}
	.step__number {
		display: inline-grid;
		place-items: center;
		width: 40px;
		height: 40px;
		border-radius: 50%;
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
		font-family: var(--md-ref-typeface-brand);
		font-weight: 800;
		font-size: 1.125rem;
	}
	.step__title {
		margin: 16px 0 0;
		font-size: 1.1875rem;
		font-weight: 700;
	}
	.step__body {
		margin: 6px 0 0;
		color: var(--md-sys-color-on-surface-variant);
		line-height: 1.55;
	}

	.crew {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 0 40px;
		margin: 28px 0 0;
		padding: 0;
		list-style: none;
	}
	.crew__row {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 16px 0;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.crew__text {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.crew__name {
		font-size: 1.0625rem;
		font-weight: 700;
	}
	.crew__does {
		color: var(--md-sys-color-on-surface-variant);
		line-height: 1.45;
	}

	.apps {
		display: grid;
		grid-template-columns: repeat(3, minmax(0, 1fr));
		gap: 16px;
		margin: 24px 0 0;
		padding: 0;
		list-style: none;
	}
	.app {
		display: flex;
		flex-direction: column;
		gap: 6px;
		height: 100%;
		padding: 18px 20px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-lowest);
		color: inherit;
		text-decoration: none;
		transition: background 160ms ease;
	}
	.app:hover {
		background: var(--md-sys-color-surface-container-low);
	}
	.app__name {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		font-weight: 700;
		font-size: 1.0625rem;
	}
	.app__blurb {
		color: var(--md-sys-color-on-surface-variant);
		line-height: 1.45;
	}
	.app__host {
		margin-top: auto;
		padding-top: 6px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}

	.closing {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 14px;
		max-width: 640px;
		margin: 0 auto;
		padding: clamp(72px, 11vw, 120px) 4px clamp(56px, 8vw, 88px);
		text-align: center;
	}
	.closing__title {
		margin: 6px 0 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: clamp(2rem, 4.4vw, 3rem);
		font-weight: 800;
		letter-spacing: -0.03em;
		line-height: 1.05;
	}
	.closing__lede {
		margin: 0 0 10px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 1.0625rem;
		line-height: 1.55;
	}

	.footer {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: center;
		gap: 4px 12px;
		max-width: 1200px;
		margin: 0 auto;
		padding: 20px 4px 28px;
		border-top: 1px solid var(--md-sys-color-outline-variant);
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.footer a {
		display: inline-flex;
		align-items: center;
		min-height: 44px;
		padding: 0 10px;
		border-radius: 999px;
		color: inherit;
	}

	@media (max-width: 960px) {
		.stage {
			grid-template-columns: 1fr;
		}
		.steps {
			grid-template-columns: repeat(2, minmax(0, 1fr));
		}
		.apps {
			grid-template-columns: 1fr;
		}
	}
	@media (max-width: 640px) {
		.page {
			padding: 0 8px;
		}
		.top__link:first-child {
			display: none;
		}
		.stage {
			padding: 32px 20px 20px;
			border-radius: 32px 32px 32px 10px;
		}
		.stage__nomi {
			left: auto;
			right: -42%;
			bottom: auto;
			top: -16%;
			width: 72%;
		}
		.chat {
			padding: 14px;
		}
		.scenes__pick {
			min-height: 44px;
		}
		/* On a phone the panel runs below the fold, so the chat starts at its top. */
		.chat__play {
			justify-content: flex-start;
		}
		.section {
			padding-inline: 20px;
		}
		.steps,
		.crew {
			grid-template-columns: 1fr;
		}
	}
	@media (prefers-reduced-motion: reduce) {
		.stage__nomi :global(.stage__nomi-svg),
		.bubble {
			animation: none;
		}
		.chat__result,
		.stage__line--on,
		.stage__line--out {
			transition: none;
		}
	}
</style>
