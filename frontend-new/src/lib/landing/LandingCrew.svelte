<script lang="ts" module>
	import type { LandingAgent } from './LandingMascot.svelte';

	export type CrewCard = { agent: LandingAgent; name: string; role: string; body: string; example: string };
</script>

<script lang="ts">
	import LandingMascot from './LandingMascot.svelte';

	// Figma, Landing v2 → Crew (170:2895): the section head, then the crew four to a row, each a Crew
	// Card (170:2759, 320 high, 24 inset, radius 32, in the member's tint): the name, the role, what
	// they do, an example in mono, and the mascot peeking from the foot, tilted 8°, which hops up on
	// hover (the component's note).
	type Props = { id: string; title: string; lead: string; crew: CrewCard[] };
	let { id, title, lead, crew }: Props = $props();
</script>

<section {id} class="crew" aria-labelledby="{id}-title">
	<div class="inner">
		<div class="head">
			<h2 id="{id}-title">{title}</h2>
			<p>{lead}</p>
		</div>
		<ul class="grid">
			{#each crew as member (member.agent)}
				<li class="card {member.agent}">
					<h3>{member.name}</h3>
					<p class="role">{member.role}</p>
					<p class="body">{member.body}</p>
					<span class="example">{member.example}</span>
					<span class="peek"><LandingMascot agent={member.agent} size={150} /></span>
				</li>
			{/each}
		</ul>
	</div>
</section>

<style>
	.crew {
		background: var(--lp-bg-subtle);
	}

	.inner {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 64px;
		max-width: 1440px;
		margin: 0 auto;
		padding: 120px 64px;
	}

	.head {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 16px;
		text-align: center;
	}

	h2 {
		max-width: 1000px;
		margin: 0;
		font-size: clamp(34px, 4.4vw, 56px);
		line-height: 1.04;
		font-weight: 500;
		letter-spacing: -0.04em;
		text-wrap: balance;
	}

	.head p {
		max-width: 600px;
		margin: 0;
		font-size: 18px;
		line-height: 1.55;
		letter-spacing: -0.18px;
		color: var(--lp-text-2);
	}

	.grid {
		display: grid;
		grid-template-columns: repeat(4, minmax(0, 1fr));
		gap: 16px;
		width: 100%;
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.card {
		position: relative;
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 8px;
		height: 320px;
		padding: 24px;
		overflow: hidden;
		border-radius: var(--lp-radius-2xl);
	}

	h3 {
		margin: 0;
		font-size: 24px;
		line-height: 30px;
		font-weight: 600;
		letter-spacing: -0.48px;
	}

	.role {
		margin: 0;
		font-size: 14px;
		line-height: 18px;
		font-weight: 500;
		letter-spacing: -0.14px;
		color: var(--lp-text-2);
	}

	.body {
		margin: 0;
		font-size: 14px;
		line-height: 1.5;
		letter-spacing: -0.14px;
		color: var(--lp-text-2);
	}

	.example {
		padding: 4px 8px;
		border-radius: var(--lp-radius-md);
		background: var(--lp-surface);
		font-family: var(--nm-mono);
		font-size: 12px;
		line-height: 16px;
		white-space: nowrap;
	}

	/* The design's 169.4 box at (99, 200), the 150 mascot tilted 8° in its middle. */
	.peek {
		position: absolute;
		top: 200px;
		left: 99.12px;
		display: flex;
		align-items: center;
		justify-content: center;
		width: 169.42px;
		height: 169.42px;
		rotate: 8deg;
		transition: translate 0.35s var(--lp-spring);
	}

	.card:hover .peek {
		translate: 0 -12px;
	}

	.nomi {
		background: var(--lp-crew-nomi);
	}

	.sora {
		background: var(--lp-crew-sora);
	}

	.koda {
		background: var(--lp-crew-koda);
	}

	.iris {
		background: var(--lp-crew-iris);
	}

	.dana {
		background: var(--lp-crew-dana);
	}

	.kala {
		background: var(--lp-crew-kala);
	}

	.maya {
		background: var(--lp-crew-maya);
	}

	.tara {
		background: var(--lp-crew-tara);
	}

	.rena {
		background: var(--lp-crew-rena);
	}

	.raga {
		background: var(--lp-crew-raga);
	}

	.supervisor {
		background: var(--lp-crew-supervisor);
	}

	.personality {
		background: var(--lp-crew-personality);
	}

	@media (max-width: 1023px) {
		.inner {
			padding: 80px 24px;
		}

		.grid {
			grid-template-columns: repeat(2, minmax(0, 1fr));
		}
	}

	@media (max-width: 599px) {
		.inner {
			padding: 64px 16px;
		}

		.grid {
			grid-template-columns: minmax(0, 1fr);
		}

		.card {
			height: 260px;
		}

		.peek {
			top: 140px;
			left: auto;
			right: 8px;
		}
	}
</style>
