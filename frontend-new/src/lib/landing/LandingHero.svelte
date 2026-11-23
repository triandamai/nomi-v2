<script lang="ts" module>
	import type { LandingAgent } from './LandingMascot.svelte';

	export type HeroToast = { agent: LandingAgent; name: string; line: string };
	export type HeroChip = { agent: LandingAgent; label: string };
</script>

<script lang="ts">
	import LandingComposer from './LandingComposer.svelte';
	import LandingMascot from './LandingMascot.svelte';

	// Figma, Landing v2 → Hero (170:2777): the 80px grid fading out at the foot, the headline in three
	// lines with a mascot pill in each, the lead, the composer, "or try:" and its chips, and six crew
	// notes floating around it. The composer is a real form: what's typed goes to sign in, then to a
	// new chat with it in the field (the page's action). Attach and the mic need an account too.
	// Motion (get_motion_context): the lines rise in 150ms apart, the composer after them, then the
	// chips; the mascots hop in turn; the notes pop in one by one and bob.
	type Props = {
		title: string;
		/** The headline's three lines, each [before the pill, after it]. */
		lines: [[string, string], [string, string], [string, string]];
		lead: string;
		label: string;
		value?: string;
		action: string;
		attach: string;
		voice: string;
		send: string;
		tryLabel: string;
		chips: HeroChip[];
		toasts: HeroToast[];
	};
	let { title, lines, lead, label, value = $bindable(''), action, attach, voice, send, tryLabel, chips, toasts }: Props = $props();

	let field: HTMLTextAreaElement | undefined = $state();

	// Where each note floats in the 1440 frame (its box), its tilt, and which way it bobs.
	const SPOTS = [
		{ left: 53.07, top: 161, width: 288.54, height: 70.89, tilt: 3, bob: -7 },
		{ left: 104, top: 352.3, width: 279.79, height: 65.67, tilt: -2, bob: 7 },
		{ left: 46.05, top: 563, width: 272.79, height: 65.42, tilt: 2, bob: -7 },
		{ left: 1134, top: 131.92, width: 252.59, height: 69.01, tilt: -3, bob: 7 },
		{ left: 1107.05, top: 354, width: 236.81, height: 64.17, tilt: 2, bob: -7 },
		{ left: 1127, top: 546.03, width: 258.8, height: 64.94, tilt: -2, bob: 7 },
	];

	function pick(text: string) {
		value = text;
		field?.focus();
	}
</script>

