<script lang="ts">
	import DocPage from '$lib/docs/ui/DocPage.svelte';
	import DocSection from '$lib/docs/ui/DocSection.svelte';
	import DoDont from '$lib/docs/ui/DoDont.svelte';
	import LinkCard from '$lib/docs/ui/LinkCard.svelte';
	import Note from '$lib/docs/ui/Note.svelte';
	import { docHref } from '$lib/docs/href';

	// Get started → For designers. Working in Nomi.pen, handing a design over, what developers
	// translate (AGENTS.md §3 in plain words), reviewing a build, and the known pen slips (§9).
	const sheets = [
		{
			id: 'YOYH1',
			name: 'Design System / Nomi',
			path: '/design-system/nomi',
			holds: 'Foundations and core components: colour, crew colour, type, space, radius and icons, actions, inputs, navigation, identity and status, conversation and build, overlays, layout shells, the rules, and the mascots with the app chrome.',
		},
		{
			id: 'Ljvtr',
			name: 'Design System / Chat responses',
			path: '/design-system/chat',
			holds: 'Everything Nomi can put in a reply: thinking and planning, rich content, actions, response feedback, states and errors, message controls and the composer, long threads, crew behaviour, mobile chat.',
		},
		{
			id: 'DcG0H',
			name: 'Design System / Build',
			path: '/design-system/build',
			holds: 'Everything between describing an app and a live URL: the estimate, building, share and export, first run, the workspace, code, terminal and phone, building together, building on mobile, the story sheet, build types.',
		},
	];

	const translations: [title: string, body: string][] = [
		['Strokes take no room', 'In the pen a 1px stroke sits inside the frame. On the web a border pushes content in. So cards, panels, menus and boards draw their stroke as an inner outline and keep your paddings exactly. Buttons, links and inputs keep a real border (it carries the focus ring) and lose 1px of padding on each bordered side: your 16 becomes 15.'],
		['Layout is read, not eyeballed', 'Gap, padding per side, fill width, fixed width and alignment come straight from the frame. A frame without an alignment is read as top-aligned, not centred, so set it if you mean centre.'],
		['Line heights round like the pen', 'Line heights you leave on “normal” come from the type size. Explicit ones map to the nearest leading token (1.25, 1.3, 1.35, 1.4, 1.45, 1.5, 1.6, 1.7), rounded to a whole pixel as the pen does.'],
		['What’s drawn open is built open', 'Menus, dialogs, tooltips and filled-in cards drawn open on a sheet are rendered open there too. Hover, focus and pressed states drawn side by side are shown without a pointer.'],
		['Your components are their components', 'When a part is its own component in the pen (Segment, Tab, Menu Item, Tooltip, Dialog), it is its own component in code, and the interactive piece is built from it.'],
		['A value with no variable gets a name', 'If a frame uses a size or colour that no variable holds, developers add a named token with a note saying where the pen draws it. Better: add the variable in the pen.'],
		['Text is measured like your Mac', 'CI renders text with font hinting off, so lines wrap where they wrap in the pen.'],
	];

	const slips: [pen: string, code: string][] = [
		['Crew-working banner: the button says “Team and invites” (pasted from another frame).', 'The button reads “Stop all”.'],
		['Move to a project: the search reuses “Search chats, memory, files ⌘K”.', 'The search says “Search projects”.'],
		['Sidebar recents: a long title wraps out of its 30px row.', 'A long title is truncated on one line.'],
		['Apostrophes are straight (\').', 'Messages use typographic apostrophes (’).'],
		['Inline examples in prose are frames 4px apart.', 'Inline prose uses the font’s own spaces.'],
		['A wrapped user bubble is as wide as its longest line.', 'It can be up to 2px wider (CSS sizes a wrapped block to its max width).'],
		['The standard menu’s items are 220 wide inside a 236 panel.', 'Copied as drawn, although it may be a slip.'],
	];
</script>

<DocPage
	eyebrow="Get started"
	title="For designers"
	lede="Nomi.pen is the source of truth for how Nomi looks. What you draw is built 1:1, held there by CI. This is how to work in the file, hand a design over and review what comes back."
	wide
