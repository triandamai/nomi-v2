<script lang="ts">
	import {
		ArrowUp,
		ArrowUpRight,
		Bell,
		Check,
		CircleAlert,
		Database,
		Download,
		Folder,
		GitBranch,
		GitFork,
		Globe,
		House,
		Inbox,
		MessageSquare,
		Mic,
		Paperclip,
		Plus,
		Rocket,
		RotateCcw,
		Search,
		Settings,
		Users,
		Wallet,
		X,
	} from '@lucide/svelte';
	import Icon from '$lib/ds/foundations/Icon.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import IconButton from '$lib/ds/actions/IconButton.svelte';
	import Link from '$lib/ds/actions/Link.svelte';
	import Chip from '$lib/ds/inputs/Chip.svelte';
	import NavItem from '$lib/ds/navigation/NavItem.svelte';
	import DocPage from '$lib/docs/ui/DocPage.svelte';
	import DocSection from '$lib/docs/ui/DocSection.svelte';
	import DoDont from '$lib/docs/ui/DoDont.svelte';
	import Example from '$lib/docs/ui/Example.svelte';
	import Note from '$lib/docs/ui/Note.svelte';
	import { docHref } from '$lib/docs/href';

	// Foundations → Iconography. The rules from AGENTS.md §2 and Icon.svelte; sizes as the components
	// use them; the set is the sheet's icon grid (Block / Iconography).
	const set = [
		['house', House],
		['message-square', MessageSquare],
		['folder', Folder],
		['wallet', Wallet],
		['bell', Bell],
		['git-fork', GitFork],
		['inbox', Inbox],
		['search', Search],
		['plus', Plus],
		['arrow-up', ArrowUp],
		['mic', Mic],
		['paperclip', Paperclip],
		['rotate-ccw', RotateCcw],
		['download', Download],
		['git-branch', GitBranch],
		['globe', Globe],
		['database', Database],
		['rocket', Rocket],
		['circle-alert', CircleAlert],
		['check', Check],
		['x', X],
		['settings', Settings],
		['users', Users],
		['arrow-up-right', ArrowUpRight],
	] as const;
	const code = `<script lang="ts">
	import { GitBranch, Settings } from '@lucide/svelte';
	import Icon from '$lib/ds/foundations/Icon.svelte';
	import IconButton from '$lib/ds/actions/IconButton.svelte';
<\/script>

<!-- Beside its words: decorative, hidden from screen readers. -->
<span class="inline-flex items-center gap-1.5 text-ui text-text-2">
	<Icon icon={Settings} size={14} /> Settings
</span>

<!-- Carrying meaning on its own: labelled. -->
<span class="inline-flex items-center gap-1.5">
	<Icon icon={GitBranch} size={14} label="Branch" />
	<span class="font-mono text-label text-text">main</span>
</span>

<!-- Icon-only control: the button is named. -->
<IconButton icon={Settings} label="Settings" />`;
</script>

<DocPage
	eyebrow="Foundations"
	title="Iconography"
	lede="Lucide only, at a 1.5 stroke, in three sizes: 16 in navigation and icon buttons, 14 beside text, 12 in chips. An icon is decorative unless it carries meaning on its own."
	wide
