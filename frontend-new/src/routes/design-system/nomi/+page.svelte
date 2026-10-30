<script lang="ts">
	import ColourSection from '$lib/gallery/ColourSection.svelte';
	import CrewColourSection from '$lib/gallery/CrewColourSection.svelte';
	import TypeSection from '$lib/gallery/TypeSection.svelte';
	import SpaceSection from '$lib/gallery/SpaceSection.svelte';
	import ActionsSection from '$lib/gallery/ActionsSection.svelte';
	import InputsSection from '$lib/gallery/InputsSection.svelte';
	import NavigationSection from '$lib/gallery/NavigationSection.svelte';
	import ConversationSection from '$lib/gallery/ConversationSection.svelte';
	import OverlaysSection from '$lib/gallery/OverlaysSection.svelte';
	import ShellsSection from '$lib/gallery/ShellsSection.svelte';
	import RulesSection from '$lib/gallery/RulesSection.svelte';
	import AgentsSection from '$lib/gallery/AgentsSection.svelte';

	const principles: [title: string, body: string][] = [
		['One accent, under 10%', 'Mint marks the single thing to act on. Neutrals do the hierarchy.'],
		['Mono for machine output', 'Paths, IDs, URLs, tokens, costs, timestamps, model names.'],
		['Borders over shadows', 'Shadows only for things that truly float: menus, toasts, dialogs.'],
		['Honest states', 'Show the real tool and target. No fake progress. Cost always visible.'],
		['Mascots are identity, not decoration', 'One per agent, flat colour, used only where an agent is speaking or working.'],
	];
</script>

<header class="flex justify-between gap-12 py-12">
	<div class="flex max-w-xl flex-col gap-3">
		<p class="font-mono text-label tracking-label text-text-3 uppercase">Nomi design system · v1.0</p>
		<h1 class="text-display font-medium tracking-display text-text">An instrument, not a toy.</h1>
		<p class="text-body text-text-2">
			Neutral surfaces, one accent, mono for anything a machine produced. Every value below is a token in this file, themed for dark
			and light.
		</p>
	</div>
	<ol class="flex w-96 shrink-0 flex-col rounded-md border border-border">
		{#each principles as [title, body], index (title)}
			<li class="flex gap-3 border-b border-border p-3 last:border-b-0">
				<span class="font-mono text-label text-text-3">0{index + 1}</span>
				<span class="flex flex-col gap-1">
					<span class="text-ui font-medium text-text">{title}</span>
					<span class="text-caption text-text-3">{body}</span>
				</span>
			</li>
		{/each}
	</ol>
</header>

<ColourSection />
<CrewColourSection />
<TypeSection />
<SpaceSection />
<ActionsSection />
<InputsSection />
<NavigationSection />
<ConversationSection />
<OverlaysSection />
<ShellsSection />
<RulesSection />
<AgentsSection />
