<script lang="ts">
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// Flow / Mobile — You & Settings, step 11 (a crew member) → Member: the mascot at 96 (a custom
	// member's initial on the surface, the same size), 14 above the name at 22 as the screen's title
	// and what they do at 14 under it.
	type Props = { agent?: CrewId; name: string; role: string };
	let { agent, name, role }: Props = $props();
</script>

<div class="flex flex-col gap-3.5 px-5 pt-1 pb-4.5">
	{#if agent}
		<Mascot {agent} size={96} />
	{:else}
		<span aria-hidden="true" class="inline-flex size-24 items-center justify-center rounded-full bg-surface text-phone-large font-semibold text-text-2 outline -outline-offset-1 outline-border">{name.charAt(0).toUpperCase()}</span>
	{/if}
	<div class="flex flex-col gap-1">
		<h1 class="text-phone-display font-semibold text-text">{name}</h1>
		<p class="text-phone-sm text-text-2">{role}</p>
	</div>
</div>
