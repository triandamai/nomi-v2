<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import Button from '$lib/ds/actions/Button.svelte';
	import Status from '$lib/ds/status/Status.svelte';
	import Segmented from '$lib/ds/inputs/Segmented.svelte';
	import CodeBlock from '$lib/ds/chat/CodeBlock.svelte';
	import DocPage from '$lib/docs/ui/DocPage.svelte';
	import DocSection from '$lib/docs/ui/DocSection.svelte';
	import DoDont from '$lib/docs/ui/DoDont.svelte';
	import Example from '$lib/docs/ui/Example.svelte';
	import Note from '$lib/docs/ui/Note.svelte';
	import { CATEGORIES } from '$lib/docs/catalog';
	import { highlight } from '$lib/docs/highlight';
	import { docHref } from '$lib/docs/href';

	// Get started → For developers. Running it, where components live, composing them, tokens,
	// theming, accessibility, i18n, tests and gates, proving 1:1, and how the docs read the source
	// (AGENTS.md §4–§7, §10; pen/README.md; src/lib/docs/api).
	const run = `npm ci
npx playwright install chromium   # once, for the component tests
npm run dev                        # http://localhost:5175/design-system`;

	const useCode = `<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import Button from '$lib/ds/actions/Button.svelte';
	import Status from '$lib/ds/status/Status.svelte';

	let { onstop }: { onstop: () => void } = $props();
<\/script>

<Status kind="working" agent="koda" label={m.build_working({ name: 'Koda' })} />
<Button variant="secondary" label={m.build_stop()} onclick={onstop} />`;

	const bindCode = `<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import Segmented from '$lib/ds/inputs/Segmented.svelte';

	let view = $state('chat');
<\/script>

<Segmented
	label={m.ws_switch()}
	bind:value={view}
	options={[
		{ value: 'chat', label: m.ws_chat() },
		{ value: 'preview', label: m.ws_preview() },
	]}
/>`;

	const message = `// messages/en.json, at the end
"build_working": "{name} is building"

// messages/id.json, at the end, written at the same time
"build_working": "{name} sedang membangun"

// in a component
import { m } from '$lib/paraglide/messages';
m.build_working({ name: 'Koda' }); // Koda is building`;

	const gates = 'npm run check && npm run test:unit && npm run test:components && npm run build && npm run test:e2e';

	const pen = `npm run pen:report                                  # every section and board against pen/bounds.json
npm run pen:compare -- build workspace              # every text in one section against pen/export/
npm run pen:compare -- chat rich-content --all      # every line, not only where the offset changes
npm run pen:boxes -- nomi "section[aria-label=Rules] section" 2   # an element's boxes`;

	const frame = `Print(JSON.stringify(Get("<frame id>", {depth: 3, resolveInstances: true})));
Get("<frame id>", (n, c) => { if (c.depth <= 3) Print("  ".repeat(c.depth) + n.name, c.bounds); }, {resolveInstances: true});`;

	let view = $state('chat');
	const noop = () => {};
</script>

<DocPage
	eyebrow="Get started"
	title="For developers"
	lede="The components are the design, in Svelte 5 and Tailwind v4 with the pen’s tokens and nothing else. Use them, compose them, and prove a screen matches the pen before it ships."
	wide
