<script lang="ts">
	import { Users } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Button from '../actions/Button.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import Status from '../status/Status.svelte';
	import Tag from '../status/Tag.svelte';
	import PeerAvatar from './PeerAvatar.svelte';

	// Build together → Asks in line: two people, one Koda, one change at a time. Each ask with who
	// asked and where it is; the next one can be moved up.
	type Ask = { text: string; who: { name: string; initials: string; peer?: 1 | 2 }; state: 'running' | 'next' | 'queued'; meta?: string };
	type Props = { asks: Ask[]; onfirst?: (index: number) => void };
	let { asks, onfirst }: Props = $props();

	const next = $derived(asks.findIndex((ask) => ask.state === 'next'));
</script>

<section class="flex w-full flex-col overflow-hidden rounded-md border border-border bg-surface">
	<header class="flex items-center gap-2 px-3.5 py-2.5">
		<Mascot agent="koda" size={20} />
		<p class="text-ui font-medium text-text">{asks.length === 1 ? m.queue_title_one() : m.queue_title({ count: String(asks.length) })}</p>
		<span class="min-w-0 flex-1 text-right font-mono text-label text-text-3">{m.queue_meta()}</span>
	</header>
	<ol>
		{#each asks as ask, index (index)}
			<li class="flex items-center gap-2.5 border-t border-border px-3.5 py-2.5">
				<span class="w-3 shrink-0 font-mono text-label text-text-3">{index + 1}</span>
				<PeerAvatar initials={ask.who.initials} peer={ask.who.peer} size={20} />
				<span class="flex min-w-0 flex-1 flex-col gap-0.5">
					<span class="text-ui text-text">{ask.text}</span>
					{#if ask.meta}<span class="font-mono text-label text-text-3">{ask.meta}</span>{/if}
				</span>
				{#if ask.state === 'running'}
					<Status kind="working" agent="koda" label={m.build_working({ name: 'Koda' })} />
				{:else if ask.state === 'next'}
					<Tag kind="neutral" label={m.queue_next()} />
				{/if}
			</li>
		{/each}
	</ol>
	<footer class="flex flex-wrap items-center gap-2 border-t border-border px-3.5 py-2">
		<span class="min-w-0 flex-1 basis-48 text-caption text-text-3">{m.queue_hint({ agent: 'Koda' })}</span>
		{#if onfirst && next > 0}
			<Button label={m.queue_first({ name: asks[next].who.name.split(/\s+/)[0] })} variant="ghost" iconStart={Users} onclick={() => onfirst(next)} />
		{/if}
	</footer>
</section>
