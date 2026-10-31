<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import PeerAvatar from '../build/PeerAvatar.svelte';

	// Mobile / Teammate Message: someone else in the project's chat on a phone, in their colour,
	// with when and whom they asked, at the phone's 16pt.
	type Person = { name: string; initials: string; peer?: 1 | 2 };
	type Props = { who: Person; time?: string; to?: string; text: string };
	let { who, time, to, text }: Props = $props();

	const meta = $derived([time, to && m.together_to({ name: to })].filter(Boolean).join(' · '));
</script>

<article class="flex w-full gap-2.5">
	<PeerAvatar initials={who.initials} peer={who.peer} />
	<div class="flex min-w-0 flex-1 flex-col gap-1">
		<p class="flex flex-wrap items-center gap-1.5">
			<span class={['text-ui font-medium', who.peer === 1 ? 'text-peer-1' : who.peer === 2 ? 'text-peer-2' : 'text-text']}>{who.name}</span>
			{#if meta}<span class="font-mono text-label text-text-3">{meta}</span>{/if}
		</p>
		<p class={['rounded-3xl px-3.5 py-2.5 text-phone leading-cozy whitespace-pre-wrap text-text', who.peer === 1 ? 'bg-peer-1-bg' : who.peer === 2 ? 'bg-peer-2-bg' : 'bg-surface-2']}>{text}</p>
	</div>
</article>
