<script lang="ts">
	import AgentByline from '$lib/ds/identity/AgentByline.svelte';
	import DataTable from '$lib/ds/chat/DataTable.svelte';
	import DocPage from '$lib/docs/ui/DocPage.svelte';
	import DocSection from '$lib/docs/ui/DocSection.svelte';
	import DoDont from '$lib/docs/ui/DoDont.svelte';
	import Example from '$lib/docs/ui/Example.svelte';
	import Note from '$lib/docs/ui/Note.svelte';
	import { docHref } from '$lib/docs/href';

	// Content → Numbers, time & money. AGENTS.md §2 (mono for machine output, honest states) and
	// every number as the pen writes it (pen/export). Where the pen isn't consistent, this says so.
	type Row = [form: string, example: string, where: string];

	const money: Row[] = [
		['A full amount', 'Rp 2.200.000', '“Rp”, a space, a dot between thousands. Carts, emails, prices.'],
		['In a table', '1.173.000', 'Bare numbers, right-aligned; the currency once, in the caption: “Rp · from Dana · 142 transactions”.'],
		['Millions, short', 'Rp 2,1 jt', 'Juta, with a decimal comma. Summaries, prose, chat lists.'],
		['Thousands, short', 'Rp 500rb', 'Ribu, no space. Suggestions, options, quick figures.'],
		['Per period', 'Rp 285.990 / month', 'The period after a slash with spaces.'],
		['Hidden when shared', 'Rp •••', 'Amounts and names are masked by default when a reply or chat is shared.'],
		['A change', '+12% · −6%', 'Always signed; a true minus sign (−), not a hyphen.'],
	];

	const times: Row[] = [
		['Clock time', '14:02 · 08:13', '24-hour, with a leading zero.'],
		['With seconds', '08:41:02', 'Only in a terminal log.'],
		['A range', '07:00–11:00', 'En dash, no spaces.'],
		['A date', '25 Nov 2026', 'Day, short month, year.'],
		['Date and time', '25 Nov · 09:00 · 2 Nov, 09:00', 'Both forms appear in the pen.'],
		['A span of days', '1–7 Oct · 1–2 Nov · Sat 1 – Sun 2 Nov', 'En dash; spaced when each side has a weekday.'],
		['Relative', 'now · 2h · 2d · YESTERDAY', 'Lists use short relative times; the date divider is a mono label.'],
		['Ahead', 'expires in 7 days · resets 1 Nov', 'Words for how long; a date for when.'],
	];

	const durations: Row[] = [
		['Running clock', '00:14 · 01:12', 'Minutes and seconds, while something runs: “Deploying · 00:14”, “Mario · running · 00:21”.'],
		['Short clock', '0:12 · 0:24', 'The pen also writes it without the leading zero: “Publishing… 0:12”, “Retrying in 0:24”.'],
		['Finished', '5m 40s · 41s · 8s · 1.4s', '“Built in 5m 40s”, “built in 41s”, “Thought for 8s”, a reply’s “1.4s”.'],
		['An estimate', 'About 5 min · ~3 min left · 4–7 min', 'Before or during a run. Never a percentage.'],
	];

	const ids: Row[] = [
		['A deploy or a run', 'deploy #2 · run #3', 'Numbered with #.'],
		['A commit', 'main@4be1c07', 'Branch @ short hash.'],
		['A request', 'req_9d41e0b2', 'The provider’s own ID, quoted next to its error.'],
		['A key or card', '••a91f · Visa •• 4412', 'Only the last characters, after bullets.'],
		['A model', 'deepseek-flash · claude-sonnet', 'As the provider names it, in mono. In a sentence or a button, its product name: “Try Claude Sonnet”.'],
		['Tokens', '812 tokens · 21.8K / 1M tokens this month', 'K and M, with a decimal point.'],
		['A file size', '212 KB · PDF · 4,2 MB · max 20 MB', 'Nomi’s own lines write a decimal comma; quoted provider text keeps its own (“48.2 MB”).'],
		['A path or address', 'src/routes/menu/+page.svelte · warung-kopi-menu-3f9a12.trian.space', 'As the machine has it; addresses without https://.'],
		['A version', 'v0.0.75', 'Lower-case v.'],
		['Keys', '⌘K · ⇧↵ · esc · F2', 'Symbols for the modifiers and return; esc in lower case.'],
	];

	const tables: { title: string; lede: string; rows: Row[] }[] = [
		{ title: 'Money', lede: 'Nomi’s money is Rupiah, written the Indonesian way in both languages.', rows: money },
		{ title: 'Time and dates', lede: 'Month names are English and short; the pen draws no Indonesian dates.', rows: times },
		{ title: 'Durations', lede: 'The real clock, never a progress bar.', rows: durations },
		{ title: 'IDs, models and machine values', lede: 'Always in mono, exactly as the machine produced them.', rows: ids },
	];

	const weeks = [
		['1–7 Oct', '312.000', '168.000', '480.000', { value: '+12%', tone: 'bad' as const }],
		['8–14 Oct', '286.000', '241.000', '527.000', { value: '+31%', tone: 'bad' as const }],
		['15–21 Oct', '301.000', '322.000', '623.000', { value: '+48%', tone: 'bad' as const }],
		['22–28 Oct', '274.000', '196.000', '470.000', { value: '−6%', tone: 'good' as const }],
	];
</script>

