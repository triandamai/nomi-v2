<script lang="ts">
	import type { Snippet } from 'svelte';
	import { GitFork } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Mascot from '../agents/Mascot.svelte';
	import Button from '../actions/Button.svelte';
	import type { CrewId } from '../agents/crew';

	// Design System / Projects → Project / Remix Card (htwHt): an app someone shared, to start from.
	// One grid column (384) wide: a 104 preview on its maker's crew tint (16 / 40 in, cut at the
	// bottom), then its name over who made it (the crew who built it, 20px, overlapping 6, and the
	// maker in mono), and Remix, under a line drawn inside. The maker line is cut short where the
	// pen lets it run past its column (AGENTS.md §9).
	type Props = {
		name: string;
		/** "Bu Sari · Yogyakarta · 214 remixes". */
		maker: string;
		/** The crew who built it. */
		crew?: CrewId[];
		/** The preview's tint. */
		tone: CrewId;
		/** A picture of the app. */
		preview: Snippet;
		onremix: () => void;
	};
	let { name, maker, crew = [], tone, preview, onremix }: Props = $props();
</script>

<article aria-label={name} class="flex w-96 max-w-full flex-col overflow-hidden rounded-md bg-surface outline -outline-offset-1 outline-border-strong">
	<div class="flex h-26 overflow-hidden px-10 pt-4" style="background: var(--crew-{tone}-bg)">{@render preview()}</div>
	<div class="flex items-center gap-3 border-t border-border px-4 pt-2.75 pb-3">
		<div class="flex min-w-0 flex-1 flex-col gap-1">
			<h3 class="truncate text-ui font-medium text-text">{name}</h3>
			<p class="flex min-w-0 items-center gap-2">
				{#if crew.length}
					<span class="flex shrink-0">
						{#each crew as agent, index (agent)}<span class={['inline-flex', index > 0 && '-ml-1.5']}><Mascot {agent} size={20} /></span>{/each}
					</span>
				{/if}
				<span class="truncate font-mono text-label text-text-3">{maker}</span>
			</p>
		</div>
		<Button variant="secondary" label={m.projects_remix()} iconEnd={GitFork} aria-label={m.projects_remix_label({ name })} onclick={onremix} />
	</div>
</article>
