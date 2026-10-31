<script lang="ts">
	import { ArrowRight, Globe, GitBranch, Pencil, Trash2 } from '@lucide/svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import Kbd from '$lib/ds/actions/Kbd.svelte';
	import Status from '$lib/ds/status/Status.svelte';
	import EventRow from '$lib/ds/conversation/EventRow.svelte';
	import KVRow from '$lib/ds/conversation/KVRow.svelte';
	import LiveStep from '$lib/ds/conversation/LiveStep.svelte';
	import MessageAgent from '$lib/ds/conversation/MessageAgent.svelte';
	import ProjectRow from '$lib/ds/lists/ProjectRow.svelte';
	import MenuPanel from '$lib/ds/overlays/MenuPanel.svelte';
	import CrewRow from '$lib/ds/identity/CrewRow.svelte';
	import AgentByline from '$lib/ds/identity/AgentByline.svelte';
	import ResponseActions from '$lib/ds/chat/ResponseActions.svelte';
	import DocPage from '$lib/docs/ui/DocPage.svelte';
	import DocSection from '$lib/docs/ui/DocSection.svelte';
	import Example from '$lib/docs/ui/Example.svelte';
	import Note from '$lib/docs/ui/Note.svelte';
	import SheetEmbed from '$lib/docs/ui/SheetEmbed.svelte';
	import RulesSection from '$lib/gallery/RulesSection.svelte';
	import { PRINCIPLES } from '$lib/docs/principles';
	import { slugify } from '$lib/docs/slug';
	import { docHref } from '$lib/docs/href';

	// Foundations → Principles. The five from the Nomi sheet's header (principles.ts), what each
	// means in practice from AGENTS.md §2, and the sheet's own Rules section. Copy in the examples is
	// the gallery's.
	const noop = () => {};
</script>

<DocPage
	eyebrow="Foundations"
	title="Principles"
	lede="An instrument, not a toy. Calm, dense, precise: a well-made tool for people's money, plans and projects, not a playful chat toy. Five principles follow from that, and every screen in Nomi keeps them."
	wide