>
	<DocSection title="Run it" lede="Node 22.12 or newer, inside frontend-new/.">
		<CodeBlock code={run} language="sh" />
		<ul>
			<li>The docs and the three sheets are at <code>/design-system</code>. They are on in <code>npm run dev</code> and wherever <code>PUBLIC_DS_GALLERY=1</code> is set (CI, e2e, review builds), and off in production.</li>
			<li><code>?theme=dark</code> or <code>?theme=light</code> forces a theme on a sheet.</li>
			<li>The old frontend keeps port 5173. <code>npm run preview</code> serves a production build on 4175.</li>
		</ul>
	</DocSection>

	<DocSection title="Where components live">
		<p>
			Every component is in <code>src/lib/ds/&lt;category&gt;/</code>, and every folder there is a category in these docs. Each file starts
			with a comment naming the pen frame it implements and anything it does differently on purpose:
			<code>// Design System / Nomi → Overlays / Menu: …</code>
		</p>
		<div class="flex flex-wrap gap-2">
			{#each CATEGORIES as category (category.key)}
				<a href={docHref('/design-system/components', category.key)} class="rounded-sm border border-border px-2.25 py-1 font-mono text-label text-text-2 hover:border-border-strong hover:text-text">{category.key}</a>
			{/each}
		</div>
		<p>
			Components are <strong>presentational only</strong>: no <code>$app/*</code>, no API types, no fetching. Links, active state, data and
			actions come in as props and callbacks. Pages and flows adapt the API’s data to those props, and every backend call from a load or
			action goes through <code>apiFetch</code> in <code>src/lib/server/api.ts</code>.
		</p>
	</DocSection>

	<DocSection title="Using a component" lede="Import it from its file, pass props, and take every word from a message.">
		<Example title="A build that’s running" description="Status names who is working; the stop is secondary." code={useCode}>
			<Status kind="working" agent="koda" label={m.build_working({ name: 'Koda' })} />
			<Button variant="secondary" label={m.build_stop()} onclick={noop} />
		</Example>
		<Example title="A two-way value" description="Values a component changes are $bindable." code={bindCode}>
			<Segmented
				label={m.ws_switch()}
				bind:value={view}
				options={[
					{ value: 'chat', label: m.ws_chat() },
					{ value: 'preview', label: m.ws_preview() },
				]}
			/>
		</Example>
		<h3>Composing</h3>
		<ul>
			<li>When the pen has a part as its own component, so does the code: <code>Segment</code>, <code>Tab</code>, <code>MenuRow</code>, <code>MenuPanel</code>, <code>TooltipBubble</code>, <code>DialogPanel</code>. The interactive wrapper composes the part.</li>
			<li>Slots are snippets. Shared types are exported from <code>&lt;script module&gt;</code>.</li>
			<li>Props are declared with a <code>Props</code> type on <code>$props()</code>; ids come from <code>$props.id()</code>; initial state is copied with a closure. Never name a variable <code>state</code>.</li>
			<li>To show a state the sheet draws open, components take <code>inline</code> (menus, dialogs) or initial state, and <code>data-hover</code>, <code>data-focus</code>, <code>data-active</code> for states without a pointer.</li>
		</ul>
	</DocSection>

	<DocSection title="Tokens and Tailwind" lede="Tailwind knows only the design. Its own palette, fonts, sizes, radii and shadows are switched off in src/app.css.">
		<ul>
			<li>A class is the token’s name without <code>nm-</code>: <code>bg-surface</code>, <code>text-text-2</code>, <code>border-border-strong</code>, <code>text-ui</code>, <code>rounded-sm</code>, <code>shadow-float</code>. All of them are on the <a href={docHref('/design-system/tokens')}>Tokens</a> page.</li>
			<li>No raw colours, no arbitrary values (<code>[…]</code>), no <code>dark:</code>, no shadows other than <code>shadow-float</code> and <code>shadow-focus</code>.</li>
			<li>Write classes in the markup, as class strings or <code>class=&#123;[…]&#125;</code> arrays, not in script variables. <code>src/lib/ds/lint</code> reads the markup of every <code>.svelte</code> file and fails the unit tests on a class Tailwind doesn’t generate.</li>
			<li>Strokes: containers that aren’t interactive use <code>outline -outline-offset-1 outline-border</code> and keep the pen’s padding; buttons, links and inputs keep <code>border</code> and take 1px off the padding on each bordered side.</li>
			<li>A value the pen uses that no token has becomes a named token in <code>src/app.css</code>, with a comment saying where the pen draws it. A value that only belongs to a gallery illustration goes in that file’s scoped <code>&lt;style&gt;</code>.</li>
		</ul>
	</DocSection>

	<DocSection title="Theming">
		<p>
			Every colour token is written with <code>light-dark()</code>, so a class is right in both themes and there is nothing to switch. The
			person’s choice (light, dark or system) is stored in the <code>nm_theme</code> cookie and set on <code>&lt;html data-theme&gt;</code>
			before the first frame. Any element with <code>data-theme</code> re-themes everything inside it, like the light phone preview inside
			the dark workspace. See <a href={docHref('/design-system/foundations/theming')}>Dark &amp; light</a>.
		</p>
	</DocSection>

	<DocSection title="Accessibility">
		<ul>
			<li>Real elements and roles: radiogroup, tablist, menu, dialog, tree, switch, meter, status and alert.</li>
			<li>Full keyboard support, as the platform does it.</li>
			<li>A visible <code>focus-visible</code> ring (2px, offset 2; 3px on buttons) on everything interactive, with hover, active and disabled states.</li>
			<li>A name for every icon-only control; decorative things <code>aria-hidden</code>.</li>
			<li>44px targets on coarse pointers (<code>pointer-coarse:</code>) and <code>motion-reduce</code> respected.</li>
		</ul>
		<p>More on <a href={docHref('/design-system/foundations/accessibility')}>Accessibility</a>.</p>
	</DocSection>

	<DocSection title="Words: Paraglide, English and Indonesian">
		<p>
			Every string a person reads is a message in <code>messages/en.json</code> and <code>messages/id.json</code>, added at the end of both
			files with the Indonesian written at the same time. <code>npm run i18n</code> compiles them into <code>src/lib/paraglide</code> (the
			check and test scripts run it for you).
		</p>
		<CodeBlock code={message} language="ts" tokens={highlight(message, 'ts')} />
		<p>
			<code>src/lib/messages.test.ts</code> fails on a key missing from one language, an empty string or a dropped placeholder. The docs’ own
			copy is documentation and isn’t translated. More on <a href={docHref('/design-system/content/languages')}>Languages</a>.
		</p>
	</DocSection>

	<DocSection title="Tests and gates" lede="Test first, always: the failing test, then the code, then the refactor.">
		<ul>
			<li><code>*.svelte.test.ts</code> run in a real Chromium (Vitest browser mode) with the app’s CSS: components, roles, keyboard, focus, ARIA, events and the pen’s geometry. <code>src/lib/ds/pen.svelte.test.ts</code> collects the geometry.</li>
			<li>Every other <code>*.test.ts</code> runs in Node: pure modules, server code, the relay, messages, tokens, the class lint.</li>
			<li><code>e2e/*.e2e.ts</code> (Playwright) run the production server on 4175 against the stub backend in <code>e2e/stub-api.mjs</code>, including the pen-parity guard.</li>
		</ul>
		<CodeBlock code={gates} language="sh" />
		<p>The gates are chained so a failure stops the chain, and CI runs the same. Never report a gate as passing without having run it.</p>
	</DocSection>

	<DocSection title="Proving 1:1" lede="With the dev server running.">
		<ol>
			<li><strong>Read the frame</strong> through the Pencil tools, never the <code>.pen</code> file: its padding, gap, alignment, stroke, fill, radius and text style, and <code>ctx.bounds</code> for exact positions.</li>
			<li><strong>Write the test first</strong>, asserting the geometry the pen gives you: heights, insets, computed styles.</li>
			<li><strong>Compare</strong> the gallery against the pen’s snapshots with the pen tools.</li>
			<li><strong>Keep CI green</strong>: <code>e2e/pen-parity.e2e.ts</code> asserts every section’s offset and height and every board’s box on all three sheets. If the pen itself changed, re-export <code>pen/bounds.json</code> and <code>pen/export/</code> in the same PR.</li>
			<li><strong>Look at it</strong> in both themes, at 1512 and 390.</li>
		</ol>
		<CodeBlock code={pen} language="sh" />
		<p>One frame’s exact layout, run through the Pencil <code>execute</code> tool (more snippets in <code>pen/README.md</code>):</p>
		<CodeBlock code={frame} language="js" />
		<Note>
			<p>
				The e2e browser and the pen tools run Chromium with <code>--font-render-hinting=none</code>, so Linux measures text as the Mac the
				pen is drawn on. Keep it. The HTML export collapses trailing spaces between inline runs and mis-sizes negative gaps and some fill
				widths: when <code>pen:compare</code> flags something odd, check the frame’s own bounds before changing a component.
			</p>
		</Note>
	</DocSection>

	<DocSection title="How the docs read the code" lede="Component pages are generated, so they can’t fall behind.">
		<ul>
			<li>The <code>virtual:ds-api</code> Vite plugin (<code>src/lib/docs/api</code>) reads every component when the docs are built: the props from the type given to <code>$props()</code>, and the notes from the comments written above it. That is the props table and “Notes from the source”.</li>
			<li>The same plugin follows imports: what a component is built from, what uses it, and which sheet sections show it (“On the pen”).</li>
			<li>What the code can’t say (a summary, when to use it, do’s and don’ts, writing, accessibility, related components) is written in <code>src/lib/docs/entries/&lt;category&gt;.ts</code>.</li>
			<li>Live examples are files in <code>src/lib/docs/examples/&lt;component slug&gt;/&lt;nn&gt;-&lt;name&gt;.svelte</code>, starting with a comment that gives the title and description. The page shows the file’s own code, so what people copy is what they see.</li>
			<li>A new folder in <code>src/lib/ds</code> needs its category added to <code>src/lib/docs/catalog.ts</code>.</li>
		</ul>
		<p>So the comment above <code>$props()</code> is documentation: write it for the person reading the docs.</p>
	</DocSection>

	<DocSection title="Best practices">
		<DoDont
			dos={[
				'Read the pen frame before writing the component; measure, don’t guess.',
				'Write the failing test first.',
				'Take every visible word from a message, in English and Indonesian.',
				'Run pen:report and the whole gate chain before asking for review.',
			]}
			donts={[
				'Fetch data or import $app/* inside src/lib/ds.',
				'Raw colours, arbitrary values, dark: or extra shadows.',
				'Classes built in script variables.',
				'Import from frontend/, or fake what the API doesn’t have (it goes in BACKEND_TODO.md).',
			]}
		/>
	</DocSection>
</DocPage>
