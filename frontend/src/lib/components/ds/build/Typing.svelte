<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import PeerAvatar from './PeerAvatar.svelte';

	// Build together → Typing: a teammate writing to Koda right now.
	type Props = { who: { name: string; initials: string; peer?: 1 | 2 } };
	let { who }: Props = $props();

	const first = $derived(who.name.split(/\s+/)[0]);
</script>

<p role="status" class="flex items-center gap-2">
	<PeerAvatar initials={who.initials} peer={who.peer} size={20} />
	<span class="text-caption text-text-3">{m.together_typing({ name: first })}</span>
	<span aria-hidden="true" class="flex items-center gap-0.75">
		<span class="size-1 animate-pulse rounded-full bg-text-2 motion-reduce:animate-none"></span>
		<span class="size-1 rounded-full bg-text-3"></span>
		<span class="size-1 rounded-full bg-text-3"></span>
	</span>
</p>
