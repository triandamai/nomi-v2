<script lang="ts">
	// A sample photo (Unsplash), bundled so the gallery needs nothing from elsewhere.
	import kosRoom from '../images/kos-room.jpg';
	import { Carrot, Egg, FileSpreadsheet, Fish, Milk } from '@lucide/svelte';
	import DataTable from '$lib/ds/chat/DataTable.svelte';
	import Cart from '$lib/ds/chat/Cart.svelte';
	import CodeBlock from '$lib/ds/chat/CodeBlock.svelte';
	import Diff from '$lib/ds/chat/Diff.svelte';
	import Hyperlink from '$lib/ds/chat/Hyperlink.svelte';
	import Citation from '$lib/ds/chat/Citation.svelte';
	import Sources from '$lib/ds/chat/Sources.svelte';
	import LinkPreview from '$lib/ds/chat/LinkPreview.svelte';
	import MentionCrew from '$lib/ds/chat/MentionCrew.svelte';
	import MentionFile from '$lib/ds/chat/MentionFile.svelte';
	import MentionPicker from '$lib/ds/chat/MentionPicker.svelte';
	import Reminder from '$lib/ds/chat/Reminder.svelte';
	import SectionLabel from '$lib/ds/status/SectionLabel.svelte';
	import type { CodeLine } from '$lib/ds/chat/code';
	import GallerySection from '../GallerySection.svelte';
	import GalleryBoard from '../GalleryBoard.svelte';

	let cart = $state([
		{ name: 'Telur ayam negeri', detail: '1 tray · 10 butir', icon: Egg, quantity: 2, price: '58.000' },
		{ name: 'Susu UHT full cream', detail: '1 L', icon: Milk, quantity: 3, price: '57.000' },
		{ name: 'Wortel lokal', detail: '500 g', icon: Carrot, quantity: 1, price: '9.500' },
		{ name: 'Ikan kembung', icon: Fish, quantity: 0, price: '—', available: false, swap: 'Swap for tongkol' },
	]);

	const tokens: CodeLine[] = [
		[{ text: "// Weekly food budget from October's average", kind: 'comment' }],
		[
			{ text: 'export function ', kind: 'keyword' },
			{ text: 'weeklyBudget', kind: 'fn' },
			{ text: '(total: ' },
			{ text: 'number', kind: 'type' },
			{ text: ', weeks = ' },
			{ text: '4', kind: 'number' },
			{ text: '): ' },
			{ text: 'number', kind: 'type' },
			{ text: ' {' },
		],
		[{ text: '  ' }, { text: 'const ', kind: 'keyword' }, { text: 'base = Math.' }, { text: 'round', kind: 'fn' }, { text: '(total / weeks)' }],
		[{ text: '  ' }, { text: 'return ', kind: 'keyword' }, { text: 'base - ' }, { text: '25_000', kind: 'number' }, { text: '  // one less GoFood order', kind: 'comment' }],
		[{ text: '}' }],
		[],
		[{ text: 'weeklyBudget', kind: 'fn' }, { text: '(' }, { text: '2_100_000', kind: 'number' }, { text: ')  ' }, { text: '// → 500_000', kind: 'comment' }],
	];
	const code = tokens.map((line) => line.map((token) => token.text).join('')).join('\n');

	const mentionOptions = [
		{ kind: 'crew' as const, agent: 'dana' as const, name: 'Dana', role: 'Money and subscriptions' },
		{ kind: 'crew' as const, agent: 'kala' as const, name: 'Kala', role: 'Reminders on time' },
		{ kind: 'crew' as const, agent: 'maya' as const, name: 'Maya', role: 'Reads attachments' },
		{ kind: 'file' as const, name: 'rent-2026.xlsx', where: 'Drive', icon: FileSpreadsheet },
	];
	let activeMention = $state(0);
</script>

<GallerySection
	title="Rich content"
	description="Structure only when the content has it. Tables for comparable rows, cards for things you act on, code and diffs in mono."