<section class="hero" aria-labelledby="landing-hero-title">
	<div class="grid" aria-hidden="true"></div>
	<div class="frame">
		<h1 id="landing-hero-title" class="headline" aria-label={title}>
			<span class="line rise" style:--lp-delay="0.1s" aria-hidden="true">
				<span>{lines[0][0]}</span>
				<span class="pill pill-nomi"><LandingMascot agent="nomi" size={68.1} hop={0.92} /></span>
				<span>{lines[0][1]}</span>
			</span>
			<span class="line rise" style:--lp-delay="0.25s" aria-hidden="true">
				<span>{lines[1][0]}</span>
				<span class="pill pill-crew">
					<span class="overlap"><LandingMascot agent="dana" size={57.6} hop={1.04} /></span>
					<span class="overlap"><LandingMascot agent="kala" size={57.6} hop={1.16} /></span>
					<LandingMascot agent="tara" size={57.6} hop={1.28} />
				</span>
				<span>{lines[1][1]}</span>
			</span>
			<span class="line rise" style:--lp-delay="0.4s" aria-hidden="true">
				<span>{lines[2][0]}</span>
				<span class="pill pill-koda"><LandingMascot agent="koda" size={67.2} hop={1.4} /></span>
				<span>{lines[2][1]}</span>
			</span>
		</h1>
		<p class="lead">{lead}</p>

		<div class="rise composer-wrap" style:--lp-delay="0.7s" style:--lp-rise="20px">
			<LandingComposer bind:value bind:field {label} {action} {attach} {voice} {send} />
		</div>

		<div class="chips">
			<span class="try">{tryLabel}</span>
			{#each chips as chip, index (chip.label)}
				<button class="chip rise" style:--lp-delay="{1.2 + index * 0.1}s" style:--lp-rise="10px" type="button" onclick={() => pick(chip.label)}>
					<LandingMascot agent={chip.agent} size={20} />
					{chip.label}
				</button>
			{/each}
		</div>

		<div class="toasts" aria-hidden="true">
			{#each toasts.slice(0, SPOTS.length) as toast, index (toast.agent)}
				{@const spot = SPOTS[index]}
				<div class="spot" style:left="{spot.left}px" style:top="{spot.top}px" style:width="{spot.width}px" style:height="{spot.height}px">
					<div class="tilt" style:rotate="{spot.tilt}deg">
						<div class="toast {toast.agent}" style:--lp-delay="{0.6 + index * 0.2}s" style:--lp-bob="{spot.bob}px">
							<LandingMascot agent={toast.agent} size={24} />
							<span class="text"><span class="name">{toast.name}</span><span class="note">{toast.line}</span></span>
						</div>
					</div>
				</div>
			{/each}
		</div>
	</div>
</section>

<style>
	.hero {
		position: relative;
		overflow: hidden;
	}

	/* 1px lines every 80, from the frame's left edge, fading out over its last 220. */
	.grid {
		position: absolute;
		inset: 0 0 -40px 0;
		background-image: linear-gradient(to right, var(--lp-border-subtle) 1px, transparent 1px), linear-gradient(to bottom, var(--lp-border-subtle) 1px, transparent 1px);
		background-size: 80px 80px;
		background-position: calc(50% - 720px) 0;
		mask-image: linear-gradient(to bottom, black 624px, transparent 844px);
	}

	.frame {
		position: relative;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 28px;
		max-width: 1440px;
		margin: 0 auto;
		padding: 112px 16px 128px;
	}

	/* Display/2XL: 72 / 1.02, −2.88. The pills scale with it (their sizes are in em of 72). */
	.headline {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
		margin: 0;
		font-size: clamp(38px, 6vw, 72px);
		line-height: 1.02;
		font-weight: 500;
		letter-spacing: -0.04em;
		color: var(--lp-text);
	}

	.line {
		display: flex;
		align-items: center;
		gap: 0.25em;
		white-space: nowrap;
	}

	.pill {
		display: flex;
		align-items: flex-end;
		justify-content: center;
		height: 1.0667em;
		padding-top: 0.1556em;
		overflow: hidden;
		border-radius: 999px;
		font-size: inherit;
	}

	.pill :global(.mascot) {
		width: 0.9459em !important;
		height: 0.9459em !important;
	}

	.pill-nomi {
		width: 1.6667em;
		background: var(--lp-green-subtle);
	}

	.pill-crew {
		width: 2.6222em;
		background: var(--lp-yellow-subtle);
	}

	.pill-crew :global(.mascot) {
		width: 0.8em !important;
		height: 0.8em !important;
	}

	.overlap {
		display: flex;
		margin-right: -0.1556em;
	}

	.pill-koda {
		width: 1.6667em;
		background: var(--lp-crew-koda);
	}

	.pill-koda :global(.mascot) {
		width: 0.9333em !important;
		height: 0.9333em !important;
	}

	.lead {
		max-width: 600px;
		margin: 0;
		font-size: 18px;
		line-height: 1.55;
		letter-spacing: -0.18px;
		text-align: center;
		color: var(--lp-text-2);
	}

	.composer-wrap {
		display: flex;
		justify-content: center;
		width: 100%;
	}

	.chips {
		display: flex;
		flex-wrap: wrap;
		justify-content: center;
		align-items: center;
		gap: 8px;
	}

	.try {
		font-family: var(--nm-mono);
		font-size: 12px;
		line-height: 16px;
		color: var(--lp-text-3);
	}

	.chip {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		padding: 7px 11px;
		border: 1px solid var(--lp-border);
		border-radius: 999px;
		background: var(--lp-surface);
		font: inherit;
		font-size: 14px;
		line-height: 18px;
		font-weight: 500;
		letter-spacing: -0.14px;
		color: var(--lp-text);
		cursor: pointer;
	}

	.chip:hover {
		border-color: var(--lp-border-strong);
	}

	.chip:focus-visible {
		outline: 2px solid var(--lp-focus);
		outline-offset: 2px;
	}

	.rise {
		animation: lp-rise 0.5s var(--lp-spring) both;
		animation-delay: var(--lp-delay, 0s);
	}

	/* The notes sit in the 1440 frame, so they show only where there's room around the column. */
	.toasts {
		position: absolute;
		inset: 0;
		pointer-events: none;
	}

	.spot {
		position: absolute;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	/* Each note keeps its tilt from the design while it pops in and bobs. */
	.tilt {
		display: flex;
		flex: none;
	}

	.toast {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 10px 14px;
		border-radius: var(--lp-radius-xl);
		white-space: nowrap;
		animation:
			lp-pop 0.5s var(--lp-spring) both,
			lp-bob 1.1s ease-in-out calc(var(--lp-delay) + 0.5s) infinite alternate;
		animation-delay: var(--lp-delay), calc(var(--lp-delay) + 0.5s);
	}

	.toast .text {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.name {
		font-size: 14px;
		line-height: 18px;
		font-weight: 500;
		letter-spacing: -0.14px;
	}

	.note {
		font-family: var(--nm-mono);
		font-size: 12px;
		line-height: 16px;
		color: var(--lp-text-2);
	}

	.dana {
		background: var(--lp-crew-dana);
		color: var(--lp-crew-dana-fg);
	}

	.kala {
		background: var(--lp-crew-kala);
		color: var(--lp-crew-kala-fg);
	}

	.maya {
		background: var(--lp-crew-maya);
		color: var(--lp-crew-maya-fg);
	}

	.koda {
		background: var(--lp-crew-koda);
		color: var(--lp-crew-koda-fg);
	}

	.tara {
		background: var(--lp-crew-tara);
		color: var(--lp-crew-tara-fg);
	}

	.rena {
		background: var(--lp-crew-rena);
		color: var(--lp-crew-rena-fg);
	}

	@media (max-width: 1279px) {
		.toasts {
			display: none;
		}
	}

	@media (max-width: 767px) {
		.frame {
			padding: 56px 16px 72px;
		}

		.lead {
			font-size: 16px;
		}
	}
</style>