>
	<DocSection title="Nomi.pen">
		<p>
			The whole design lives in one file, <strong>Nomi.pen</strong>, opened in the Pencil app from pen.dev. It holds three design sheets,
			40 flows (each with a light twin) and the app screens. Where the pen is silent about behaviour, timings or copy, the prototype
			<code>marioprasetyo/nomi-prototype</code> is the reference.
		</p>
		<p>
			The file itself isn’t kept in the code repository; the repository keeps snapshots of it in <code>frontend-new/pen/</code>, re-exported
			when the pen changes. Agents read the file only through the Pencil tools and never change it unless asked. Where the file is shared
			with the team isn’t written down in the repository yet.
		</p>
	</DocSection>

	<DocSection title="The three design sheets" lede="Each is rebuilt in these docs from the real components, section by section, and must land on your bounds to the pixel.">
		<div class="grid gap-3 md:grid-cols-3">
			{#each sheets as sheet (sheet.id)}
				<LinkCard href={docHref(sheet.path)} eyebrow={sheet.id} title={sheet.name} blurb={sheet.holds} />
			{/each}
		</div>
		<p>
			Flows are named after their area and step, like <em>Flow / Chat — Share &amp; delete</em>. Every frame in the Build flows uses the
			values on the Build sheet’s <a href={docHref('/design-system/patterns/build/story-sheet')}>Story sheet</a> (one user, one project, one
			clock). Values that aren’t decided stay in brackets in the design, like <code>[hosting domain]</code> and
			<code>[cost per 1K tokens]</code>, so nobody mistakes them for real numbers.
		</p>
	</DocSection>

	<DocSection title="Variables, not values" lede="The pen’s Rules: use nm-* tokens for every colour, size, gap and radius.">
		<ul>
			<li><code>nm-*</code> variables are the interface: surfaces, lines, text, the accent, danger and warning, focus, overlay and shadow, plus sizes, radii and spacing. Each colour resolves per theme, dark and light.</li>
			<li><code>crew-*</code> variables are each crew member’s colour, with <code>-bg</code> and <code>-ink</code>. They belong only on surfaces that are that member’s.</li>
			<li>
				The variables are exported to <code>tokens.json</code> and become the CSS variables and Tailwind classes the code uses, named the same
				without <code>nm-</code>: <code>nm-surface</code> is <code>bg-surface</code>. Every one is on the
				<a href={docHref('/design-system/tokens')}>Tokens</a> page.
			</li>
			<li>Never paint a raw hex in a product frame. A test in the code fails on raw colours, so a hard-coded value can’t be built as drawn.</li>
		</ul>
	</DocSection>

	<DocSection title="Handing over a design">
		<h3>Frames</h3>
		<ul>
			<li>Pick a shell before drawing a screen: <strong>App</strong> (sidebar 248, top bar 52, content padded 32–56, panel 300–380), <strong>Workspace</strong> (project bar 52, agent chat 440, preview), or <strong>Focus</strong> (one column 480–720).</li>
			<li>Desktop frames are 1512 × 862 at 1×. Phone frames are 390 wide: one column, sheets instead of popovers, 44pt targets, 16pt body text.</li>
			<li>Every flow gets a <strong>Light twin</strong>: the same frames in the light theme, so both themes are designed, not guessed.</li>
		</ul>
		<h3>Inside the frame</h3>
		<ul>
			<li>Use auto layout. Gap, padding, fill and alignment are what developers read; a position nudged by hand can’t be.</li>
			<li>Draw open what should be seen open: the menu with its items, the dialog, the filled-in card.</li>
			<li>Draw the empty, loading, error and populated states together. They ship together.</li>
			<li>Write the real copy, in sentence case. It is built word for word and becomes an English and Indonesian message.</li>
			<li>Make a part a component when it repeats; the code mirrors your components one for one.</li>
		</ul>
	</DocSection>

	<DocSection title="What developers translate" lede="The few places where the web and the pen measure differently. You don’t need to draw for them; it helps to know why a value in code looks off by one.">
		<div class="grid gap-x-8 gap-y-6 md:grid-cols-2">
			{#each translations as [title, body] (title)}
				<div class="flex flex-col gap-1.5 border-l border-border pl-4">
					<h3 class="text-ui font-medium text-text">{title}</h3>
					<p class="text-ui leading-normal text-text-2">{body}</p>
				</div>
			{/each}
		</div>
	</DocSection>

	<DocSection title="Reviewing a build">
		<ul>
			<li>Open the <a href={docHref('/design-system/nomi')}>Nomi</a>, <a href={docHref('/design-system/chat')}>Chat responses</a> and <a href={docHref('/design-system/build')}>Build</a> sheets next to the pen. Sections sit where yours do: 64 around the sheet and between sections.</li>
			<li>Look at both themes. Add <code>?theme=dark</code> or <code>?theme=light</code> to a sheet’s address, or use the theme switch at the top of these docs.</li>
			<li>Look at 1512 and at 390.</li>
			<li>Every component page shows the component live; each example switches between the page’s theme, dark and light.</li>
			<li>Positions are already checked: CI fails if a section or board moves a pixel. What you are looking for is everything else: colour, type, copy, states.</li>
			<li>When something differs, name the sheet, the section and the frame, so it can be compared text by text against the pen’s own export.</li>
		</ul>
	</DocSection>

	<DocSection title="Known pen slips" lede="Differences kept on purpose, because the pen looks wrong there. Anything else that differs is a bug.">
		<div class="overflow-x-auto rounded-md outline -outline-offset-1 outline-border">
			<table class="w-full min-w-170 border-collapse text-left">
				<thead>
					<tr class="border-b border-border bg-surface font-mono text-label tracking-label text-text-3 uppercase">
						<th scope="col" class="w-1/2 px-4 py-2.5 font-normal">In the pen</th>
						<th scope="col" class="px-4 py-2.5 font-normal">In the code</th>
					</tr>
				</thead>
				<tbody>
					{#each slips as [pen, code] (pen)}
						<tr class="border-b border-border align-top last:border-b-0">
							<td class="px-4 py-3 text-ui leading-normal text-text-2">{pen}</td>
							<td class="px-4 py-3 text-ui leading-normal text-text">{code}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
		<Note><p>The list lives in AGENTS.md §9. A new difference is added there with its reason, never made silently. Fixing one in the pen lets the code follow the pen again.</p></Note>
	</DocSection>

	<DocSection title="Best practices">
		<DoDont
			dos={[
				'Use nm-* and crew-* variables for every colour, size, gap and radius.',
				'Draw desktop at 1512 and phone at 390, each with a Light twin.',
				'Use auto layout, and set the alignment you mean.',
				'Draw empty, loading, error and populated states together.',
				'Write the real copy, in sentence case.',
			]}
			donts={[
				'Raw hex values in product frames.',
				'Gradients, glass, glows or a second accent hue.',
				'Shadows on flat cards or rows.',
				'Emoji or illustrations in UI chrome.',
				'Progress bars that creep toward 90%.',
			]}
		/>
	</DocSection>
</DocPage>