>
	<div class="grid items-start gap-x-8 gap-y-6 lg:grid-cols-2">
		<GalleryBoard title="Table" note="comparable rows only">
			<DataTable
				title="Food spending by week"
				columns={['Week', 'Groceries', 'GoFood', 'Total', 'vs Sep']}
				rows={[
					['1–7 Oct', '312.000', '168.000', '480.000', { value: '+12%', tone: 'bad' }],
					['8–14 Oct', '286.000', '241.000', '527.000', { value: '+31%', tone: 'bad' }],
					['15–21 Oct', '301.000', '322.000', '623.000', { value: '+48%', tone: 'bad' }],
					['22–28 Oct', '274.000', '196.000', '470.000', { value: '−6%', tone: 'good' }],
				]}
				total={['October', '1.173.000', '927.000', '2.100.000', { value: '+20%', tone: 'bad' }]}
				source="Rp · from Dana · 142 transactions"
				link={{ href: 'https://docs.google.com/spreadsheets', label: 'Open in Sheets' }}
			/>
		</GalleryBoard>

		<GalleryBoard title="Cart" note="you check out; Nomi records">
			<Cart
				title="Weekly groceries"
				store="Sayurbox"
				items={cart}
				subtotal="124.500"
				delivery="10.000"
				total="Rp 134.500"
				note="Prices from Sayurbox at 14:05. Dana records it when you pay."
				checkoutHref="https://www.sayurbox.com"
				onquantity={(index, quantity) => (cart[index].quantity = quantity)}
				onswap={() => {}}
				onsave={() => {}}
			/>
		</GalleryBoard>

		<GalleryBoard title="Code" note="syntax tokens: nm-code-*">
			<CodeBlock language="typescript" file="src/lib/budget.ts" {code} {tokens} />
		</GalleryBoard>

		<GalleryBoard title="Diff" note="Koda's edits, applied on approval">
			<Diff
				path="src/lib/budget.ts"
				hunks={[
					{
						header: '@@ -2,4 +2,6 @@ export function weeklyBudget',
						lines: [
							{ kind: 'context', old: 2, new: 2, text: 'export function weeklyBudget(total: number, weeks = 4) {' },
							{ kind: 'del', old: 3, text: '  return Math.round(total / weeks)' },
							{ kind: 'add', new: 3, text: '  const base = Math.round(total / weeks)' },
							{ kind: 'add', new: 4, text: '  if (base < 300_000) return base' },
							{ kind: 'add', new: 5, text: '  return base - 25_000' },
							{ kind: 'context', old: 4, new: 6, text: '}' },
						],
					},
				]}
				request={{ agent: 'koda', name: 'Koda', files: 1 }}
				onapply={() => {}}
				onreject={() => {}}
			/>
		</GalleryBoard>

		<GalleryBoard title="Links & sources" note="hyperlink · citation · preview">
			<div class="flex flex-col gap-5">
				<SectionLabel label="Hyperlink + citation" />
				<p class="text-body"><Hyperlink href="#rich-sources-1" label="house rules Bu Sri sent" /></p>
				<p class="flex"><Citation n={1} title="Peraturan Kos Mawar 2026.pdf" href="#rich-sources-1" /></p>
				<p class="text-body text-text-2">
					Rent goes up to Rp 2,2 jt from January, per the <Hyperlink href="#rich-sources-1" label="house rules Bu Sri sent" />
					<Citation n={1} title="Peraturan Kos Mawar 2026.pdf" href="#rich-sources-1" />.
				</p>
				<Sources
					id="rich-sources"
					sources={[
						{ title: 'Peraturan Kos Mawar 2026.pdf', detail: 'attachment · read by Maya', kind: 'file' },
						{ title: 'Kos Mawar Tebet — kamar AC', detail: 'kosku.id', kind: 'web', href: 'https://kosku.id' },
					]}
				/>
				<SectionLabel label="URL preview" />
				<LinkPreview
					href="https://kosku.id"
					host="kosku.id"
					title="Kos Mawar Tebet — kamar AC, kamar mandi dalam"
					description="Rp 2.200.000 / bulan · 5 menit ke Stasiun Tebet · listrik token sendiri"
					image={kosRoom}
				/>
			</div>
		</GalleryBoard>

		<GalleryBoard title="Mentions & reminders" note="crew tint only on crew-owned bits">
			<div class="flex flex-col gap-5">
				<SectionLabel label="Mention" />
				<p class="flex"><MentionCrew agent="dana" name="Dana" /></p>
				<p class="flex"><MentionFile name="rent-2026.xlsx" icon={FileSpreadsheet} /></p>
				<p class="text-body text-text-2">Ask <MentionCrew agent="dana" name="Dana" /> to log it in <MentionFile name="rent-2026.xlsx" icon={FileSpreadsheet} /></p>
				<MentionPicker id="rich-mentions" options={mentionOptions} bind:active={activeMention} onselect={() => {}} />
				<SectionLabel label="Reminder" />
				<Reminder title="Pay kos rent to Bu Sri" when="25 Nov · 09:00 · every month" owner={{ agent: 'kala', name: 'Kala' }} onedit={() => {}} />
				<Reminder title="Pay kos rent to Bu Sri" when="25 Nov 09:00 · every month" due doneLabel="Mark paid" onsnooze={() => {}} ondone={() => {}} />
			</div>
		</GalleryBoard>
	</div>
</GallerySection>
