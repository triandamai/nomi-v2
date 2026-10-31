<script lang="ts">
	import DocPage from '$lib/docs/ui/DocPage.svelte';
	import DocSection from '$lib/docs/ui/DocSection.svelte';
	import DoDont from '$lib/docs/ui/DoDont.svelte';
	import Guidance from '$lib/docs/ui/Guidance.svelte';
	import Note from '$lib/docs/ui/Note.svelte';
	import SheetEmbed from '$lib/docs/ui/SheetEmbed.svelte';
	import SpaceSection from '$lib/gallery/SpaceSection.svelte';
	import ShellsSection from '$lib/gallery/ShellsSection.svelte';
	import { docHref } from '$lib/docs/href';

	// Foundations → Space & layout. The scale from tokens.json and AGENTS.md §2, uses from the sheet's
	// Spacing block, the shells from src/lib/ds/shells and build/split.ts.
	const scale = [
		{ step: '2xs', px: 2, tw: '0.5', use: 'Nav item stack' },
		{ step: 'xs', px: 4, tw: '1', use: 'Label to value' },
		{ step: 'sm', px: 6, tw: '1.5', use: 'Icon to label' },
		{ step: 'md', px: 8, tw: '2', use: 'Chip groups' },
		{ step: 'lg', px: 12, tw: '3', use: 'Row content' },
		{ step: 'xl', px: 16, tw: '4', use: 'Panel padding' },
		{ step: '2xl', px: 20, tw: '5', use: 'Panel side pad' },
		{ step: '3xl', px: 24, tw: '6', use: 'Section gap' },
		{ step: '4xl', px: 32, tw: '8', use: 'Page padding' },
		{ step: '5xl', px: 48, tw: '12', use: 'Page header gap' },
	];
	const extras = [
		{ px: 9, tw: '2.25' },
		{ px: 10, tw: '2.5' },
		{ px: 14, tw: '3.5' },
		{ px: 28, tw: '7' },
	];
	const shells = [
		{
			name: 'App shell',
			slug: 'app-shell',
			for: 'Home, Chats, Projects, Money…',
			parts: ['Sidebar 248', 'Top bar 52', 'Content padded 32–56', 'Crew panel 300–380'],
			narrow: 'Below 1024px the sidebar moves into a drawer; the panel shows only from 1280px up.',
		},
		{
			name: 'Workspace shell',
			slug: 'workspace-shell',
			for: 'Inside one project',
			parts: ['Project bar 52', 'Agent chat 440, resizable 360–720', 'Preview, code or terminal: never under 480'],
			narrow: 'On a phone, Chat and Preview switch at the top instead of sitting side by side. A double-click on the split resets the chat to 520.',
		},
		{
			name: 'Focus shell',
			slug: 'focus-shell',
			for: 'Onboarding, dialogs, checkout',
			parts: ['One centred column, 480 or 720'],
			narrow: 'The column fills the screen, 16 from each edge.',
		},
	];
</script>

<DocPage
	eyebrow="Foundations"
	title="Space & layout"
	lede="A 4px grid with 2 and 6 for dense UI, the pen's few paddings of its own, two screen sizes and three shells. Gaps and padding come only from the scale."
	wide
