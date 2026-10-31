<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import PeerAvatar from './PeerAvatar.svelte';

	// Build together → Teammate message: someone else in the project's chat, in their peer colour,
	// with when and whom they asked.
	type Person = { name: string; initials: string; peer?: 1 | 2 };
	type Props = { who: Person; time?: string; to?: string; text: string };
	let { who, time, to, text }: Props = $props();
</script>

<article class="flex w-full gap-2.5">
	<PeerAvatar initials={who.initials} peer={who.peer} />
	<div class="flex min-w-0 flex-1 flex-col gap-1">
		<p class="flex flex-wrap items-center gap-2">
			<span class={['text-ui font-medium', who.peer === 1 ? 'text-peer-1' : who.peer === 2 ? 'text-peer-2' : 'text-text']}>{who.name}</span>
			{#if time}<span class="font-mono text-label text-text-3">{time}</span>{/if}
			{#if to}<span class="font-mono text-label text-text-3">{m.together_to({ name: to })}</span>{/if}
		</p>
		<p class={['rounded-md px-3.5 py-2 text-body whitespace-pre-wrap text-text', who.peer === 1 ? 'bg-peer-1-bg' : who.peer === 2 ? 'bg-peer-2-bg' : 'bg-surface-2']}>{text}</p>
	</div>
</article>
