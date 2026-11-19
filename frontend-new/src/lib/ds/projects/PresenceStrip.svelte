<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import PeerAvatar from '../build/PeerAvatar.svelte';

	// Flow / Mobile Build — Invite & build together (Qw22t, GnUU0) → Here Strip: under the switch while
	// someone else has the project open, a 22 face in their colour and "Sari is here · Code" at 13,
	// [6,16], 8 apart, a line under it. Tapping it lists who's here; Follow (in their colour) goes
	// where they go. With more than one, the first is named with how many others.
	type Person = { name: string; initials: string; peer?: 1 | 2; where: string };
	type Props = { people: Person[]; onopen: () => void; onfollow?: () => void };
	let { people, onopen, onfollow }: Props = $props();
	const first = $derived(people[0]);
</script>

{#if first}
	<div class="flex w-full items-center gap-2 border-b border-border bg-bg px-4 pt-1.5 pb-1.25">
		<button type="button" aria-haspopup="dialog" class="flex min-h-5.5 min-w-0 flex-1 cursor-pointer items-center gap-2 rounded-sm text-left outline-offset-2 pointer-coarse:min-h-11" onclick={onopen}>
			<span class="flex -space-x-1.5">
				{#each people.slice(0, 3) as person (person.name)}<PeerAvatar initials={person.initials} peer={person.peer} size={20} ring />{/each}
			</span>
			<span class="min-w-0 flex-1 truncate text-ui text-text-2">{people.length > 1 ? m.pb_here_many({ name: first.name, count: people.length - 1, where: first.where }) : m.pb_here({ name: first.name, where: first.where })}</span>
		</button>
		{#if onfollow}
			<button
				type="button"
				aria-label={m.pb_follow_name({ name: first.name })}
				class={['inline-flex h-9 shrink-0 cursor-pointer items-center rounded-full border px-2.25 text-ui font-medium outline-offset-1 pointer-coarse:min-h-11', first.peer === 2 ? 'border-peer-2 text-peer-2' : 'border-peer-1 text-peer-1']}
				onclick={onfollow}>{m.presence_follow()}</button
			>
		{/if}
	</div>
{/if}
