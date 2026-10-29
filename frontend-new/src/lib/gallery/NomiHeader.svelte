<script lang="ts">
	import Mascot from '$lib/ds/agents/Mascot.svelte';

	/**
	 * The Nomi sheet's DS Header: the title block beside the five principles, a divider under both.
	 */
	const principles: [title: string, body: string][] = [
		['One accent, under 10%', 'Mint marks the single thing to act on. Neutrals do the hierarchy.'],
		['Mono for machine output', 'Paths, IDs, URLs, tokens, costs, timestamps, model names.'],
		['Borders over shadows', 'Shadows only for things that truly float: menus, toasts, dialogs.'],
		['Honest states', 'Show the real tool and target. No fake progress. Cost always visible.'],
		['Mascots are identity, not decoration', 'One per agent, flat colour, used only where an agent is speaking or working.'],
	];
</script>

<header class="flex flex-col gap-12 border-b border-border pb-9.75 lg:flex-row">
	<div class="flex min-w-0 flex-1 flex-col gap-3">
		<p class="flex items-center gap-2.5">
			<Mascot agent="nomi" size={32} />
			<span class="font-mono text-label tracking-label text-text-3">NOMI DESIGN SYSTEM · v1.0 · 21 OCT 2026</span>
		</p>
		<h1 class="text-display font-medium tracking-display text-text">An instrument, not a toy.</h1>
		<p class="w-160 max-w-full text-body leading-normal text-text-2">
			Neutral surfaces, one accent, mono for anything a machine produced. Every value below is a token in this file, themed for dark and
			light.
		</p>
	</div>
	<ol class="flex w-140 max-w-full shrink-0 flex-col overflow-hidden rounded-md border border-border">
		{#each principles as [title, body], index (title)}
			<li class="flex gap-4 border-b border-border px-3.75 pt-3 pb-2.75 first:pt-2.75 last:border-b-0">
				<span class="font-mono text-label text-text">0{index + 1}</span>
				<span class="flex min-w-0 flex-1 flex-col gap-0.5">
					<span class="text-ui font-medium text-text">{title}</span>
					<span class="text-caption text-text-3">{body}</span>
				</span>
			</li>
		{/each}
	</ol>
</header>