<DocPage
	eyebrow="Content"
	title="Numbers, time & money"
	lede="Costs, durations, timestamps, IDs and the mono that shows them, as the pen writes them. Nothing here is rounded, guessed or prettier than the real value."
	wide
>
	<DocSection title="Mono for machine output">
		<p>
			Anything a machine produced is set in Geist Mono: paths, IDs, URLs, tokens, money amounts with units, costs, timestamps, durations,
			model names, versions and keyboard keys. Everything a person wrote, or reads as prose, is Geist.
		</p>
		<p>
			In the pen, a value that stands on its own is mono: a table cell, a meta line, the time in a byline. Inside a sentence or a button
			label it takes the sentence’s font: “Food is October’s biggest jump: Rp 2,1 jt against Rp 1,4 jt in September.”, “Deploying · 00:14”.
		</p>
		<Example title="Values standing on their own" description="A byline’s time and duration, a reply’s meta line, a table of amounts." align="start">
			<div class="flex w-full max-w-160 flex-col gap-4">
				<AgentByline agent="koda" name="Koda" meta="15:37 · 4m 12s" />
				<span class="font-mono text-label text-text-3">deepseek-flash · 1.4s · 812 tokens</span>
				<DataTable
					title="Food spending by week"
					columns={['Week', 'Groceries', 'GoFood', 'Total', 'vs Sep']}
					rows={weeks}
					total={['October', '1.173.000', '927.000', '2.100.000', { value: '+20%', tone: 'bad' }]}
					source="Rp · from Dana · 142 transactions"
				/>
			</div>
		</Example>
	</DocSection>

	{#each tables as table (table.title)}
		<DocSection title={table.title} lede={table.lede}>
			<div class="overflow-x-auto rounded-md outline -outline-offset-1 outline-border">
				<table class="w-full min-w-170 border-collapse text-left">
					<thead>
						<tr class="border-b border-border bg-surface font-mono text-label tracking-label text-text-3 uppercase">
							<th scope="col" class="w-40 px-4 py-2.5 font-normal">Form</th>
							<th scope="col" class="w-80 px-4 py-2.5 font-normal">As the pen writes it</th>
							<th scope="col" class="px-4 py-2.5 font-normal">How</th>
						</tr>
					</thead>
					<tbody>
						{#each table.rows as [form, example, where] (form)}
							<tr class="border-b border-border align-top last:border-b-0">
								<th scope="row" class="px-4 py-3 text-left text-ui font-normal text-text-2">{form}</th>
								<td class="px-4 py-3 font-mono text-caption leading-normal break-words text-text">{example}</td>
								<td class="px-4 py-3 text-ui leading-normal text-text-2">{where}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</DocSection>
	{/each}

	<DocSection title="Counts and progress">
		<ul>
			<li>Progress is a count in words: “5 of 9 written”, “Step 2 of 6 · menu page”, “2 of 4 done”, “Question 1 of 4”.</li>
			<li>Selections say how many: “3 selected · Rp 285.990 / month”.</li>
			<li>A limit shows both sides: “12.480 / 10.000” characters, “21.8K / 1M tokens this month”.</li>
			<li>One and many are two messages: “1 new reply”, “2 new replies”. See <a href={docHref('/design-system/content/languages')}>Languages</a>.</li>
		</ul>
	</DocSection>

	<DocSection title="Cost">
		<ul>
			<li>Cost is always visible and never the headline: a quiet line under the choice. “About 5 min · included in your free plan”.</li>
			<li>After a build, the actual figures replace the estimate: “Built in 5m 40s · used your free allowance”.</li>
			<li>Cost is never guessed: providers don’t report a charge on failed calls, so a failure never shows a guessed charge.</li>
		</ul>
		<Note tone="warning">
			<p>
				Not decided yet. The Build sheet’s Story sheet keeps these values in brackets: the cost per 1K tokens, the Pro price in IDR, the Pro
				allowance, the Free plan’s live-project cap and builds per month, the hosting domain, the payment provider and the e-wallet list. An estimate in rupiah is written as
				<code>Rp (38–62) × [cost per 1K tokens]</code> until then. Don’t fill them in.
			</p>
		</Note>
	</DocSection>

	<DocSection title="Where the pen isn’t consistent" lede="Copy the frame you’re building. These are open, not rules.">
		<ul>
			<li>The running clock is drawn both as <code>00:14</code> and as <code>0:12</code>.</li>
			<li>Decimals outside money: token counts and seconds use a point (<code>21.8K</code>, <code>1.4s</code>); file sizes in Nomi’s own lines use a comma (<code>4,2 MB</code>).</li>
			<li>Date and time are joined with a middle dot (<code>25 Nov · 09:00</code>) in one place and a comma (<code>2 Nov, 09:00</code>) in another.</li>
			<li>No other currency is drawn, and the code has no shared formatter for money, dates or durations yet: components take these values as ready-written strings.</li>
		</ul>
	</DocSection>

	<DocSection title="Best practices">
		<DoDont
			dos={[
				'Mono for every value a machine produced.',
				'Rp with a space and dots for thousands: Rp 2.200.000.',
				'A true minus sign and en dashes for ranges.',
				'The real elapsed time while something runs.',
			]}
			donts={[
				'Round, guess or prettify a cost.',
				'A percentage or a bar for work of unknown length.',
				'Mono inside a running sentence.',
				'Strip a provider’s ID or reason.',
			]}
		/>
	</DocSection>
</DocPage>
