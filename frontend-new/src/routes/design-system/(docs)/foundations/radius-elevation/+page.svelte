<script lang="ts">
	import { Bell, Download, GitBranch, Globe, Pencil, Trash2 } from '@lucide/svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';
	import MenuPanel from '$lib/ds/overlays/MenuPanel.svelte';
	import Toast from '$lib/ds/overlays/Toast.svelte';
	import DocPage from '$lib/docs/ui/DocPage.svelte';
	import DocSection from '$lib/docs/ui/DocSection.svelte';
	import DoDont from '$lib/docs/ui/DoDont.svelte';
	import Example from '$lib/docs/ui/Example.svelte';
	import Note from '$lib/docs/ui/Note.svelte';
	import { docHref } from '$lib/docs/href';

	// Foundations → Radius & elevation. Radii from tokens.json and app.css (with the comments that say
	// where the pen draws each), shadows from app.css, the rules from AGENTS.md §2–3.
	const radii = [
		{ id: '2xs', px: '3', use: "A skeleton's bars." },
		{ id: 'xs', px: '4', use: 'Keyboard keys, tags.' },
		{ id: 'sm', px: '6', use: 'Buttons, chips, rows, inputs. A pen token (nm-radius-sm).' },
		{ id: 'inner', px: '8', use: 'A button or segment inside a 10–12 container on a phone.' },
		{ id: 'md', px: '10', use: 'Panels, cards, the composer, callouts, boards. A pen token (nm-radius-md).' },
		{ id: 'lg', px: '12', use: 'Phones: buttons.' },
		{ id: 'xl', px: '14', use: 'Phones: options.' },
		{ id: '2xl', px: '16', use: 'Phones: toasts.' },
		{ id: '3xl', px: '18', use: 'Phones: cards and bubbles.' },
		{ id: 'device', px: '44', use: "The phone preview's corners." },
		{ id: 'full', px: '∞', use: 'Pills, dots, toggles.' },
	];
	const noop = () => {};
</script>

<DocPage
	eyebrow="Foundations"
	title="Radius & elevation"
	lede="Two radii carry the app: 6 for controls and 10 for containers. Edges come from 1px borders; one shadow, only for what truly floats."
	wide
