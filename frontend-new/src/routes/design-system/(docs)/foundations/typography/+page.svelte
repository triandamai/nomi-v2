<script lang="ts">
	import Kbd from '$lib/ds/actions/Kbd.svelte';
	import SectionLabel from '$lib/ds/status/SectionLabel.svelte';
	import Tag from '$lib/ds/status/Tag.svelte';
	import AgentByline from '$lib/ds/identity/AgentByline.svelte';
	import DocPage from '$lib/docs/ui/DocPage.svelte';
	import DocSection from '$lib/docs/ui/DocSection.svelte';
	import DoDont from '$lib/docs/ui/DoDont.svelte';
	import Example from '$lib/docs/ui/Example.svelte';
	import Note from '$lib/docs/ui/Note.svelte';
	import SheetEmbed from '$lib/docs/ui/SheetEmbed.svelte';
	import TypeSection from '$lib/gallery/TypeSection.svelte';
	import { docHref } from '$lib/docs/href';

	// Foundations → Typography. The scale and line heights from AGENTS.md §2 and app.css; samples are
	// the Type section's and the gallery's own copy.
	const scale = [
		{ id: 'display', size: '28 / 36 · −0.4', use: 'Page and section titles, the greeting', sample: 'Morning, Mario.' },
		{ id: 'heading', size: '20 / 26', use: 'Large card titles where the pen draws them', sample: 'Build estimate' },
		{ id: 'body', size: '15 / 19', use: 'Message body, page intro', sample: "I couldn't reply because your DeepSeek account is out of credits." },
		{ id: 'ui', size: '13 / 17', use: 'Default UI: buttons, nav, rows, labels', sample: 'Open workspace' },
		{ id: 'caption', size: '12 / 16', use: 'Descriptions, secondary rows', sample: 'Transactions, budgets, subscriptions' },
		{ id: 'label', size: '11 / 14', use: 'Mono meta, kbd, section labels', sample: '402 insufficient_balance · req_9d41e0b2' },
		{ id: 'micro', size: '10 / 13', use: "Only where the pen draws it: a citation's number, a token's name", sample: 'crew-personality-ink' },
		{ id: 'nano', size: '9 / 12', use: 'Only where the pen draws it: initials on a 20px avatar', sample: 'MP' },
	] as const;
	const phone = [
		{ id: 'phone-title', size: '17 / 22', use: 'Sheet titles', sample: 'Move to a project' },
		{ id: 'phone-heading', size: '18 / 23', use: "A plan's summary", sample: 'A daily salah tracker' },
		{ id: 'phone', size: '16 / 21', use: 'Body and inputs, so iOS does not zoom', sample: 'Kirim bukti transfer ke Bu Sri ya' },
		{ id: 'phone-sm', size: '14 / 18', use: 'Second lines', sample: 'Transactions, budgets, subscriptions' },
	] as const;
	const leading = [
		{ id: 'tight', ratio: 1.25, use: 'Where the pen sets it' },
		{ id: 'firm', ratio: 1.3, use: 'Where the pen sets it' },
		{ id: 'snug', ratio: 1.35, use: 'Where the pen sets it' },
		{ id: 'cozy', ratio: 1.4, use: 'Where the pen sets it' },
		{ id: 'comfy', ratio: 1.45, use: 'Where the pen sets it' },
		{ id: 'normal', ratio: 1.5, use: 'Where the pen sets it' },
		{ id: 'relaxed', ratio: 1.6, use: 'Prose: a reply, a page of text' },
		{ id: 'code', ratio: 1.7, use: 'Code blocks' },
	] as const;
	const px = (ratio: number, size: number) => Math.round(ratio * size);
	const labelCode = `<SectionLabel label="What's in this project" />
<Tag kind="no-go" label="No-go" />`;
</script>

<DocPage
	eyebrow="Foundations"
	title="Typography"
	lede="Two families, a short scale and three weights. Geist for everything people read; Geist Mono for what a machine produced. Sentence case everywhere."
	wide
