<script lang="ts">
	import { ArrowUpDown, LoaderCircle } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import PeerAvatar from '../build/PeerAvatar.svelte';
	import PhoneButton from './PhoneButton.svelte';

	// Mobile / Request Queue: two people, one Koda, one change at a time, on a phone. Each ask with
	// who asked and where it is; the next one can go first with a thumb-sized button.
	type Ask = { text: string; who: { name: string; initials: string; peer?: 1 | 2 }; state: 'running' | 'next' | 'queued'; meta?: string };
	type Props = { asks: Ask[]; onfirst?: (index: number) => void };
	let { asks, onfirst }: Props = $props();

	const next = $derived(asks.findIndex((ask) => ask.state === 'next'));
</script>

<section class="flex w-full flex-col gap-3 rounded-3xl border border-border bg-surface p-3.75">
	<p class="flex items-center gap-2">
		<Mascot agent="koda" size={20} />
		<span class="min-w-0 flex-1 text-body font-medium text-text">{asks.length === 1 ? m.queue_title_one() : m.queue_title({ count: String(asks.length) })}</span>
	</p>
	<ol class="flex flex-col gap-3">
		{#each asks as ask, index (index)}
			<li class="flex items-center gap-2.5 rounded-lg bg-surface-2 px-3 py-2.5">
				<PeerAvatar initials={ask.who.initials} peer={ask.who.peer} />
				<span class="flex min-w-0 flex-1 flex-col gap-0.5">
					<span class="text-body leading-firm text-text">{ask.text}</span>
					{#if ask.meta}<span class="font-mono text-label text-text-3">{ask.meta}</span>{/if}
				</span>
				{#if ask.state === 'running'}
					<span class="inline-flex shrink-0 animate-spin text-text-2 motion-reduce:animate-none"><Icon icon={LoaderCircle} size={18} /></span>
				{/if}
			</li>
		{/each}
	</ol>
	<p class="text-ui text-text-3">{m.queue_hint_short()}</p>
	{#if onfirst && next > 0}
		<PhoneButton label={m.queue_go_first({ name: asks[next].who.name.split(/\s+/)[0] })} variant="secondary" icon={ArrowUpDown} onclick={() => onfirst(next)} />
	{/if}
</section>
