<script lang="ts">
	import CodeBlock from '$lib/ds/chat/CodeBlock.svelte';
	import DocPage from '$lib/docs/ui/DocPage.svelte';
	import DocSection from '$lib/docs/ui/DocSection.svelte';
	import DoDont from '$lib/docs/ui/DoDont.svelte';
	import Note from '$lib/docs/ui/Note.svelte';
	import { docHref } from '$lib/docs/href';

	// Get started → Contributing. The order a change to the design system takes: pen, exports, a
	// failing test, the component, the sheet, the proof, the docs, the gates, a PR (AGENTS.md §0,
	// §3–§8, pen/README.md, src/lib/docs).
	const steps: [title: string, body: string][] = [
		['Change the pen', 'How Nomi looks is decided in Nomi.pen, never in code first. A new component, state, colour or line of copy is drawn there, in both themes.'],
		['Re-export what the code is held to', 'Variables changed: re-export `tokens.json` (`tokens.test.ts` fails until `tokens.css` matches). A sheet’s sections or boards moved: re-export `pen/bounds.json` and `pen/export/` with the snippets in `pen/README.md`.'],
		['Read the frame', 'Through the Pencil tools: every padding, gap, alignment, stroke, fill, radius and text style, and the exact bounds. Never open the `.pen` file directly.'],
		['Write the failing test', 'A component test in real Chromium for the pen’s geometry, roles, keyboard and focus. New words need their keys in both message files, or `messages.test.ts` fails.'],
		['Build the component', 'In `src/lib/ds/<category>/`, starting with a comment that names the pen frame. Tokens only; every visible word a message in English and Indonesian.'],
		['Put it on the sheet', 'Add or update its section in `src/lib/gallery`, in the pen’s order, and its bounds in `pen/bounds.json`.'],
		['Prove it', 'With `npm run dev` running: `npm run pen:report`, then `pen:compare` and `pen:boxes` where it drifts. Look at it in dark and light, at 1512 and 390.'],
		['Update the docs', 'In the same PR: the component’s entry in `src/lib/docs/entries/<category>.ts` and any live example in `src/lib/docs/examples/<slug>/`. A new folder in `src/lib/ds` needs its category in `src/lib/docs/catalog.ts`.'],
		['Run the gates', 'The whole chain, in order. Report only what you actually ran and saw.'],
		['Open a pull request', 'From its own branch, never main. Wait for green CI and fix failures on the branch.'],
	];

	const gates = 'npm run check && npm run test:unit && npm run test:components && npm run build && npm run test:e2e';

	const entry = `// src/lib/docs/entries/<category>.ts
import type { ComponentDoc } from '../types';

export default [
	{
		file: 'actions/Button.svelte',
		summary: 'One or two sentences: what it is and what it is for.',
		use: ['When to reach for it.'],
		avoid: ['When not to, and what to use instead.'],
		dos: [],
		donts: [],
		content: ['How to write its labels.'],
		accessibility: ['Keyboard, screen reader, focus and motion notes.'],
		related: ['actions/IconButton.svelte'],
	},
] satisfies ComponentDoc[];`;
</script>

