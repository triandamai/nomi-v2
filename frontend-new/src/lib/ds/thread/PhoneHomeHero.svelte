<script lang="ts">
	import Mascot from '../agents/Mascot.svelte';

	// Home on a phone, the top of it: Nomi at 96 over the greeting in the phone's large title and the
	// line on what to do, centred, 16 under the mascot and 8 between, on a grid that fades out under
	// them like the landing hero's. Not in the pen: the person's own layout (AGENTS.md §9).
	type Props = { title: string; intro: string };
	let { title, intro }: Props = $props();
</script>

<div class="relative flex flex-col items-center px-8 pt-16 pb-10 text-center">
	<div class="grid-lines pointer-events-none absolute inset-0" data-home-grid aria-hidden="true"></div>
	<div class="relative"><Mascot agent="nomi" size={96} /></div>
	<h1 class="relative mt-6 text-phone-large font-semibold tracking-phone-large text-text">{title}</h1>
	<p class="relative mt-2 max-w-xs text-phone text-text-2">{intro}</p>
</div>

<style>
	/* 1px lines every 34 (eleven across a 375 phone), fading out under the greeting. Scoped here:
	   the tokens have no pattern utility. */
	.grid-lines {
		background-image: linear-gradient(to right, var(--nm-border) 1px, transparent 1px), linear-gradient(to bottom, var(--nm-border) 1px, transparent 1px);
		background-size: 34px 34px;
		background-position: center top;
		mask-image: linear-gradient(to bottom, black 40%, transparent 100%);
	}
</style>
