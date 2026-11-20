<script lang="ts" module>
	import type { LandingAgent } from './LandingMascot.svelte';

	// The parade (170:3058) as its layers place it in the 1164 strip: each mascot's box (its tilted
	// square), the tilt, and when it hops in the page's 6s loop (get_motion_context, Hop/*).
	const PARADE: { agent: LandingAgent; x: number; y: number; box: number; tilt: number; hop: number }[] = [
		{ agent: 'raga', x: 0, y: 32.78, box: 100.38, tilt: 6, hop: 0.72 },
		{ agent: 'dana', x: 83.62, y: 19.28, box: 115.02, tilt: -6, hop: 0.81 },
		{ agent: 'sora', x: 180.56, y: 25.07, box: 110.21, tilt: 6, hop: 0.99 },
		{ agent: 'kala', x: 273.13, y: 28.93, box: 104.57, tilt: -6, hop: 0.9 },
		{ agent: 'koda', x: 360.56, y: 13.5, box: 125.48, tilt: 6, hop: 0.99 },
		{ agent: 'nomi', x: 467.01, y: 0, box: 139.88, tilt: -6, hop: 1.08 },
		{ agent: 'supervisor', x: 586.57, y: 15.43, box: 123.39, tilt: 6, hop: 1.17 },
		{ agent: 'iris', x: 691.13, y: 21.21, box: 114.45, tilt: -6, hop: 1.26 },
		{ agent: 'tara', x: 787.55, y: 27, box: 108.75, tilt: 6, hop: 1.26 },
		{ agent: 'maya', x: 878.78, y: 17.36, box: 117.11, tilt: -6, hop: 1.35 },
		{ agent: 'rena', x: 977.62, y: 30.85, box: 102.47, tilt: 6, hop: 1.44 },
		{ agent: 'personality', x: 1063.15, y: 23.14, box: 110.84, tilt: -6, hop: 1.53 },
	];
	// A square tilted 6° fills a box 1.099 times its side.
	const SIDE = (box: number) => Math.round((box / 1.099) * 100) / 100;
</script>

<script lang="ts">
	import LandingComposer from './LandingComposer.svelte';
	import LandingMascot from './LandingMascot.svelte';

	// Figma, Landing v2 → Final CTA (170:3053), from its layer metadata and the page's render (the
	// design context wasn't read: the Figma MCP's limit). A 1312 panel, radius 32, in the light
	// green-to-mint wash the render shows (#f6fce2 → #ecfaf8, measured), 96 above the title (72),
	// the line (18 / 28) and an empty composer, 24 apart; the whole crew hops along its foot,
	// cut off by the panel's edge as drawn.
	type Props = { title: string; lead: string; label: string; placeholder: string; action: string; attach: string; voice: string; send: string };
	let { title, lead, label, placeholder, action, attach, voice, send }: Props = $props();
	let value = $state('');
</script>

<section class="cta" aria-labelledby="landing-cta-title">
	<div class="panel">
		<h2 id="landing-cta-title">{title}</h2>
		<p>{lead}</p>
		<LandingComposer bind:value {label} {placeholder} {action} {attach} {voice} {send} rows={1} />
		<div class="parade" aria-hidden="true">
			{#each PARADE as member (member.agent)}
				<span class="spot" style:left="{member.x}px" style:top="{member.y}px" style:width="{member.box}px" style:height="{member.box}px">
					<span class="tilt" style:rotate="{member.tilt}deg"><LandingMascot agent={member.agent} size={SIDE(member.box)} hop={member.hop} /></span>
				</span>
			{/each}
		</div>
	</div>
</section>

<style>
	.cta {
		max-width: 1440px;
		margin: 0 auto;
		padding: 120px 64px 64px;
	}

	.panel {
		position: relative;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 24px;
		height: 474.34px;
		padding: 96px 16px 0;
		overflow: hidden;
		border-radius: var(--lp-radius-2xl);
		background: linear-gradient(90deg, var(--lp-cta-from), var(--lp-cta-to));
		text-align: center;
	}

	/* Display/2XL: 72 / 1.02 */
	h2 {
		max-width: 1000px;
		margin: 0;
		font-size: clamp(36px, 6vw, 72px);
		line-height: 1.02;
		font-weight: 500;
		letter-spacing: -0.04em;
		text-wrap: balance;
	}

	p {
		max-width: 600px;
		margin: 0;
		font-size: 18px;
		line-height: 28px;
		letter-spacing: -0.18px;
		color: var(--lp-text-2);
	}

	.parade {
		position: absolute;
		top: 383px;
		left: 50%;
		width: 1164px;
		height: 140px;
		translate: -50% 0;
		pointer-events: none;
	}

	.spot {
		position: absolute;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.tilt {
		display: flex;
	}

	@media (max-width: 1023px) {
		.cta {
			padding: 80px 24px 48px;
		}
	}

	@media (max-width: 599px) {
		.cta {
			padding: 64px 16px 32px;
		}

		.panel {
			height: auto;
			padding: 56px 16px 140px;
		}

		.parade {
			top: auto;
			bottom: -48px;
		}
	}
</style>
