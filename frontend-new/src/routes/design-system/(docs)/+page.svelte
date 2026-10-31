<script lang="ts">
	import Button from '$lib/ds/actions/Button.svelte';
	import Mascot from '$lib/ds/agents/Mascot.svelte';
	import SectionLabel from '$lib/ds/status/SectionLabel.svelte';
	import DocSection from '$lib/docs/ui/DocSection.svelte';
	import LinkCard from '$lib/docs/ui/LinkCard.svelte';
	import { guides } from '$lib/docs/nav';
	import { components } from '$lib/docs/components';
	import { PATTERNS } from '$lib/docs/patterns';
	import { PRINCIPLES } from '$lib/docs/principles';
	import { docHref } from '$lib/docs/href';
	import exported from '$lib/ds/tokens/tokens.json';

	// The front door: what Nomi's design system is, a way in for each kind of reader, the
	// principles, and everything that's inside.
	const audiences = guides[0].links.slice(1);
	const inside = [
		{ title: 'Foundations', path: '/design-system/foundations/principles', count: `${guides[1].links.length} pages`, blurb: 'Colour, type, space, radius, icons, the crew, themes and states.' },
		{ title: 'Components', path: '/design-system/components', count: `${components.length}`, blurb: 'Every component, live, with its props, rules and writing guidance.' },
		{ title: 'Patterns', path: '/design-system/patterns', count: `${PATTERNS.length}`, blurb: 'How components combine for one job, from a reply to a live app.' },
		{ title: 'Content', path: '/design-system/content/voice-and-tone', count: `${guides[2].links.length} pages`, blurb: 'How Nomi sounds: voice, writing, vocabulary, numbers, languages.' },
		{ title: 'Tokens', path: '/design-system/tokens', count: `${Object.keys(exported.tokens).length}`, blurb: 'Every value the design allows, with its class and both themes.' },
		{ title: 'Pen sheets', path: '/design-system/nomi', count: '3', blurb: 'The design sheets of Nomi.pen, rebuilt from the real components.' },
	];
</script>

<svelte:head><title>Nomi design system</title></svelte:head>

<div class="flex w-full max-w-300 flex-col gap-16">
	<header class="flex flex-col gap-5 border-b border-border pb-12">
		<p class="flex items-center gap-2.5">
			<Mascot agent="nomi" size={32} />
			<SectionLabel label="Nomi design system" />
		</p>
		<h1 class="text-display font-medium tracking-display text-text">An instrument, not a toy.</h1>
		<p class="max-w-170 text-body leading-relaxed text-text-2">
			The design system behind Nomi: neutral surfaces, one mint accent, mono for anything a machine produced. Everything here is the real
			code of the new frontend, held to the Nomi.pen design pixel for pixel, for designers, developers, writers, marketing and anyone
			who shapes how Nomi looks and sounds.
		</p>
		<div class="flex flex-wrap gap-2 pt-1">
			<Button label="Get started" href={docHref('/design-system/get-started')} />
			<Button variant="secondary" label="Browse components" href={docHref('/design-system/components')} />
		</div>
	</header>

	<DocSection title="Start here" lede="A way in for each of us.">
		<div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
			{#each audiences as audience (audience.path)}
				<LinkCard href={docHref(audience.path)} title={audience.title} blurb={audience.blurb} />
			{/each}
		</div>
	</DocSection>

	<DocSection title="Principles" lede="Five rules every screen follows. When in doubt, these decide.">
		<ol class="flex flex-col overflow-hidden rounded-md border border-border">
			{#each PRINCIPLES as [title, body], index (title)}
				<li class="flex gap-4 border-b border-border px-4 py-3.5 last:border-b-0">
					<span class="font-mono text-label leading-firm text-text">0{index + 1}</span>
					<span class="flex min-w-0 flex-1 flex-col gap-0.5">
						<span class="text-ui font-medium text-text">{title}</span>
						<span class="text-caption text-text-3">{body}</span>
					</span>
				</li>
			{/each}
		</ol>
		<p class="text-ui"><a class="text-text underline decoration-border-strong underline-offset-3 hover:decoration-text" href={docHref('/design-system/foundations/principles')}>What each principle means in practice</a></p>
	</DocSection>

	<DocSection title="What's inside">
		<div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
			{#each inside as part (part.path)}
				<LinkCard href={docHref(part.path)} eyebrow={part.count} title={part.title} blurb={part.blurb} />
			{/each}
		</div>
	</DocSection>

	<DocSection title="Held to the pen">
		<p class="max-w-170 text-body leading-relaxed text-text-2">
			Nomi.pen is the source of truth. The three design sheets are rebuilt here from the real components, and a check on every pull request
			fails if any section or board moves off the pen by a single pixel. Props tables are read from the code when the docs are built, so
			they can't fall behind it.
		</p>
	</DocSection>
</div>