>
	<DocSection title="Radius" lede="The class is rounded- plus the name. Desktop needs sm and md almost everywhere; the larger ones are for phones.">
		<div class="overflow-x-auto rounded-md outline -outline-offset-1 outline-border">
			<table class="w-full min-w-170 border-collapse text-left">
				<thead>
					<tr class="border-b border-border bg-surface font-mono text-label tracking-label text-text-3 uppercase">
						<th scope="col" class="w-40 px-4 py-2.5 font-normal">Class</th>
						<th scope="col" class="w-16 px-4 py-2.5 font-normal">px</th>
						<th scope="col" class="w-32 px-4 py-2.5 font-normal">Shape</th>
						<th scope="col" class="px-4 py-2.5 font-normal">Where</th>
					</tr>
				</thead>
				<tbody>
					{#each radii as row (row.id)}
						<tr class="border-b border-border align-middle last:border-b-0">
							<th scope="row" class="px-4 py-3 text-left font-normal"><code class="font-mono text-caption text-text">rounded-{row.id}</code></th>
							<td class="px-4 py-3 font-mono text-label text-text-2">{row.px}</td>
							<td class="px-4 py-3">
								<span
									class={[
										'block h-12 w-20 bg-surface outline -outline-offset-1 outline-border-strong',
										row.id === '2xs' && 'rounded-2xs',
										row.id === 'xs' && 'rounded-xs',
										row.id === 'sm' && 'rounded-sm',
										row.id === 'inner' && 'rounded-inner',
										row.id === 'md' && 'rounded-md',
										row.id === 'lg' && 'rounded-lg',
										row.id === 'xl' && 'rounded-xl',
										row.id === '2xl' && 'rounded-2xl',
										row.id === '3xl' && 'rounded-3xl',
										row.id === 'device' && 'rounded-device',
										row.id === 'full' && 'rounded-full',
									]}
								></span>
							</td>
							<td class="px-4 py-3 text-ui leading-normal text-text-2">{row.use}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
		<p>
			Only <code>sm</code> and <code>md</code> are tokens in the pen; the rest are named in <code>app.css</code> for the places the pen draws
			them, each with a comment saying where. Tailwind's own radii are switched off, so a size that isn't here doesn't exist.
		</p>
		<Example title="sm inside md" description="Controls take 6; the panel around them takes 10." align="start">
			<div class="w-80 max-w-full">
				<EmptyState icon={Bell} title="Reminders" count={0} body="Nothing scheduled today. Ask Kala, or add one yourself." action={{ label: 'Add reminder', href: '#reminders' }} />
			</div>
			<div class="w-80 max-w-full">
				<TextField label="Project name" value="budget-keluarga" helper="Lowercase, numbers and dashes. Becomes your URL." mono />
			</div>
		</Example>
	</DocSection>

	<DocSection title="Borders over shadows" lede="Edges come from 1px strokes. A card, a row, a panel or a board is flat.">
		<p>
			Use <code>border</code> for quiet edges and dividers and <code>border-strong</code> for inputs, buttons and emphasis. A container that
			isn't interactive draws its stroke inside, as the pen does: <code>outline -outline-offset-1 outline-border</code>. Raised areas take
			<code>surface</code>, not a shadow.
		</p>
		<p>
			Only things that truly float above the page cast a shadow: menus, toasts, dialogs and the floating ask bar. They take
			<code>shadow-float</code> (0 8px 24px in the <code>shadow</code> colour) with a <code>border-strong</code> edge. Dialogs also sit on the
			<code>overlay</code> scrim.
		</p>
		<Example title="Flat and floating" description="The card sits on the page with a border. The menu and the toast float, so they cast the one shadow." align="start">
			<div class="flex w-full flex-wrap items-start gap-8">
				<div class="w-80 max-w-full">
					<EmptyState icon={Bell} title="Reminders" count={0} body="Nothing scheduled today. Ask Kala, or add one yourself." />
				</div>
				<MenuPanel
					id="radius-menu"
					aria-label="Project actions"
					title="salah-daily-tracker"
					items={[
						{ label: 'Visit live site', icon: Globe },
						{ label: 'Export repo', icon: GitBranch, shortcut: '⌘E' },
						{ label: 'Rename', icon: Pencil, shortcut: 'F2' },
						'divider',
						{ label: 'Delete project', icon: Trash2, danger: true },
					]}
				/>
				<Toast kind="info" icon={Download} title="Repo exported: salah-daily-tracker.zip" detail="23 files · 184 KB" action={{ label: 'Show', onclick: noop }} ondismiss={noop} />
			</div>
		</Example>
	</DocSection>

	<DocSection title="The focus glow is not elevation" lede="shadow-focus is the soft glow around a focused field, as the pen draws it.">
		<p>
			<code>shadow-focus</code> (0 0 8px in the <code>focus</code> colour) goes with a field's mint border while it has focus. It marks the
			field you are typing in; it doesn't lift anything off the page. Only the fields use it: the text field and search.
		</p>
		<Example title="A focused field" description="The border turns accent and the glow appears." align="start">
			<div class="w-80 max-w-full">
				<TextField label="Project name" value="budget-kel" helper="Lowercase, numbers and dashes. Becomes your URL." mono data-focus />
			</div>
		</Example>
		<Note>
			<p>
				A test reads every component's classes and fails on any shadow other than <code>shadow-float</code> and <code>shadow-focus</code>. The
				focus ring itself is an outline, not a shadow: see <a href={docHref('/design-system/foundations/accessibility')}>Accessibility</a>.
			</p>
		</Note>
	</DocSection>

	<DocSection title="Best practices">
		<DoDont
			dos={[
				'rounded-sm on controls, rounded-md on containers.',
				'A 1px stroke for every edge, drawn inside on containers.',
				'shadow-float on menus, toasts, dialogs and the floating ask bar.',
				'rounded-full for pills and dots.',
			]}
			donts={[
				'Shadows on flat cards or rows.',
				'Glows, glass or gradients to lift something.',
				'shadow-focus as decoration or elevation.',
				'Radii that are not in the list.',
			]}
		/>
	</DocSection>
</DocPage>
