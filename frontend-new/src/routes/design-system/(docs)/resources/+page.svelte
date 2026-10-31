<script lang="ts">
	import CodeBlock from '$lib/ds/chat/CodeBlock.svelte';
	import DocPage from '$lib/docs/ui/DocPage.svelte';
	import DocSection from '$lib/docs/ui/DocSection.svelte';
	import LinkCard from '$lib/docs/ui/LinkCard.svelte';
	import Note from '$lib/docs/ui/Note.svelte';
	import { docHref } from '$lib/docs/href';
	import exported from '$lib/ds/tokens/tokens.json';
	import bounds from '../../../../../pen/bounds.json';

	// Resources → Pen file & tools. Nomi.pen and its sheets, the rules for agents (AGENTS.md,
	// CLAUDE.md), the pen snapshots and tools (pen/README.md), and the shared static copy (§10).
	const sheets = [
		{ id: 'YOYH1', title: 'Nomi', path: '/design-system/nomi', blurb: 'Design System / Nomi: foundations and core components.' },
		{ id: 'Ljvtr', title: 'Chat responses', path: '/design-system/chat', blurb: 'Design System / Chat responses: everything Nomi can put in a reply.' },
		{ id: 'DcG0H', title: 'Build', path: '/design-system/build', blurb: 'Design System / Build: from describing an app to a live URL.' },
	];
	const sections = Object.values(bounds.sheets).reduce((sum, sheet) => sum + sheet.sections.length, 0);

	const rules: [part: string, about: string][] = [
		['§0', 'The one rule: the pen, 1:1.'],
		['§1', 'Boundaries: only frontend-new/, the backend API as it is, no secrets.'],
		['§2', 'The visual language: character, colour, type, space, icons, layout, states, copy, don’ts.'],
		['§3', 'From the pen to code: strokes, layout, open states, components, missing values.'],
		['§4', 'Components and code: where, presentational only, runes, Tailwind, theme, accessibility, i18n, server.'],
		['§5', 'The gallery: the sheets, sections and boards.'],
		['§6', 'Proving 1:1: read the frame, test first, compare, CI.'],
		['§7', 'Tests and gates.'],
		['§8', 'Git and pull requests.'],
		['§9', 'Known pen slips we don’t copy.'],
		['§10', 'Running, and the static copy.'],
	];

	const snapshots: [file: string, what: string, usedBy: string][] = [
		['src/lib/ds/tokens/tokens.json', `The pen’s variables: ${Object.keys(exported.tokens).length} tokens, exported ${exported.exported}.`, '`tokens.test.ts` keeps `tokens.css` identical to it.'],
		['pen/bounds.json', `Every section (${sections}) and board of the three sheets, at the sheet’s 1600 width; exported ${bounds.exported}.`, '`e2e/pen-parity.e2e.ts` in CI and `npm run pen:report`.'],
		['pen/export/<sheet>--<section>.html', 'The pen’s own HTML export of each section.', '`npm run pen:compare`, as the text-by-text reference.'],
	];

	const tools = `npm run pen:report                                  # every section and board against pen/bounds.json
npm run pen:compare -- build workspace              # every text in one section against pen/export/
npm run pen:compare -- chat rich-content --all      # every line, not only where the offset changes
npm run pen:boxes -- nomi "section[aria-label=Rules] section" 2   # an element's boxes`;

	const frame = `Print(JSON.stringify(Get("<frame id>", {depth: 3, resolveInstances: true})));
Get("<frame id>", (n, c) => { if (c.depth <= 3) Print("  ".repeat(c.depth) + n.name, c.bounds); }, {resolveInstances: true});`;

	const share = `npm run build:gallery   # the docs and sheets as static files in build-gallery/`;
</script>

