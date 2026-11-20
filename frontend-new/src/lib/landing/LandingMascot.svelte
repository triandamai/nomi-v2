<script lang="ts" module>
	export type LandingAgent = 'nomi' | 'sora' | 'koda' | 'iris' | 'dana' | 'kala' | 'maya' | 'tara' | 'rena' | 'raga' | 'supervisor' | 'personality';

	const FILES = import.meta.glob<string>('./assets/mascots/*.svg', { eager: true, query: '?url', import: 'default' });
	const src = (agent: LandingAgent) => FILES[`./assets/mascots/${agent}.svg`];

	// Where each drawing sits in its square, as the design's instances place it: Nomi's mark is
	// inset, a few bodies run a hair past the top or left edge.
	const INSET: Partial<Record<LandingAgent, string>> = {
		nomi: '3.33% 3.95%',
		dana: '-0.59% 0 0 0',
		kala: '0 0 0 -1.53%',
		maya: '-1.56% 0 0 0',
		tara: '0 0 0 -1.08%',
		rena: '-0.24% 0 0 0',
		supervisor: '-0.13% 0 0 0',
		personality: '0 0 0 -0.41%',
	};
</script>

<script lang="ts">
	// Figma, Nomi Builder → Mascot (170:250): Nomi and the crew as the landing page draws them, flat
	// doodles at any size, from the design's own files. The app's crew wear Mascot.svelte's shapes;
	// this page keeps its own drawings, as its design does. Decorative unless given a label.
	type Props = {
		agent: LandingAgent;
		/** Its square, in px. */
		size: number;
		label?: string;
		/** Hop in the page's 6s loop, starting this many seconds in. */
		hop?: number;
	};
	let { agent, size, label, hop }: Props = $props();
</script>

<span
	class={['mascot', hop !== undefined && 'hop']}
	style:width="{size}px"
	style:height="{size}px"
	style:--lp-delay={hop !== undefined ? `${hop}s` : undefined}
	role={label ? 'img' : undefined}
	aria-label={label}
	aria-hidden={label ? undefined : 'true'}
>
	<span class="art" style:inset={INSET[agent] ?? '0'}><img src={src(agent)} alt="" draggable="false" /></span>
</span>

<style>
	.mascot {
		position: relative;
		display: inline-block;
		flex-shrink: 0;
	}

	.art {
		position: absolute;
	}

	/* lp-hop is landing.css's: two hops in the page's 6s loop. */
	.hop {
		transform-origin: 50% 100%;
		animation: lp-hop 6s infinite;
		animation-delay: var(--lp-delay, 0s);
	}

	.art img {
		display: block;
		width: 100%;
		height: 100%;
		max-width: none;
	}
</style>
