<script lang="ts">
	import { ArrowRight, GitBranch, Settings } from '@lucide/svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import IconButton from '$lib/ds/actions/IconButton.svelte';
	import Kbd from '$lib/ds/actions/Kbd.svelte';
	import Icon from '$lib/ds/foundations/Icon.svelte';
	import VisuallyHidden from '$lib/ds/foundations/VisuallyHidden.svelte';
	import Mascot from '$lib/ds/agents/Mascot.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import Tabs from '$lib/ds/inputs/Tabs.svelte';
	import Segmented from '$lib/ds/inputs/Segmented.svelte';
	import Toggle from '$lib/ds/inputs/Toggle.svelte';
	import Status from '$lib/ds/status/Status.svelte';
	import DocPage from '$lib/docs/ui/DocPage.svelte';
	import DocSection from '$lib/docs/ui/DocSection.svelte';
	import DoDont from '$lib/docs/ui/DoDont.svelte';
	import Example from '$lib/docs/ui/Example.svelte';
	import Note from '$lib/docs/ui/Note.svelte';
	import exported from '$lib/ds/tokens/tokens.json';
	import { docHref } from '$lib/docs/href';

	// Foundations → Accessibility. Focus from app.css and the components, keyboard from each
	// component's own handlers (Tabs, Segmented, Menu, Dialog, Tooltip, Toggle, ResizeHandle), targets
	// from pointer-coarse:, contrast computed here from tokens.json so it can't drift from the pen.
	const keyboard: [what: string, role: string, keys: string][] = [
		['Tabs', 'tablist', '← → move between tabs and open them; Home and End jump to the ends. Only the open tab is in the Tab order.'],
		['Segmented', 'radiogroup', 'Arrow keys move the choice, like the platform’s own radio buttons. Only the chosen option is in the Tab order.'],
		['Menu', 'menu', 'Opening it moves focus to the first item. ↑ ↓ move, Home and End jump, Enter or Space picks. Escape closes and returns focus to the button; Tab or a click elsewhere closes.'],
		['Dialog', 'dialog', 'A native modal: focus stays inside until it closes, and Escape closes it.'],
		['Tooltip', 'tooltip', 'Appears on hover or keyboard focus, and Escape hides it. It describes its trigger to screen readers.'],
		['Toggle', 'switch', 'A real button: Space or Enter flips it.'],
		['Resize handle', 'separator', '← → resize the chat by 16px (64 with Shift); Home and End go to its narrowest and widest.'],
	];

	// WCAG relative luminance and contrast, on the tokens' opaque values.
	const tokens = exported.tokens as Record<string, { light?: string; dark?: string }>;
	const luminance = (hex: string) => {
		const [r, g, b] = [1, 3, 5].map((at) => parseInt(hex.slice(at, at + 2), 16) / 255).map((c) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4));
		return 0.2126 * r + 0.7152 * g + 0.0722 * b;
	};
	const ratio = (a: string, b: string) => {
		const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
		return (hi + 0.05) / (lo + 0.05);
	};
	const pairs: [fg: string, bg: string, use: string][] = [
		['text', 'bg', 'Primary text'],
		['text-2', 'bg', 'Secondary text, body copy, icons'],
		['text-3', 'bg', 'Meta, captions, placeholders'],
		['text-3', 'surface', 'Meta on a card'],
		['accent', 'bg', 'The live dot, a chosen option'],
		['accent-ink', 'accent', 'The primary button’s label'],
		['bg', 'text', 'The inverse button’s label'],
		['danger', 'bg', 'Error text'],
		['warning', 'bg', 'Needs attention'],
	];
	const contrast = pairs.map(([fg, bg, use]) => ({
		fg,
		bg,
		use,
		dark: ratio(tokens[`nm-${fg}`].dark!, tokens[`nm-${bg}`].dark!),
		light: ratio(tokens[`nm-${fg}`].light!, tokens[`nm-${bg}`].light!),
	}));

	const hiddenCode = `<script lang="ts">
	import VisuallyHidden from '$lib/ds/foundations/VisuallyHidden.svelte';
<\/script>

<!-- Seen: "Inbox 3". Heard: "Inbox, 3 unread". -->
<span>Inbox <span>3<VisuallyHidden text=" unread" /></span></span>`;
	let tab = $state('transcript');
	let view = $state('all');
</script>