{#snippet rich(text: string)}{#each text.split('`') as part, index (index)}{#if index % 2}<code class="rounded-xs bg-surface-2 px-1 font-mono text-label text-text">{part}</code>{:else}{part}{/if}{/each}{/snippet}

<DocPage eyebrow="Resources" title="Pen file & tools" lede="Nomi.pen and its three sheets, the rules every person and agent follows, the snapshots the code is held to, the tools that compare them, and the copy of these docs the team shares." wide>
	<DocSection title="Nomi.pen">
		<p>
			The design file, made in pen.dev and opened in the Pencil app. It is the source of truth for how Nomi looks: three design sheets, 40
			flows each with a light twin, and the app screens. The code repository doesn’t keep the file; it keeps snapshots of it (below).
		</p>
		<ul>
			<li>People open it in the Pencil app.</li>
			<li>Agents reach it only through the Pencil tools (<code>get_app_state</code>, <code>execute</code>, <code>read_skill</code>). They never open a <code>.pen</code> file with a text reader or the shell, and never change the design unless asked.</li>
			<li>Where the pen is silent on behaviour, timings or copy, the prototype <code>marioprasetyo/nomi-prototype</code> is the reference.</li>
		</ul>
	</DocSection>

	<DocSection title="The sheets" lede="Rebuilt from the real components and held to the pen by CI. Add ?theme=dark or ?theme=light to force a theme.">
		<div class="grid gap-3 md:grid-cols-3">
			{#each sheets as sheet (sheet.id)}
				<LinkCard href={docHref(sheet.path)} eyebrow={sheet.id} title={sheet.title} blurb={sheet.blurb} />
			{/each}
		</div>
	</DocSection>

	<DocSection title="The rules" lede="In frontend-new/, next to the code.">
		<ul>
			<li><code>AGENTS.md</code>: the rules for every agent and every person working in the folder. Read before changing anything.</li>
			<li><code>CLAUDE.md</code>: points Claude Code at AGENTS.md and adds its own steps: read a frame with the Pencil tools before building, then finish with <code>npm run pen:report</code> and the full gate chain.</li>
			<li><code>pen/README.md</code>: the snapshots, the comparing tools and the snippets that refresh them.</li>
			<li><code>README.md</code>: running, checking and deploying the app. <code>BACKEND_TODO.md</code>: what the designed flows need from the backend that it doesn’t have yet.</li>
		</ul>
		<div class="overflow-x-auto rounded-md outline -outline-offset-1 outline-border">
			<table class="w-full min-w-170 border-collapse text-left">
				<thead>
					<tr class="border-b border-border bg-surface font-mono text-label tracking-label text-text-3 uppercase">
						<th scope="col" class="w-20 px-4 py-2.5 font-normal">AGENTS.md</th>
						<th scope="col" class="px-4 py-2.5 font-normal">What it covers</th>
					</tr>
				</thead>
				<tbody>
					{#each rules as [part, about] (part)}
						<tr class="border-b border-border align-top last:border-b-0">
							<th scope="row" class="px-4 py-2.5 text-left font-mono text-label font-normal text-text-3">{part}</th>
							<td class="px-4 py-2.5 text-ui leading-normal text-text-2">{about}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	</DocSection>

	<DocSection title="Snapshots of the pen" lede="Exported from Nomi.pen, never edited by hand. Re-export them in the same PR when the pen changes.">
		<div class="overflow-x-auto rounded-md outline -outline-offset-1 outline-border">
			<table class="w-full min-w-200 border-collapse text-left">
				<thead>
					<tr class="border-b border-border bg-surface font-mono text-label tracking-label text-text-3 uppercase">
						<th scope="col" class="w-72 px-4 py-2.5 font-normal">File</th>
						<th scope="col" class="px-4 py-2.5 font-normal">What</th>
						<th scope="col" class="w-72 px-4 py-2.5 font-normal">Used by</th>
					</tr>
				</thead>
				<tbody>
					{#each snapshots as [file, what, usedBy] (file)}
						<tr class="border-b border-border align-top last:border-b-0">
							<th scope="row" class="px-4 py-3 text-left font-mono text-label leading-normal font-normal break-all text-text">{file}</th>
							<td class="px-4 py-3 text-ui leading-normal text-text-2">{what}</td>
							<td class="px-4 py-3 text-ui leading-normal text-text-2">{@render rich(usedBy)}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
		<p>
			Every token, with its class and both values, is on the <a href={docHref('/design-system/tokens')}>Tokens</a> page. The snippets that
			refresh <code>bounds.json</code> and <code>export/</code> through the Pencil <code>execute</code> tool are in <code>pen/README.md</code>.
		</p>
	</DocSection>

	<DocSection title="The pen tools" lede="Run with the dev server up (npm run dev, port 5175; add --url for another).">
		<CodeBlock code={tools} language="sh" />
		<ul>
			<li><code>pen:report</code> lists every section and board against <code>pen/bounds.json</code>.</li>
			<li><code>pen:compare</code> renders the pen’s export in Chromium with local Geist and the pen’s whole-pixel line heights, and compares every text; the first line marked ◀ is where the drift starts.</li>
			<li><code>pen:boxes</code> prints an element’s boxes, to set next to the frame’s own bounds.</li>
		</ul>
		<p>Before building a component, read its frame’s exact layout through the Pencil <code>execute</code> tool:</p>
		<CodeBlock code={frame} language="js" />
		<Note>
			<p>
				The export is a good reference, not a perfect one: it collapses trailing spaces between inline runs and mis-sizes negative gaps and
				some fill widths. When a line looks wrong, check the frame’s own bounds in the pen before changing a component.
			</p>
		</Note>
	</DocSection>

	<DocSection title="The shared static copy" lede="The link the team opens.">
		<CodeBlock code={share} language="sh" />
		<ul>
			<li>Builds the docs and the three sheets, alone, as static files in <code>build-gallery/</code>. They work from any path on any static host; <code>e2e/gallery-static.e2e.ts</code> and <code>e2e/docs.e2e.ts</code> serve them under a nested path to prove it.</li>
			<li>Internal links end in <code>.html</code> there (every docs link goes through <code>docHref</code>), and assets sit in <code>app/</code> because share hosts can reserve names starting with <code>_</code>.</li>
			<li>Rebuild and republish it after the design system changes.</li>
		</ul>
	</DocSection>
</DocPage>