>
	<DocSection title="Lucide, at 1.5" lede="One icon family, one stroke, drawn through one component.">
		<p>
			Every icon is a Lucide icon from <code>@lucide/svelte</code>, drawn through <code>foundations/Icon.svelte</code>, which sets the stroke to
			1.5 at every size. Don't bring in another set or draw icons by hand. The one exception is the GitHub mark the pen draws ("Push to
			GitHub"): Lucide dropped its brand icons, so <code>GithubMark</code> keeps Lucide's own drawing of it, shaped like any Lucide icon so
			<code>Icon</code> can render it.
		</p>
		<Example title="The set on the sheet" description="The icons the Nomi sheet shows, at 16 in text-2.">
			<div class="grid grid-cols-4 gap-3 sm:grid-cols-8">
				{#each set as [name, glyph] (name)}
					<span class="flex flex-col items-center gap-1.5">
						<span class="flex size-8 items-center justify-center rounded-sm bg-surface text-text-2"><Icon icon={glyph} size={16} /></span>
						<span class="w-20 truncate text-center font-mono text-label text-text-3">{name}</span>
					</span>
				{/each}
			</div>
		</Example>
	</DocSection>

	<DocSection title="Sizes" lede="Three for the app, a few more only where the pen draws them.">
		<div class="overflow-x-auto rounded-md outline -outline-offset-1 outline-border">
			<table class="w-full min-w-170 border-collapse text-left">
				<thead>
					<tr class="border-b border-border bg-surface font-mono text-label tracking-label text-text-3 uppercase">
						<th scope="col" class="w-20 px-4 py-2.5 font-normal">Size</th>
						<th scope="col" class="w-20 px-4 py-2.5 font-normal">Icon</th>
						<th scope="col" class="px-4 py-2.5 font-normal">Where</th>
					</tr>
				</thead>
				<tbody>
					<tr class="border-b border-border align-middle">
						<th scope="row" class="px-4 py-3 text-left font-mono text-label font-normal text-text">16</th>
						<td class="px-4 py-3 text-text-2"><Icon icon={Folder} size={16} /></td>
						<td class="px-4 py-3 text-ui leading-normal text-text-2">Navigation rows and icon buttons (a 16px glyph in a 30px target).</td>
					</tr>
					<tr class="border-b border-border align-middle">
						<th scope="row" class="px-4 py-3 text-left font-mono text-label font-normal text-text">14</th>
						<td class="px-4 py-3 text-text-2"><Icon icon={Folder} size={14} /></td>
						<td class="px-4 py-3 text-ui leading-normal text-text-2">Inline with text: a button's icon beside its label, a link's arrow, a card or callout header.</td>
					</tr>
					<tr class="border-b border-border align-middle">
						<th scope="row" class="px-4 py-3 text-left font-mono text-label font-normal text-text">12</th>
						<td class="px-4 py-3 text-text-2"><Icon icon={Folder} size={12} /></td>
						<td class="px-4 py-3 text-ui leading-normal text-text-2">Chips, and a chip's chevron.</td>
					</tr>
					<tr class="align-middle">
						<th scope="row" class="px-4 py-3 text-left font-mono text-label font-normal text-text">Others</th>
						<td class="px-4 py-3 text-text-2"><span class="flex items-end gap-2"><Icon icon={Folder} size={13} /><Icon icon={Folder} size={15} /><Icon icon={Folder} size={18} /><Icon icon={Folder} size={22} /></span></td>
						<td class="px-4 py-3 text-ui leading-normal text-text-2">13, 15 and 18, and the phone's 20, 22 and 26: only where the pen draws them. The component accepts nothing else.</td>
					</tr>
				</tbody>
			</table>
		</div>
		<Example title="Each size in place" description="A nav row and an icon button at 16, a button and a link at 14, a chip at 12." align="start">
			<div class="flex w-full flex-wrap items-center gap-6">
				<div class="w-56"><NavItem href="#projects" icon={Folder} label="Projects" count={4} /></div>
				<IconButton icon={Settings} label="Settings" />
				<Button variant="secondary" label="Export repo" iconStart={Download} />
				<Link href="#plan" label="Upgrade to Pro" />
				<Chip kind="provision" label="git repo" icon={GitBranch} />
			</div>
		</Example>
	</DocSection>

	<DocSection title="Colour" lede="Icons take the colour of the text around them.">
		<p>
			Lucide draws with <code>currentColor</code>, so an icon is coloured by its wrapper: usually <code>text-text-2</code>, or
			<code>text-text-3</code> where it is quiet (a chip, an empty state's header). It goes <code>text-text</code> when its row is active,
			<code>text-accent</code> for success, and <code>text-danger</code> for an error.
		</p>
	</DocSection>

	<DocSection title="Decorative or meaningful" lede="Most icons repeat what the words already say. Those are hidden; the few that stand alone get a name.">
		<ul>
			<li>
				<strong>Decorative</strong>: an icon beside its own words (a button's icon, a nav row's icon). <code>Icon</code> without a
				<code>label</code> is hidden from screen readers, so nothing is read twice.
			</li>
			<li>
				<strong>Meaningful</strong>: an icon that says something no words next to it say. Give it a <code>label</code>; it is then announced as
				an image with that name.
			</li>
			<li>
				<strong>Icon-only controls</strong>: use <code>IconButton</code>. Its <code>label</code> is required and names the button, so it never
				goes unnamed.
			</li>
		</ul>
		<Example title="All three" description="A decorative icon, a labelled one, and an icon-only button." {code}>
			<span class="inline-flex items-center gap-1.5 text-ui text-text-2"><Icon icon={Settings} size={14} /> Settings</span>
			<span class="inline-flex items-center gap-1.5 text-text-2">
				<Icon icon={GitBranch} size={14} label="Branch" />
				<span class="font-mono text-label text-text">main</span>
			</span>
			<IconButton icon={Settings} label="Settings" />
		</Example>
	</DocSection>

	<DocSection title="In code">
		<p>
			Import the glyph from <code>@lucide/svelte</code> and pass it to <code>Icon</code> with a size: <code>&lt;Icon icon={'{Folder}'} size={'{14}'} /&gt;</code>.
			Never use a Lucide component directly: it would draw at Lucide's own 2px stroke.
		</p>
		<Note>
			<p>See <a href={docHref('/design-system/components/icon')}>Icon</a> and <a href={docHref('/design-system/components/icon-button')}>IconButton</a> for their props.</p>
		</Note>
	</DocSection>

	<DocSection title="Best practices">
		<DoDont
			dos={[
				'Lucide through Icon.svelte, at 16, 14 or 12.',
				'Colour an icon with its wrapper, usually text-2.',
				'A label on any icon that carries meaning on its own.',
				'IconButton, with its label, for icon-only controls.',
			]}
			donts={[
				'Another icon set, emoji or illustrations in UI chrome.',
				"Lucide's default 2px stroke.",
				'Sizes the pen does not draw.',
				'An icon as the only sign of an error or a state: pair it with words.',
			]}
		/>
	</DocSection>
</DocPage>