{#snippet verdict(value: number)}
	<span class="flex items-center gap-2 whitespace-nowrap">
		<span class="font-mono text-label text-text">{value.toFixed(1)}:1</span>
		{#if value < 3}
			<span class="font-mono text-label text-danger">under 3</span>
		{:else if value < 4.5}
			<span class="font-mono text-label text-warning">under 4.5</span>
		{/if}
	</span>
{/snippet}

<DocPage
	eyebrow="Foundations"
	title="Accessibility"
	lede="Real elements and roles, every control usable from the keyboard with a focus you can see, targets a finger can hit, names for what has no words, motion that stops when asked, and nothing said by colour alone."
	wide
>
	<DocSection title="Focus you can see" lede="Every interactive element shows focus-visible: a 2px ring in the focus colour, 2px off the element; 3px on buttons.">
		<p>
			The ring is set once for the whole app (<code>:focus-visible</code> in <code>app.css</code>): 2px wide, 2px off the element, in
			<code>focus</code>, a translucent mint. Buttons draw it 3px wide against their own edge. Rows, icon buttons and tabs draw it 1px off
			instead. Fields show focus differently: the border turns accent and the
			<code>shadow-focus</code> glow appears. The ring shows for the keyboard, not for a click.
		</p>
		<Example title="Focus, shown without a keyboard" description="A button, an icon button and a field in their focus states. Press Tab inside the example to see the real thing." align="start">
			<div class="flex w-full flex-wrap items-start gap-6">
				<Button label="Start building" iconEnd={ArrowRight} data-focus />
				<Button variant="secondary" label="Add credits" data-focus />
				<IconButton icon={Settings} label="Settings" data-focus />
				<div class="w-72 max-w-full"><TextField label="Project name" value="budget-kel" helper="Lowercase, numbers and dashes. Becomes your URL." mono data-focus /></div>
			</div>
		</Example>
	</DocSection>

	<DocSection title="Every control from the keyboard" lede="As the platform does it: Tab moves between controls, arrows move inside a group, Escape backs out.">
		<div class="overflow-x-auto rounded-md outline -outline-offset-1 outline-border">
			<table class="w-full min-w-170 border-collapse text-left">
				<thead>
					<tr class="border-b border-border bg-surface font-mono text-label tracking-label text-text-3 uppercase">
						<th scope="col" class="w-36 px-4 py-2.5 font-normal">Component</th>
						<th scope="col" class="w-28 px-4 py-2.5 font-normal">Role</th>
						<th scope="col" class="px-4 py-2.5 font-normal">Keys</th>
					</tr>
				</thead>
				<tbody>
					{#each keyboard as [what, role, keys] (what)}
						<tr class="border-b border-border align-top last:border-b-0">
							<th scope="row" class="px-4 py-3 text-left text-ui font-medium text-text">{what}</th>
							<td class="px-4 py-3"><code class="font-mono text-label text-text-2">{role}</code></td>
							<td class="px-4 py-3 text-ui leading-normal text-text-2">{keys}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
		<Example title="Try it" description="Tab into the tabs, then use the arrow keys; Tab again to the segmented control and the switch." align="start">
			<div class="flex w-full flex-col gap-5">
				<Tabs
					label="Activity"
					bind:value={tab}
					tabs={[
						{ value: 'transcript', label: 'Transcript' },
						{ value: 'events', label: 'Events' },
					]}
				/>
				<span class="flex flex-wrap items-center gap-6">
					<Segmented
						label="Projects"
						bind:value={view}
						options={[
							{ value: 'all', label: 'All', count: 4 },
							{ value: 'live', label: 'Live', count: 2 },
							{ value: 'building', label: 'Building', count: 1 },
						]}
					/>
					<span class="flex items-center gap-2 text-ui text-text-2"><Toggle label="Weekly summary" checked /><span aria-hidden="true">Weekly summary</span></span>
				</span>
			</div>
		</Example>
		<p>
			Real elements first: a button is a <code>&lt;button&gt;</code>, a link an <code>&lt;a&gt;</code>, and the roles the pattern needs where
			there is no element for it (<code>radiogroup</code>, <code>tablist</code>, <code>menu</code>, <code>dialog</code>, <code>tree</code>,
			<code>switch</code>, <code>meter</code>, <code>status</code> and <code>alert</code>). In these docs, <Kbd keys="⌘K" /> jumps to search.
		</p>
	</DocSection>

	<DocSection title="Targets a finger can hit" lede="44px on coarse pointers, without loosening the desktop.">
		<p>
			Controls keep their drawn size under a mouse (a button is 30px tall) and grow to at least 44px where the pointer is coarse, with
			<code>pointer-coarse:min-h-11</code> on a button or <code>pointer-coarse:size-11</code> on an icon button. Every new control does the
			same. Phones are drawn with 44pt targets from the start.
		</p>
	</DocSection>

	<DocSection title="Names for what has no words" lede="Anything that means something without visible words gets a name; anything that repeats the words is hidden.">
		<ul>
			<li><strong>Icon-only controls</strong>: <code>IconButton</code> requires a <code>label</code>, which names the button.</li>
			<li><strong>Icons</strong> are hidden from screen readers unless they carry meaning alone; then give <code>Icon</code> a <code>label</code>.</li>
			<li><strong>Mascots</strong> are hidden next to the member's name; a mascot that stands alone takes a <code>label</code>.</li>
			<li><strong>Extra words</strong> only a screen reader needs (what a count means, an icon-only control's state) go in <code>VisuallyHidden</code>.</li>
		</ul>
		<Example title="Named and hidden" description="A labelled branch icon, an icon button, a mascot with a label, and a count with hidden words." code={hiddenCode} align="start">
			<span class="inline-flex items-center gap-1.5 text-text-2"><Icon icon={GitBranch} size={14} label="Branch" /><span class="font-mono text-label text-text">main</span></span>
			<IconButton icon={Settings} label="Settings" />
			<Mascot agent="nomi" size={32} label="Nomi" />
			<span class="text-ui text-text-2">Inbox <span class="font-mono text-label text-text-3">3<VisuallyHidden text=" unread" /></span></span>
		</Example>
	</DocSection>

	<DocSection title="Motion that stops when asked" lede="Nothing moves for decoration; what does move respects reduced motion.">
		<p>
			Spinners are the only animation in the components, and every one of them stops with <code>motion-reduce:animate-none</code>; the words
			next to it still say what is happening. Statuses never pulse, and skeletons don't shimmer. Anything new that moves stops the same way.
		</p>
	</DocSection>

	<DocSection title="Never colour alone" lede="Colour adds to a word, an icon or a shape. It is never the whole message.">
		<p>
			A status is a dot and a word. A field in error turns its border danger, shows an icon, and says what to do in words. An error callout has
			an icon and a title. A verdict tag says “No-go”.
		</p>
		<Example title="Colour with words" description="Each would still make sense in greyscale." align="start">
			<div class="flex w-full flex-wrap items-start gap-8">
				<span class="flex flex-col gap-2">
					<Status kind="live" label="Live" />
					<Status kind="failed" label="Deploy failed" />
					<Status kind="attention" label="Needs attention" />
				</span>
				<div class="w-72 max-w-full">
					<TextField label="Project name" value="Budget Keluarga!" helper="Lowercase, numbers and dashes. Becomes your URL." error="Use lowercase and dashes, e.g. budget-keluarga." mono />
				</div>
			</div>
		</Example>
	</DocSection>

	<DocSection title="Contrast, in both themes" lede="Computed on this page from the tokens' own values, so it always matches the pen.">
		<div class="overflow-x-auto rounded-md outline -outline-offset-1 outline-border">
			<table class="w-full min-w-170 border-collapse text-left">
				<thead>
					<tr class="border-b border-border bg-surface font-mono text-label tracking-label text-text-3 uppercase">
						<th scope="col" class="w-52 px-4 py-2.5 font-normal">Text on</th>
						<th scope="col" class="px-4 py-2.5 font-normal">Use</th>
						<th scope="col" class="w-40 px-4 py-2.5 font-normal">Dark</th>
						<th scope="col" class="w-40 px-4 py-2.5 font-normal">Light</th>
					</tr>
				</thead>
				<tbody>
					{#each contrast as row (`${row.fg}-${row.bg}`)}
						<tr class="border-b border-border align-middle last:border-b-0">
							<th scope="row" class="px-4 py-2.5 text-left font-normal"><code class="font-mono text-caption text-text">{row.fg} on {row.bg}</code></th>
							<td class="px-4 py-2.5 text-ui text-text-2">{row.use}</td>
							<td class="px-4 py-2.5">{@render verdict(row.dark)}</td>
							<td class="px-4 py-2.5">{@render verdict(row.light)}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
		<p>
			The marks follow WCAG 2's thresholds: 4.5:1 for text, 3:1 for large text. Keep <code>text-3</code> to what it is
			for (meta, captions, placeholders) and never use it for body copy. A token is changed in the pen first; see
			<a href={docHref('/design-system/get-started/contributing')}>Contributing</a>.
		</p>
	</DocSection>

	<DocSection title="Check both themes" lede="Every screen, at 1512 and 390, in dark and light.">
		<p>
			A colour that reads well in one theme can fail in the other. Check focus, errors and quiet text in both: switch the theme in the header,
			or use an example's Page / Dark / Light control. How themes work is in <a href={docHref('/design-system/foundations/theming')}>Dark &amp; light</a>.
		</p>
		<Note>
			<p>Component tests run in real Chromium and check roles, keyboard, focus and ARIA for each component; a new component comes with its own.</p>
		</Note>
	</DocSection>

	<DocSection title="Best practices">
		<DoDont
			dos={[
				'Real elements, and the role the pattern needs.',
				'A visible focus-visible state on every control.',
				'A name on every icon-only control and standalone mascot.',
				'44px targets on coarse pointers.',
			]}
			donts={[
				'Colour as the only sign of a state or an error.',
				'Removing the focus ring, or showing it on click.',
				'Motion that ignores reduced motion.',
				'text-3 for body copy.',
			]}
		/>
	</DocSection>
</DocPage>