>
	<DocSection title="On the sheet" lede="The Space, radius & icons section of the Nomi sheet. The bars are three times the step.">
		<SheetEmbed sheet="nomi" id="space-radius-icons" label="Space, radius & icons"><SpaceSection /></SheetEmbed>
	</DocSection>

	<DocSection title="The spacing scale" lede="Ten steps, each a token and a Tailwind step. The bars here are drawn at actual size.">
		<div class="overflow-x-auto rounded-md outline -outline-offset-1 outline-border">
			<table class="w-full min-w-170 border-collapse text-left">
				<thead>
					<tr class="border-b border-border bg-surface font-mono text-label tracking-label text-text-3 uppercase">
						<th scope="col" class="w-44 px-4 py-2.5 font-normal">Token</th>
						<th scope="col" class="w-20 px-4 py-2.5 font-normal">px</th>
						<th scope="col" class="w-28 px-4 py-2.5 font-normal">Tailwind</th>
						<th scope="col" class="w-24 px-4 py-2.5 font-normal">Size</th>
						<th scope="col" class="px-4 py-2.5 font-normal">On the sheet</th>
					</tr>
				</thead>
				<tbody>
					{#each scale as row (row.step)}
						<tr class="border-b border-border align-middle last:border-b-0">
							<th scope="row" class="px-4 py-2.5 text-left font-normal"><code class="font-mono text-caption text-text">nm-space-{row.step}</code></th>
							<td class="px-4 py-2.5 font-mono text-label text-text-2">{row.px}</td>
							<td class="px-4 py-2.5 font-mono text-label text-text-2">gap-{row.tw}</td>
							<td class="px-4 py-2.5"><span class="block h-4 rounded-2xs bg-accent" style:width="var(--nm-space-{row.step})"></span></td>
							<td class="px-4 py-2.5 text-caption text-text-3">{row.use}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
		<p>
			The Tailwind step is the same for every spacing utility: <code>p-4</code>, <code>gap-4</code>, <code>mt-4</code> are all 16px. Each
			quarter step is 1px, so <code>2.25</code> is 9px and <code>3.75</code> is 15px.
		</p>

		<h3>The pen's own paddings</h3>
		<p>
			A few paddings the pen draws aren't on the scale. Use them only where the pen draws them, never as a new step to reach for:
		</p>
		<div class="flex flex-wrap gap-3">
			{#each extras as extra (extra.px)}
				<span class="flex items-center gap-2 rounded-sm px-3 py-2 outline -outline-offset-1 outline-border">
					<span class="font-mono text-label text-text">{extra.px}px</span>
					<span class="font-mono text-label text-text-3">→ {extra.tw}</span>
				</span>
			{/each}
		</div>

		<h3>Strokes take no room</h3>
		<p>
			In the pen a 1px stroke sits inside the frame and doesn't move what's in it; a CSS <code>border</code> does. So a card, panel, menu or
			list draws its stroke as <code>outline -outline-offset-1 outline-border</code> and keeps the pen's padding exactly. Buttons, links and
			inputs keep <code>border</code> (their outline is the focus ring) and take 1px off each bordered side: 16 → <code>3.75</code>, 14 →
			<code>3.25</code>, 12 → <code>2.75</code>, 10 → <code>2.25</code>.
		</p>
	</DocSection>

	<DocSection title="Two screen sizes" lede="Every screen is designed, built and checked at both, in dark and light.">
		<div class="grid gap-4 md:grid-cols-2">
			<div class="flex flex-col gap-2 rounded-md p-5 outline -outline-offset-1 outline-border">
				<span class="font-mono text-label text-text-3">Desktop</span>
				<span class="font-mono text-heading text-text">1512 × 862</span>
				<span class="text-ui leading-normal text-text-2">At 1×. Columns are fixed; content fills what's left.</span>
			</div>
			<div class="flex flex-col gap-2 rounded-md p-5 outline -outline-offset-1 outline-border">
				<span class="font-mono text-label text-text-3">Phone</span>
				<span class="font-mono text-heading text-text">390 wide</span>
				<span class="text-ui leading-normal text-text-2">One column, drawn in points, with the phone sizes from Typography.</span>
			</div>
		</div>
	</DocSection>

	<DocSection title="Shells" lede="Every screen sits in one of three frames. Pick the shell before designing the screen.">
		<SheetEmbed sheet="nomi" id="layout-shells" label="Layout shells"><ShellsSection /></SheetEmbed>
		<div class="grid gap-4 lg:grid-cols-3">
			{#each shells as shell (shell.slug)}
				<div class="flex flex-col gap-3 rounded-md px-4 pt-3.5 pb-4 outline -outline-offset-1 outline-border">
					<span class="flex flex-col gap-0.5">
						<a href={docHref(`/design-system/components/${shell.slug}`)} class="text-ui font-medium text-text hover:underline hover:decoration-border-strong hover:underline-offset-4">{shell.name}</a>
						<span class="text-caption text-text-3">{shell.for}</span>
					</span>
					<ul class="flex flex-col gap-1.5">
						{#each shell.parts as part (part)}<li class="font-mono text-label text-text-2">{part}</li>{/each}
					</ul>
					<span class="text-caption leading-normal text-text-3">{shell.narrow}</span>
				</div>
			{/each}
		</div>
	</DocSection>

	<DocSection title="Phones" lede="One column, sheets, big targets.">
		<Guidance
			use={[
				'One column.',
				'Sheets instead of popovers.',
				'44pt targets on every control (pointer-coarse:).',
				'A 16pt body, so fields never zoom.',
				'The composer above the keyboard; banners at the top.',
			]}
			avoid={['Side-by-side panels: switch between them instead, as the workspace does with Chat and Preview.', 'Menus that float off a button: use a sheet.']}
		/>
		<Note>
			<p>
				Controls get their 44px target with <code>pointer-coarse:</code> (<code>pointer-coarse:min-h-11</code> on a button), so the desktop
				layout keeps its density and a finger still has room. More in <a href={docHref('/design-system/foundations/accessibility')}>Accessibility</a>.
			</p>
		</Note>
	</DocSection>

	<DocSection title="Best practices">
		<DoDont
			dos={[
				'Gaps and padding from the scale, read from the pen frame.',
				"The pen's 9, 10, 14 and 28 only where it draws them.",
				'Outline strokes on containers, so padding stays the pen’s.',
				'Check every screen at 1512 and 390, dark and light.',
			]}
			donts={[
				'Arbitrary values like p-[13px]: a test fails on them.',
				'Eyeballed spacing. Measure the frame.',
				'A preview narrower than 480 in the workspace.',
				'Popovers on phones.',
			]}
		/>
	</DocSection>
</DocPage>