>
	<DocSection title="The five" lede="Word for word as they open the Nomi sheet. Each one is explained below.">
		<ol class="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
			{#each PRINCIPLES as [title, body], index (title)}
				<li class="flex flex-col gap-1.5 rounded-md px-4 pt-3.5 pb-4 outline -outline-offset-1 outline-border">
					<span class="font-mono text-label text-text-3">0{index + 1}</span>
					<a href="#{slugify(title)}" class="text-ui font-medium text-text hover:underline hover:decoration-border-strong hover:underline-offset-4">{title}</a>
					<span class="text-caption leading-normal text-text-3">{body}</span>
				</li>
			{/each}
		</ol>
		<p>
			Neutral surfaces do the hierarchy; colour is rare and always means something. The pen file, <code>Nomi.pen</code>, is the source of truth
			for how everything looks; these principles are why it looks that way.
		</p>
	</DocSection>

	<DocSection title={PRINCIPLES[0][0]} lede={PRINCIPLES[0][1]}>
		<p>
			Mint (<code>accent</code>) marks the single thing to act on: the primary button, the live dot, the chosen option. Everything else is
			neutral. Keep it under a tenth of what a view shows.
		</p>
		<ul>
			<li>One primary button per view.</li>
			<li>Inside a panel, the main action is <strong>inverse</strong> (a text-coloured fill), so mint stays rare even on a busy screen.</li>
			<li>The live dot and the chosen option take mint too. Nothing decorative does.</li>
		</ul>
		<Example title="The page's action, and a panel's" description="One primary for the view; inside the panel the main action is inverse and the rest are secondary." align="start">
			<div class="flex w-full flex-col gap-4">
				<div class="flex flex-wrap items-center gap-3">
					<span class="min-w-0 flex-1 text-heading font-medium text-text">Projects</span>
					<Button label="Start building" iconEnd={ArrowRight} />
				</div>
				<div class="flex max-w-120 flex-col gap-3 rounded-md bg-surface p-4 outline -outline-offset-1 outline-border">
					<span class="flex items-center gap-3">
						<span class="min-w-0 flex-1 font-mono text-label text-text">salah-daily-tracker</span>
						<Status kind="live" label="Live" />
					</span>
					<span class="flex flex-wrap gap-2">
						<Button variant="inverse" label="Save changes" />
						<Button variant="secondary" label="Cancel" />
					</span>
				</div>
			</div>
		</Example>
		<p>More on the neutrals and the accent in <a href={docHref('/design-system/foundations/colour')}>Colour</a>.</p>
	</DocSection>

	<DocSection title={PRINCIPLES[1][0]} lede={PRINCIPLES[1][1]}>
		<p>
			Anything a machine produced is set in Geist Mono: paths, IDs, URLs, tokens, money amounts with units, costs, timestamps, durations, model
			names, versions and keyboard keys. Everything a person wrote, or reads as prose, is Geist.
		</p>
		<Example title="Machine output in mono" description="Who answered and when, an event, a value, a key and a reply's cost: all mono. The answer itself is prose." align="start">
			<div class="flex w-full max-w-160 flex-col gap-3">
				<MessageAgent agent="nomi" name="Nomi" meta="14:02 · deepseek-flash" text="Food is October's biggest jump: Rp 2,1 jt against Rp 1,4 jt in September." />
				<ResponseActions text="Food is October's biggest jump." meta="deepseek-flash · 1.4s · 812 tokens" onlike={noop} ondislike={noop} />
				<EventRow kind="deploy" text="deploy #14 · main@a3f9c21 · built in 41s" time="15:38" />
				<KVRow icon={GitBranch} label="Code" value="23 files · main@a3f9c21" />
				<span class="flex items-center gap-2 text-ui text-text-2">Search everything <Kbd keys="⌘K" /></span>
			</div>
		</Example>
		<p>The sizes and the one uppercase style are in <a href={docHref('/design-system/foundations/typography')}>Typography</a>.</p>
	</DocSection>

	<DocSection title={PRINCIPLES[2][0]} lede={PRINCIPLES[2][1]}>
		<p>
			Edges come from 1px strokes, not from shadows. A card, a row, a panel or a board is flat with a border. Only what truly floats above the
			page casts the one shadow: menus, toasts, dialogs and the floating ask bar.
		</p>
		<Example title="A bordered card and a floating menu" description="The row sits on the page, so it has a border. The menu floats over it, so it has the shadow." align="start">
			<div class="flex w-full flex-wrap items-start gap-8">
				<div class="flex min-w-0 flex-1 basis-80 flex-col gap-2">
					<ProjectRow
						href="#salah"
						name="salah-daily-tracker"
						description="Daily salah tracker with countdown and streaks"
						tone="nomi"
						status={{ kind: 'live', label: 'Live' }}
						url="salah-daily-tracker-0ab8f8.trian.space"
						updated="2d"
					/>
					<span class="font-mono text-label text-text-3">flat · border</span>
				</div>
				<div class="flex flex-col gap-2">
					<MenuPanel
						id="principles-menu"
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
					<span class="font-mono text-label text-text-3">floating · shadow-float</span>
				</div>
			</div>
		</Example>
		<p>The radii and the one shadow are in <a href={docHref('/design-system/foundations/radius-elevation')}>Radius &amp; elevation</a>.</p>
	</DocSection>

	<DocSection title={PRINCIPLES[3][0]} lede={PRINCIPLES[3][1]}>
		<p>
			Say what is really happening: the real tool and its target, the time so far, and the current step when nobody knows how long it takes. No
			progress bars that creep toward 90%, no spinners that pretend. What something costs stays in view. Empty, loading, error and populated
			states ship together.
		</p>
		<Example title="Progress in words" description="The step an agent is on, and how long it has taken. The button says what it is doing." align="start">
			<div class="flex w-full max-w-160 flex-col items-start gap-4">
				<LiveStep step="Writing src/routes/menu/+page.svelte" elapsed="01:12" stopLabel="Stop" onstop={noop} />
				<Button label="Deploying · 00:14" variant="inverse" loading />
			</div>
		</Example>
		<p>Loading, errors and empty lists are in <a href={docHref('/design-system/foundations/states')}>Honest states</a>.</p>
	</DocSection>

	<DocSection title={PRINCIPLES[4][0]} lede={PRINCIPLES[4][1]}>
		<p>
			Each of the ten crew members has one mascot: one colour, one silhouette, flat. It appears where that member speaks or works (their
			byline, their row, the status that says they are working) and nowhere else. It is identity, never decoration.
		</p>
		<Example title="Where a mascot belongs" description="Next to the member's name when they speak, in their row, and in place of the dot while they work." align="start">
			<div class="flex w-full max-w-120 flex-col gap-4">
				<AgentByline agent="koda" name="Koda" meta="15:37 · 4m 12s" />
				<CrewRow agent="dana" name="Dana" role="Transactions, budgets, subscriptions" state="idle" />
				<Status kind="working" label="Koda is building" agent="koda" />
			</div>
		</Example>
		<p>All ten, their colours and sizes are in <a href={docHref('/design-system/foundations/crew')}>Crew &amp; mascots</a>.</p>
	</DocSection>

	<DocSection title="Rules" lede="The checkable version, as the Nomi sheet ends. If a frame breaks one, fix it before showing it.">
		<SheetEmbed sheet="nomi" id="rules" label="Rules"><RulesSection /></SheetEmbed>
		<Note>
			<p>
				The pen wins. Where <code>Nomi.pen</code> itself breaks one of these rules (a 10px citation number, a 13px icon), the code follows the
				pen and says so in the component's comment.
			</p>
		</Note>
	</DocSection>
</DocPage>
