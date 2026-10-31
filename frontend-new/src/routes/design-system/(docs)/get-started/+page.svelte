<script lang="ts">
	import DocPage from '$lib/docs/ui/DocPage.svelte';
	import DocSection from '$lib/docs/ui/DocSection.svelte';
	import LinkCard from '$lib/docs/ui/LinkCard.svelte';
	import Note from '$lib/docs/ui/Note.svelte';
	import { guides } from '$lib/docs/nav';
	import { components } from '$lib/docs/components';
	import { PATTERNS } from '$lib/docs/patterns';
	import { docHref } from '$lib/docs/href';
	import exported from '$lib/ds/tokens/tokens.json';

	// Get started → Overview. What the design system is and isn't, where its truth lives (AGENTS.md
	// §0, §5, §6), how these docs are organised, and a way in for each role.
	const roles = guides[0].links.slice(1);

	const chain: [title: string, body: string][] = [
		['Nomi.pen', 'The design file in pen.dev. Every colour, size, component, flow and line of copy is drawn here first. When anything below disagrees with it, the pen wins.'],
		['Snapshots of the pen', 'Its variables are exported to `tokens.json`; the position and size of every section and board on the three design sheets to `pen/bounds.json`; each section’s own HTML to `pen/export/`.'],
		['Tokens and components', 'The tokens become CSS variables and the only Tailwind classes there are. The components in `src/lib/ds` are built from them, one per pen component, each naming the frame it implements.'],
		['The sheets and these docs', 'The three design sheets are rebuilt from those components, section by section. These docs show the same components, with their props read from the source.'],
		['Held in CI', '`tokens.test.ts` fails when the CSS drifts from the exported tokens. `e2e/pen-parity.e2e.ts` fails when any section or board of a sheet moves off the pen by a single pixel.'],
	];

	const parts = [
		{ title: 'Get started', path: '/design-system/get-started', count: `${guides[0].links.length} pages`, blurb: 'This page, a guide for each role, and how to change the design system.' },
		{ title: 'Foundations', path: '/design-system/foundations/principles', count: `${guides[1].links.length} pages`, blurb: 'The principles, colour, the crew, type, space, radius, icons, themes, states and accessibility.' },
		{ title: 'Content', path: '/design-system/content/voice-and-tone', count: `${guides[2].links.length} pages`, blurb: 'How Nomi sounds: voice, writing, vocabulary, numbers and money, both languages.' },
		{ title: 'Components', path: '/design-system/components', count: `${components.length}`, blurb: 'Every component in src/lib/ds, live, with its props, notes and usage.' },
		{ title: 'Patterns', path: '/design-system/patterns', count: `${PATTERNS.length}`, blurb: 'The sections of the Chat responses and Build sheets: components combined for one job.' },
		{ title: 'Tokens', path: '/design-system/tokens', count: `${Object.keys(exported.tokens).length}`, blurb: 'Every value the design allows, its CSS variable, its class, dark and light.' },
		{ title: 'Pen file & tools', path: '/design-system/resources', count: 'Resources', blurb: 'Nomi.pen, the rules for agents, the pen tools and the shared static copy.' },
		{ title: 'Pen sheets', path: '/design-system/nomi', count: '3', blurb: 'The design sheets of Nomi.pen, rebuilt 1:1 from the real components.' },
	];
</script>

{#snippet rich(text: string)}{#each text.split('`') as part, index (index)}{#if index % 2}<code class="rounded-xs bg-surface-2 px-1 font-mono text-label text-text">{part}</code>{:else}{part}{/if}{/each}{/snippet}

<DocPage
	eyebrow="Get started"
	title="Overview"
	lede="Nomi’s design system is the visual language, the components and the writing rules of the new Nomi frontend, held to the Nomi.pen design file. This page says what it covers, where its truth lives and where to begin."
	wide
>
	<DocSection title="What it is">
		<p>
			The design system is how Nomi looks and sounds, written down as code: the design tokens, {components.length} Svelte components in
			<code>src/lib/ds</code>, the patterns they make together on the Chat responses and Build sheets, and the rules for words in English
			and Bahasa Indonesia. It is the frontend in <code>frontend-new/</code>, not a copy of it: every example in these docs is the component
			that ships.
		</p>
		<p>
			Its character is <strong>“An instrument, not a toy”</strong>: calm, dense, precise. Neutral surfaces do the hierarchy, one mint accent
			marks the thing to act on, and anything a machine produced is set in mono. The
			<a href={docHref('/design-system/foundations/principles')}>five principles</a> decide when in doubt.
		</p>
	</DocSection>

	<DocSection title="What it isn’t">
		<ul>
			<li><strong>Not a second design.</strong> The pen is the source of truth. If these docs and Nomi.pen disagree, the pen is right and the docs are fixed.</li>
			<li><strong>Not a mock-up kit.</strong> Nothing here is drawn twice. Components, sheets and examples render the same code the app uses.</li>
			<li><strong>Not the app’s logic.</strong> Components are presentational only: no fetching, no API types. Pages adapt the backend’s data to their props.</li>
			<li><strong>Not the old app.</strong> <code>frontend/</code> is the old frontend. It is read for business logic, never imported or styled from.</li>
			<li><strong>Not a brand book.</strong> Logo, campaign and marketing rules aren’t drawn in the pen yet. <a href={docHref('/design-system/get-started/content')}>For content &amp; marketing</a> says what is and isn’t defined.</li>
		</ul>
	</DocSection>

	<DocSection title="Where truth lives" lede="Each step follows the one before it, and CI checks that it still does.">
		<ol class="flex flex-col overflow-hidden rounded-md border border-border">
			{#each chain as [title, body], index (title)}
				<li class="flex gap-4 border-b border-border px-4 py-3.5 last:border-b-0">
					<span class="font-mono text-label leading-firm text-text">0{index + 1}</span>
					<span class="flex min-w-0 flex-1 flex-col gap-0.5">
						<span class="text-ui font-medium text-text">{title}</span>
						<span class="text-caption leading-normal text-text-3">{@render rich(body)}</span>
					</span>
				</li>
			{/each}
		</ol>
		<p>
			So a change to how Nomi looks starts in the pen, never in code. <a href={docHref('/design-system/get-started/contributing')}>Contributing</a>
			walks through the order.
		</p>
	</DocSection>

	<DocSection title="How the docs are organised" lede="The sidebar follows the same order.">
		<div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
			{#each parts as part (part.path)}
				<LinkCard href={docHref(part.path)} eyebrow={part.count} title={part.title} blurb={part.blurb} />
			{/each}
		</div>
		<p>
			Foundations and Content are written pages. Components and Patterns are generated from the code and the sheets, so a new component or
			section appears without anyone adding a page. Search (<code>⌘K</code>) finds any of them.
		</p>
	</DocSection>

	<DocSection title="Start with your role">
		<div class="grid gap-3 sm:grid-cols-2">
			{#each roles as role (role.path)}
				<LinkCard href={docHref(role.path)} title={role.title} blurb={role.blurb} />
			{/each}
		</div>
		<Note>
			<p>
				The docs and the sheets run in <code>npm run dev</code> and wherever <code>PUBLIC_DS_GALLERY=1</code> is set; they are off in
				production. The team shares a static copy, built with <code>npm run build:gallery</code>. See
				<a href={docHref('/design-system/resources')}>Pen file &amp; tools</a>.
			</p>
		</Note>
	</DocSection>
</DocPage>