>
	<DocSection title="On the sheet" lede="The Type section of the Nomi sheet: each size with its token, its weight and what it is for.">
		<SheetEmbed sheet="nomi" id="type" label="Type"><TypeSection /></SheetEmbed>
	</DocSection>

	<DocSection title="Two families" lede="Geist for prose and interface; Geist Mono for machine output.">
		<div class="grid gap-4 md:grid-cols-2">
			<div class="flex flex-col gap-3 rounded-md p-5 outline -outline-offset-1 outline-border">
				<span class="font-mono text-label text-text-3">font-sans · Geist</span>
				<span class="text-display font-medium tracking-display text-text">Plain, specific, calm.</span>
				<span class="text-ui text-text-2">Everything a person wrote or reads as prose: messages, titles, buttons, labels, descriptions.</span>
			</div>
			<div class="flex flex-col gap-3 rounded-md p-5 outline -outline-offset-1 outline-border">
				<span class="font-mono text-label text-text-3">font-mono · Geist Mono</span>
				<span class="font-mono text-display text-text">main@a3f9c21</span>
				<span class="text-ui text-text-2">
					Anything a machine produced: paths, IDs, URLs, tokens, money with units, costs, timestamps, durations, model names, versions, keys.
				</span>
			</div>
		</div>
		<p>
			Glyphs Geist Mono doesn't have, such as <Kbd keys="⌘" /> <Kbd keys="⇧" /> <Kbd keys="↵" />, fall back to the system font, as they do in the
			pen. The one other face in the code is Fraunces, and only for the initial on an app's identity tile in a project row: that is the person's
			app, not Nomi's interface.
		</p>
	</DocSection>

	<DocSection title="The scale" lede="Each class sets the size and its line height together. Size and line are in px.">
		<div class="overflow-x-auto rounded-md outline -outline-offset-1 outline-border">
			<table class="w-full min-w-200 border-collapse text-left">
				<thead>
					<tr class="border-b border-border bg-surface font-mono text-label tracking-label text-text-3 uppercase">
						<th scope="col" class="w-40 px-4 py-2.5 font-normal">Class</th>
						<th scope="col" class="w-32 px-4 py-2.5 font-normal">Size / line</th>
						<th scope="col" class="px-4 py-2.5 font-normal">Specimen</th>
						<th scope="col" class="w-64 px-4 py-2.5 font-normal">Use</th>
					</tr>
				</thead>
				<tbody>
					{#each scale as row (row.id)}
						<tr class="border-b border-border align-middle last:border-b-0">
							<th scope="row" class="px-4 py-3 text-left font-normal"><code class="font-mono text-caption text-text">text-{row.id}</code></th>
							<td class="px-4 py-3 font-mono text-label whitespace-nowrap text-text-2">{row.size}</td>
							<td class="px-4 py-3">
								<span
									class={[
										'block text-text',
										row.id === 'display' && 'text-display font-medium tracking-display',
										row.id === 'heading' && 'text-heading font-medium',
										row.id === 'body' && 'text-body',
										row.id === 'ui' && 'text-ui font-medium',
										row.id === 'caption' && 'text-caption',
										row.id === 'label' && 'font-mono text-label',
										row.id === 'micro' && 'font-mono text-micro',
										row.id === 'nano' && 'font-mono text-nano',
									]}>{row.sample}</span
								>
							</td>
							<td class="px-4 py-3 text-caption leading-normal text-text-3">{row.use}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
		<p>
			The line heights are Geist's own height at each size, rounded to a whole pixel, as the pen draws text that sets none. <code>text-display</code>
			also brings its −0.4px tracking (<code>tracking-display</code>). <code>text-micro</code> and <code>text-nano</code> sit below the scale and
			appear only where the pen draws them.
		</p>
	</DocSection>

	<DocSection title="At most four sizes on a screen" lede="A reply already uses four. Past that, hierarchy comes from weight and the three text colours.">
		<p>
			A reply needs the name in the UI size, the body, a caption and the mono meta: four sizes. Anything more is told apart with the weights
			(400, 500, 600) and the text colours (<code>text</code>, <code>text-2</code>, <code>text-3</code>), not a fifth size.
		</p>
		<Example title="Four sizes, three colours" description="A byline, a body, a caption and a mono meta line: everything a reply needs." align="start">
			<div class="flex w-full max-w-140 flex-col gap-3">
				<AgentByline agent="nomi" name="Nomi" meta="14:02 · deepseek-flash" />
				<p class="text-body leading-relaxed text-text-2">Food is October's biggest jump: Rp 2,1 jt against Rp 1,4 jt in September.</p>
				<p class="text-caption text-text-3">Transactions, budgets, subscriptions</p>
			</div>
		</Example>
	</DocSection>

	<DocSection title="Weights" lede="400, 500 and 600. Nothing lighter, nothing bolder.">
		<div class="flex flex-col overflow-hidden rounded-md outline -outline-offset-1 outline-border">
			<div class="flex flex-wrap items-baseline gap-x-6 gap-y-1 border-b border-border px-5 py-4">
				<span class="w-36 font-mono text-label text-text-3">400 · font-normal</span>
				<span class="min-w-0 flex-1 text-body text-text">Message body, descriptions, captions</span>
			</div>
			<div class="flex flex-wrap items-baseline gap-x-6 gap-y-1 border-b border-border px-5 py-4">
				<span class="w-36 font-mono text-label text-text-3">500 · font-medium</span>
				<span class="min-w-0 flex-1 text-body font-medium text-text">Titles, buttons, names, row titles</span>
			</div>
			<div class="flex flex-wrap items-baseline gap-x-6 gap-y-1 px-5 py-4">
				<span class="w-36 font-mono text-label text-text-3">600 · font-semibold</span>
				<span class="min-w-0 flex-1 text-body font-semibold text-text">Only where the pen draws it: phone titles, the sidebar's wordmark</span>
			</div>
		</div>
	</DocSection>

	<DocSection title="Line heights" lede="Where the pen sets an explicit line height, use the leading token. Each rounds to a whole pixel, as the pen does.">
		<div class="overflow-x-auto rounded-md outline -outline-offset-1 outline-border">
			<table class="w-full min-w-200 border-collapse text-left">
				<thead>
					<tr class="border-b border-border bg-surface font-mono text-label tracking-label text-text-3 uppercase">
						<th scope="col" class="w-40 px-4 py-2.5 font-normal">Class</th>
						<th scope="col" class="w-20 px-4 py-2.5 font-normal">Ratio</th>
						<th scope="col" class="w-28 px-4 py-2.5 font-normal">At 13 · 15</th>
						<th scope="col" class="px-4 py-2.5 font-normal">Specimen at 13</th>
						<th scope="col" class="w-56 px-4 py-2.5 font-normal">Use</th>
					</tr>
				</thead>
				<tbody>
					{#each leading as row (row.id)}
						<tr class="border-b border-border align-top last:border-b-0">
							<th scope="row" class="px-4 py-3 text-left font-normal"><code class="font-mono text-caption text-text">leading-{row.id}</code></th>
							<td class="px-4 py-3 font-mono text-label text-text-2">{row.ratio}</td>
							<td class="px-4 py-3 font-mono text-label whitespace-nowrap text-text-2">{px(row.ratio, 13)} · {px(row.ratio, 15)}px</td>
							<td class="px-4 py-3">
								<p
									class={[
										'w-52 text-ui text-text-2',
										row.id === 'tight' && 'leading-tight',
										row.id === 'firm' && 'leading-firm',
										row.id === 'snug' && 'leading-snug',
										row.id === 'cozy' && 'leading-cozy',
										row.id === 'comfy' && 'leading-comfy',
										row.id === 'normal' && 'leading-normal',
										row.id === 'relaxed' && 'leading-relaxed',
										row.id === 'code' && 'leading-code',
									]}
								>
									Plain, specific, sentence case. Name the real thing.
								</p>
							</td>
							<td class="px-4 py-3 text-caption leading-normal text-text-3">{row.use}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
		<p>
			Rounding matters: 15px at 1.35 is 20, not 20.25, or a list of six steps grows by 1.5px and drifts off the pen. Text the pen leaves at
			"normal" takes the size class's own line height and no leading class.
		</p>
	</DocSection>

	<DocSection title="Sentence case, and the one uppercase style" lede="Capitals appear in one place only: the mono section label.">
		<p>
			Write everything in sentence case: titles, buttons, menu items, labels. The section label is the one exception. It is written in sentence
			case in the source and shown in mono capitals by CSS (<code>uppercase</code>, with <code>tracking-label</code>), at
			<code>text-label</code> in <code>text-text-3</code>. The go / no-go tag uses the same style.
		</p>
		<Example title="The section label" description="Written as “What's in this project” and “No-go”; shown in capitals." code={labelCode} align="start">
			<div class="flex flex-col items-start gap-3">
				<SectionLabel label="What's in this project" />
				<SectionLabel label="Recents" count={4} />
				<Tag kind="no-go" label="No-go" />
			</div>
		</Example>
	</DocSection>

	<DocSection title="On phones" lede="Phone screens have their own four sizes, drawn at 390 wide.">
		<div class="overflow-x-auto rounded-md outline -outline-offset-1 outline-border">
			<table class="w-full min-w-170 border-collapse text-left">
				<thead>
					<tr class="border-b border-border bg-surface font-mono text-label tracking-label text-text-3 uppercase">
						<th scope="col" class="w-44 px-4 py-2.5 font-normal">Class</th>
						<th scope="col" class="w-28 px-4 py-2.5 font-normal">Size / line</th>
						<th scope="col" class="px-4 py-2.5 font-normal">Specimen</th>
						<th scope="col" class="w-56 px-4 py-2.5 font-normal">Use</th>
					</tr>
				</thead>
				<tbody>
					{#each phone as row (row.id)}
						<tr class="border-b border-border align-middle last:border-b-0">
							<th scope="row" class="px-4 py-3 text-left font-normal"><code class="font-mono text-caption text-text">text-{row.id}</code></th>
							<td class="px-4 py-3 font-mono text-label whitespace-nowrap text-text-2">{row.size}</td>
							<td class="px-4 py-3">
								<span
									class={[
										'block text-text',
										row.id === 'phone-title' && 'text-phone-title font-semibold',
										row.id === 'phone-heading' && 'text-phone-heading font-semibold',
										row.id === 'phone' && 'text-phone',
										row.id === 'phone-sm' && 'text-phone-sm text-text-2',
									]}>{row.sample}</span
								>
							</td>
							<td class="px-4 py-3 text-caption leading-normal text-text-3">{row.use}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
		<p>The phone body is 16pt so iOS doesn't zoom into a field when someone starts typing in it.</p>
	</DocSection>

	<DocSection title="In code">
		<p>
			Use the classes: <code>text-ui font-medium text-text</code>, <code>font-mono text-label text-text-3</code>. Tailwind's own sizes are switched
			off, so <code>text-sm</code> doesn't exist; only the design's sizes do. Never set a font size or line height by hand: the size class carries
			its line height, and a leading class covers the rest.
		</p>
		<Note>
			<p>
				The words themselves (sentence case, naming the real thing, no "magic") are covered in <a href={docHref('/design-system/content/writing')}>Writing style</a>.
			</p>
		</Note>
	</DocSection>

	<DocSection title="Best practices">
		<DoDont
			dos={[
				'Geist Mono for paths, IDs, URLs, tokens, money, timestamps and model names.',
				'Sentence case for every title, button, label and menu item.',
				'Weight and the three text colours for hierarchy, before another size.',
				'A leading class wherever the pen sets a line height.',
			]}
			donts={[
				'More than four sizes on one screen.',
				'Title Case or ALL CAPS outside the mono section label.',
				'Weights other than 400, 500 and 600.',
				'text-micro or text-nano where the pen does not draw them.',
			]}
		/>
	</DocSection>
</DocPage>
