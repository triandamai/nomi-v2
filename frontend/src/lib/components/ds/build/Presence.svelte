<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import PeerAvatar from './PeerAvatar.svelte';

	// Build together → Presence, in the top bar: who's in the project, overlapping, the last one
	// online marked in mint.
	type Person = { name: string; initials: string; peer?: 1 | 2; online?: boolean };
	let { people }: { people: Person[] } = $props();
</script>

<span role="group" aria-label={m.presence_label()} class="relative inline-flex items-center">
	{#each people as person, index (person.initials)}
		<span class={['relative inline-flex', index > 0 && '-ml-1.5']} title={person.name}>
			<PeerAvatar initials={person.initials} peer={person.peer} ring />
			{#if person.online}<span data-online class="absolute -right-0.5 -bottom-0.5 size-2 rounded-full border-2 border-bg bg-accent"></span>{/if}
		</span>
	{/each}
</span>
