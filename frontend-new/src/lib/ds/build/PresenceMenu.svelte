<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import Button from '../actions/Button.svelte';
	import PeerAvatar from './PeerAvatar.svelte';

	// Build together → Presence menu: who's in the project, what each is looking at, and follow
	// to see what they see (you, last, can't be followed).
	type Person = { name: string; initials: string; peer?: 1 | 2; where: string; you?: boolean };
	type Props = { people: Person[]; onfollow: (index: number) => void };
	let { people, onfollow }: Props = $props();
</script>

<div class="flex w-80 max-w-full flex-col gap-0.5 rounded-md bg-surface p-1.5 shadow-float outline -outline-offset-1 outline-border-strong">
	<p class="px-2.5 pt-1.5 pb-1 font-mono text-label text-text-3"><span class="tracking-label uppercase">{m.presence_here({ count: String(people.length) })}</span></p>
	{#each people as person, index (person.initials)}
		<div class="flex items-center gap-2.5 rounded-sm px-2.5 py-2 hover:bg-surface-2">
			<PeerAvatar initials={person.initials} peer={person.peer} ring={!person.you && !!person.peer} />
			<span class="flex min-w-0 flex-1 flex-col gap-0.5">
				<span class="truncate text-ui font-medium text-text">{person.name}</span>
				<span class="text-caption text-text-3">{person.where}</span>
			</span>
			{#if !person.you}<Button label={m.presence_follow()} variant="secondary" onclick={() => onfollow(index)} />{/if}
		</div>
	{/each}
</div>
