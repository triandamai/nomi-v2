<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { onMount } from 'svelte';
	import { TAGLINE_VERBS } from '$lib/tagline';

	// "A small crew of agents that [remembers] for you.", with the verb rolling up like a wheel to
	// the next one in $lib/tagline and wrapping back to the first. Every verb sits in the same
	// grid cell, so the slot is as wide as the longest and the line never reflows. Under reduced
	// motion it shows the plain line instead.

	let { stepMs = 2200 }: { stepMs?: number } = $props();

	let index = $state(0);
	let reducedMotion = $state(false);

	onMount(() => {
		reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
		if (reducedMotion || TAGLINE_VERBS.length < 2) return;
		const handle = setInterval(() => {
			index = (index + 1) % TAGLINE_VERBS.length;
		}, stepMs);
		return () => clearInterval(handle);
	});

	// The verb rolling out sits above; every other one waits below, so the next always comes up.
	const previous = $derived((index - 1 + TAGLINE_VERBS.length) % TAGLINE_VERBS.length);
</script>

{#if reducedMotion}
	{m.auth_line()}
{:else}
	<span class="tagline">
		<span class="tagline__sr">{m.auth_line()}</span>
		<span aria-hidden="true">
			{m.tagline_before()}
			<span class="wheel">
				{#each TAGLINE_VERBS as verb, i (i)}
					<span class="wheel__verb" class:wheel__verb--current={i === index} class:wheel__verb--previous={i === previous && i !== index}>
						{verb()}
					</span>
				{/each}
			</span>
			{m.tagline_after()}
		</span>
	</span>
{/if}

<style>
	.tagline__sr {
		position: absolute;
		width: 1px;
		height: 1px;
		overflow: hidden;
		clip-path: inset(50%);
		white-space: nowrap;
	}
	.wheel {
		display: inline-grid;
		vertical-align: bottom;
		perspective: 600px;
		/* Room for descenders; the clip keeps the rolling verbs inside the line. */
		padding-bottom: 0.08em;
		overflow: hidden;
	}
	.wheel__verb {
		grid-area: 1 / 1;
		justify-self: start;
		white-space: nowrap;
		color: var(--tagline-accent, inherit);
		backface-visibility: hidden;
		opacity: 0;
		transform: translateY(90%) rotateX(-70deg);
		transform-origin: 50% 50% -0.5em;
		transition:
			transform 620ms cubic-bezier(0.34, 1.4, 0.64, 1),
			opacity 380ms ease;
	}
	.wheel__verb--current {
		opacity: 1;
		transform: none;
	}
	.wheel__verb--previous {
		transform: translateY(-90%) rotateX(70deg);
		transition:
			transform 520ms cubic-bezier(0.4, 0, 0.2, 1),
			opacity 300ms ease;
	}
</style>