{#snippet rich(text: string)}{#each text.split('`') as part, index (index)}{#if index % 2}<code class="rounded-xs bg-surface-2 px-1 font-mono text-label text-text">{part}</code>{:else}{part}{/if}{/each}{/snippet}

<DocPage
	eyebrow="Get started"
	title="Contributing"
	lede="Changing the design system: pen first, then code, then proof. The order matters, because each step is checked against the one before it."
	wide
>
	<DocSection title="The order" lede="For a new component, a changed one, or a new section on a sheet.">
		<ol class="flex flex-col overflow-hidden rounded-md border border-border">
			{#each steps as [title, body], index (title)}
				<li class="flex gap-4 border-b border-border px-4 py-3.5 last:border-b-0">
					<span class="w-5 shrink-0 font-mono text-label leading-firm text-text">{String(index + 1).padStart(2, '0')}</span>
					<span class="flex min-w-0 flex-1 flex-col gap-0.5">
						<span class="text-ui font-medium text-text">{title}</span>
						<span class="text-caption leading-normal text-text-3">{@render rich(body)}</span>
					</span>
				</li>
			{/each}
		</ol>
	</DocSection>

	<DocSection title="When the pen looks wrong">
		<ul>
			<li><strong>The pen wins</strong> over the written rules and over taste. Where it breaks a rule (a 10px citation number, a 13px icon, an inline link without an underline), follow it and say so in the component’s comment.</li>
			<li><strong>Only an obvious slip</strong> (copy pasted from another frame, text spilling out of its box) may be left out, and only by adding it to the list in AGENTS.md §9 with the reason. Never silently. <a href={docHref('/design-system/get-started/designers', 'known-pen-slips')}>The current list</a>.</li>
			<li><strong>A value no token has</strong> becomes a named token in <code>src/app.css</code>, with a comment saying where the pen draws it.</li>
		</ul>
	</DocSection>

	<DocSection title="The docs entry" lede="Props and notes come from the source by themselves. The rest is written next to it.">
		<p>
			Each category’s file in <code>src/lib/docs/entries/</code> exports the docs of its components: a summary, when to use and avoid them, do’s
			and don’ts, how to write for them, accessibility notes and related components. <code>entries.test.ts</code> fails when an entry points at a
			component that doesn’t exist or its summary is empty or over 320 characters.
		</p>
		<CodeBlock code={entry} language="ts" />
		<p>
			The written pages (these guides, Foundations, Content) are in <code>src/routes/design-system/(docs)/</code> and listed in
			<code>src/lib/docs/nav.ts</code>; <code>nav.test.ts</code> fails on a link to a page that doesn’t exist.
		</p>
	</DocSection>

	<DocSection title="Gates">
		<CodeBlock code={gates} language="sh" />
		<p>
			Chained so a failure stops the chain; CI runs the same on every pull request that touches <code>frontend-new/</code>, including the
			pen-parity guard. Never report a gate as passing without having run it.
		</p>
	</DocSection>

	<DocSection title="Branches and pull requests">
		<ul>
			<li>Never commit or push to <code>main</code>. One branch per task (<code>feat/…</code>, <code>fix/…</code>), stacked on the previous one while it’s unmerged; rebase a stack with <code>git rebase --update-refs</code>.</li>
			<li>One pull request per branch, against <code>main</code> or the branch below it in a stack, saying what changed, which tests cover it and how it was validated.</li>
			<li>Wait for green CI. Fix failures on the branch.</li>
			<li>Change files only inside <code>frontend-new/</code> (and its workflow). Never commit secrets or <code>.env</code> files.</li>
			<li>Anything the UI needs that the backend API doesn’t have goes in <code>BACKEND_TODO.md</code>, checked against the real routes first, and is left out of the UI. Never faked.</li>
		</ul>
	</DocSection>

	<DocSection title="After it merges">
		<p>
			The team’s shared link is a static copy of these docs and the sheets. Rebuild it with <code>npm run build:gallery</code> and republish
			it after the design system changes. See <a href={docHref('/design-system/resources')}>Pen file &amp; tools</a>.
		</p>
		<Note><p>Agents follow the same order. Their rules are AGENTS.md (and CLAUDE.md for Claude Code), in <code>frontend-new/</code>.</p></Note>
	</DocSection>

	<DocSection title="Best practices">
		<DoDont
			dos={[
				'Draw it in the pen first, in both themes.',
				'Re-export bounds and HTML in the same PR when the pen changed.',
				'Write the test before the component.',
				'Update the docs entry in the same PR as the component.',
			]}
			donts={[
				'Change how something looks in code alone.',
				'Leave out a part of the pen without listing it in AGENTS.md §9.',
				'Push to main, or merge on red CI.',
				'Report a gate you didn’t run.',
			]}
		/>
	</DocSection>
</DocPage>
