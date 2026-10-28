<script lang="ts">
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// Design System / Nomi → Identity: a crew member in a list, with what they do and their state.
	type Props = { agent: CrewId; name: string; role: string; state?: string; href?: string };
	let { agent, name, role, state, href }: Props = $props();
</script>

{#snippet body()}
	<Mascot {agent} size={32} />
	<span class="flex min-w-0 flex-1 flex-col gap-0.5">
		<span class="text-ui font-medium text-text">{name}</span>
		<span class="text-caption text-text-3">{role}</span>
	</span>
	{#if state}<span class="font-mono text-label text-text-3">{state}</span>{/if}
{/snippet}

{#if href}
	<a {href} class="flex w-full items-center gap-3 rounded-sm px-3 py-2 outline-offset-1 hover:bg-surface-2">{@render body()}</a>
{:else}
	<div class="flex w-full items-center gap-3 rounded-sm px-3 py-2">{@render body()}</div>
{/if}
