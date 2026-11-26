<script lang="ts">
	import { Users, X } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Button from '../actions/Button.svelte';
	import IconButton from '../actions/IconButton.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import Status from '../status/Status.svelte';
	import Tag from '../status/Tag.svelte';
	import PeerAvatar from '../build/PeerAvatar.svelte';

	// Flow / Build — Invite & build together (TnwQT) → step 9 "Two asks at once": the Chat / Request
	// Queue (t2BII) at the end of a project's chat. Two people, one Koda, one change at a time: each
	// ask with who asked and where it is. Drawn as build/RequestQueue (the old app's twin, kept as
	// it is), except for whose turn the footer's button moves: an ask already running can't wait,
	// and the next one is next already, so "Do Sari's first" moves `first`, a teammate's ask waiting
	// behind one of the person's own, and shows only when there is one (AGENTS.md §9).
	type Ask = { text: string; who: { name: string; initials: string; peer?: 1 | 2 }; state: 'running' | 'next' | 'queued'; meta?: string };
	type Props = {
		asks: Ask[];
		/** The index of the ask that can go first; none when it's left out. */
		first?: number;
		onfirst?: (index: number) => void;
		/** Removes a waiting (`next` or `queued`) ask; a `running` ask can never be removed. */
		onremove?: (index: number) => void;
	};
	let { asks, first, onfirst, onremove }: Props = $props();

	const movable = $derived(first !== undefined && first >= 0 && first < asks.length ? asks[first] : null);
</script>

<section aria-label={asks.length === 1 ? m.queue_title_one() : m.queue_title({ count: String(asks.length) })} class="flex w-full flex-col overflow-hidden rounded-md bg-surface outline -outline-offset-1 outline-border">
	<header class="flex items-center gap-2 px-3.5 py-2.5">
		<Mascot agent="koda" size={20} />
		<p class="text-ui font-medium text-text">{asks.length === 1 ? m.queue_title_one() : m.queue_title({ count: String(asks.length) })}</p>
		<span class="min-w-0 flex-1 text-right font-mono text-label text-text-3">{m.queue_meta()}</span>
	</header>
	<ol>
		{#each asks as ask, index (index)}
			<li class="flex items-center gap-2.5 border-t border-border px-3.5 pt-2.25 pb-2.5">
				<span class="w-3 shrink-0 font-mono text-label text-text-3">{index + 1}</span>
				<PeerAvatar initials={ask.who.initials} peer={ask.who.peer} size={20} />
				<span class="flex min-w-0 flex-1 flex-col gap-0.5">
					<span class="text-ui text-text">{ask.text}</span>
					{#if ask.meta}<span class="font-mono text-label text-text-3">{ask.meta}</span>{/if}
				</span>
				{#if ask.state === 'running'}
					<Status kind="working" agent="koda" label={m.build_working({ name: 'Koda' })} />
				{:else}
					{#if ask.state === 'next'}
						<Tag kind="neutral" label={m.queue_next()} />
					{/if}
					{#if onremove}
						<IconButton icon={X} iconSize={14} label={m.queue_remove()} variant="ghost" onclick={() => onremove(index)} />
					{/if}
				{/if}
			</li>
		{/each}
	</ol>
	<footer class="flex flex-wrap items-center gap-2 border-t border-border px-3.5 pt-1.75 pb-2">
		<span class="min-w-0 flex-1 basis-48 text-caption text-text-3">{m.queue_hint({ agent: 'Koda' })}</span>
		{#if onfirst && movable && first !== undefined}
			<Button label={m.queue_first({ name: movable.who.name.split(/\s+/)[0] })} variant="ghost" iconStart={Users} onclick={() => onfirst(first)} />
		{/if}
	</footer>
</section>
