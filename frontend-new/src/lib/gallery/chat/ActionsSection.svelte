<script lang="ts">
	import { Send } from '@lucide/svelte';
	import ApprovalCard from '$lib/ds/chat/ApprovalCard.svelte';
	import DecisionCard from '$lib/ds/chat/DecisionCard.svelte';
	import SelectionCard from '$lib/ds/chat/SelectionCard.svelte';
	import FormCard from '$lib/ds/chat/FormCard.svelte';
	import FollowUps from '$lib/ds/chat/FollowUps.svelte';
	import QuickReplies from '$lib/ds/chat/QuickReplies.svelte';
	import ClarifyCard from '$lib/ds/chat/ClarifyCard.svelte';
	import ClarifyReview from '$lib/ds/chat/ClarifyReview.svelte';
	import ClarifyAnswered from '$lib/ds/chat/ClarifyAnswered.svelte';
	import PlanApproval from '$lib/ds/chat/PlanApproval.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import Segmented from '$lib/ds/inputs/Segmented.svelte';
	import type { ClarifyQuestion } from '$lib/ds/chat/clarify';
	import GallerySection from '../GallerySection.svelte';
	import GalleryBoard from '../GalleryBoard.svelte';

	const subscriptions = [
		{ name: 'Netflix Premium', detail: 'monthly · BCA debit', amount: '186.000', monthly: 186_000 },
		{ name: 'Spotify Duo', detail: 'monthly · GoPay', amount: '54.990', monthly: 54_990 },
		{ name: 'iCloud+ 200 GB', detail: 'monthly · Visa •• 4412', amount: '45.000', monthly: 45_000 },
		{ name: 'Fitness First', detail: 'monthly · since Mar', amount: '650.000', monthly: 650_000 },
		{ name: 'Adobe Lightroom', detail: 'yearly · last paid Jan', amount: '1.199.000', monthly: 99_917 },
	];
	let picked = $state([true, true, true, false, false]);
	const count = $derived(picked.filter(Boolean).length);
	const total = $derived(subscriptions.reduce((sum, item, index) => sum + (picked[index] ? item.monthly : 0), 0));
	const rupiah = new Intl.NumberFormat('id-ID');

	const questions: ClarifyQuestion[] = [
		{
			topic: 'Getting there',
			title: 'How do you want to get to Bandung?',
			helper: 'You and Sari, from Jakarta. This decides the timing of everything else.',
			options: [
				{ title: 'Whoosh high-speed train', detail: '45 min from Halim, about Rp 250rb each way per person. Fits a one-night trip.', recommended: true },
				{ title: 'Drive', detail: '3–4 hours with weekend traffic, tolls and parking around Rp 400rb.' },
				{ title: 'Travel shuttle', detail: 'Door to door from Kuningan, about 3 hours, Rp 150rb each.' },
			],
			other: true,
		},
		{
			topic: 'Budget',
			title: 'What should the trip cost at most?',
			options: [{ title: 'Rp 3,5 jt for both of us', recommended: true }, { title: 'Rp 5 jt for both of us' }],
			other: true,
		},
		{
			topic: 'Dates',
			title: 'Which weekend?',
			options: [{ title: 'Sat 1 – Sun 2 Nov', recommended: true }, { title: 'Sat 8 – Sun 9 Nov' }],
			other: true,
		},
		{
			topic: 'What to cover',
			title: 'What should the plan cover?',
			helper: 'Pick any. The crew member who owns each part is noted.',
			multiple: true,
			options: [
				{ title: 'Train tickets', detail: 'Nomi finds times; you book on Access by KAI' },
				{ title: 'Where to stay', detail: '3 guesthouses in Dago under Rp 900rb a night' },
				{ title: 'Food spots', detail: "Places Sari can eat (she's vegetarian, from your memory)" },
				{ title: 'Trip budget', detail: 'Dana tracks every expense against Rp 3,5 jt' },
				{ title: 'Reminders', detail: 'Kala: pack Friday night, leave Saturday 06:30' },
				{ title: 'Share with Sari', detail: 'Tara shares the itinerary doc from your Google Drive' },
			],
		},
	];
</script>

<GallerySection
	title="Actions"
	description="Anything that sends, spends, deletes or decides for you stops here and waits. One clear primary per card; the outcome stays visible after you act."
>
	<div class="grid items-start gap-x-8 gap-y-6 lg:grid-cols-2">
		<GalleryBoard title="Approval" note="pending · approved · rejected">
			<div class="flex flex-col gap-5">
				<ApprovalCard
					agent="tara"
					title="Tara wants to send an email"
					fields={[
						{ key: 'to', value: 'bu.sri.kosmawar@gmail.com' },
						{ key: 'from', value: 'you, via Gmail' },
						{ key: 'subject', value: 'Transfer sewa November sudah masuk' },
					]}
					body="Selamat siang Bu Sri, sewa kamar 3 untuk November (Rp 2.200.000) sudah saya transfer ke BCA. Bukti terlampir. Terima kasih."
					attachment="bukti-transfer-nov.jpg · 212 KB"
					rememberLabel="Don't ask again for Bu Sri"
					approveLabel="Approve & send"
					approveIcon={Send}
					rejectLabel="Don't send"
					onapprove={() => {}}
					onreject={() => {}}
					onedit={() => {}}
				/>
				<ApprovalCard
					agent="tara"
					title="Tara sent the email to Bu Sri"
					status="approved"
					outcome="Approved by you · sent 14:06"
					action={{ label: 'View in Gmail', href: 'https://mail.google.com' }}
				/>
				<ApprovalCard
					agent="tara"
					title="Tara's email to Bu Sri"
					status="rejected"
					outcome="You chose not to send · nothing left your Gmail"
					action={{ label: 'Undo', onclick: () => {} }}
				/>
			</div>
		</GalleryBoard>

		<GalleryBoard title="Decision" note="pick one · tradeoffs visible">
			<DecisionCard
				question="When should Kala remind you about rent?"
				options={[
					{ title: '3 days before', detail: '25th, 09:00. Time to top up BCA if needed.', suggested: true },
					{ title: 'On the day', detail: '28th, 07:00. Least noise, no buffer.' },
					{ title: 'Both', detail: '25th and 28th. Two pings a month.' },
				]}
				onconfirm={() => {}}
			/>
		</GalleryBoard>

		<GalleryBoard title="Selection" note="many of many">
			<SelectionCard
				agent="dana"
				question="Which of these should Dana track as subscriptions?"
				items={subscriptions}
				selected={picked}
				total="Rp {rupiah.format(total)} / month"
				confirmLabel="Track {count}"
				onchange={(next) => (picked = next)}
				onconfirm={() => {}}
				onskip={() => {}}
			/>
		</GalleryBoard>

		<GalleryBoard title="Form" note="inline, prefilled from the chat">
			<FormCard
				agent="kala"
				title="New reminder"
				meta="prefilled from chat"
				note="Kala pings you in Inbox and on your phone"
				submitLabel="Save reminder"
				onsubmit={() => {}}
				oncancel={() => {}}
			>
				<TextField label="What" value="Pay kos rent to Bu Sri" />
				<div class="grid gap-3 sm:grid-cols-2">
					<TextField label="Date" value="25 Nov 2026" mono />
					<TextField label="Time" value="09:00" mono />
				</div>
				<div class="flex flex-col gap-1.5">
					<p class="text-ui font-medium text-text">Repeat</p>
					<Segmented
						surface="surface"
						label="Repeat"
						value="monthly"
						options={[
							{ value: 'once', label: 'Once' },
							{ value: 'weekly', label: 'Weekly' },
							{ value: 'monthly', label: 'Monthly' },
							{ value: 'yearly', label: 'Yearly' },
						]}
					/>
				</div>
			</FormCard>
		</GalleryBoard>

		<GalleryBoard title="Suggestions" note="follow-ups · quick replies">
			<div class="flex flex-col gap-4">
				<FollowUps items={['Compare with the last three months', 'Set a weekly food budget of Rp 500rb', 'Show only GoFood orders after 21:00']} onpick={() => {}} />
				<QuickReplies replies={['Yes, set it', 'Change the time', 'Not now']} onpick={() => {}} />
			</div>
		</GalleryBoard>

		<GalleryBoard title="Usage notes" note="">
			<ul class="flex flex-col gap-3 text-ui text-text-2">
				{#each ['Approval waits for every send, spend, delete or external write. Read-only work never asks.', 'Decision is one-of-many with tradeoffs written out; Selection is many-of-many with a running total.', 'Forms appear inline and arrive prefilled. Validation sits under the field, never in a toast.', 'After you act, the card collapses to a one-line outcome so the thread stays readable.', 'Follow-ups are optional next questions; Quick Replies answer the last question directly.'] as note (note)}
					<li class="flex gap-3"><span class="text-text-3">–</span>{note}</li>
				{/each}
			</ul>
		</GalleryBoard>

		<GalleryBoard title="Clarify before planning" note="one question at a time">
			<div class="flex flex-col gap-5">
				<ClarifyCard agent="nomi" title="Before I plan: 4 quick questions" {questions} onfinish={() => {}} onskipall={() => {}} />
				<ClarifyCard
					agent="nomi"
					title="Before I plan: 4 quick questions"
					{questions}
					step={3}
					answers={[{ choice: 0 }, { choice: 0 }, { choice: 0 }, { choices: [0, 1, 2, 3, 5] }]}
					onfinish={() => {}}
					onskipall={() => {}}
				/>
			</div>
		</GalleryBoard>

		<GalleryBoard title="Review & approve" note="answers → plan → run mode">
			<div class="flex flex-col gap-5">
				<ClarifyReview
					answers={[
						{ topic: 'Getting there', value: 'Whoosh high-speed train' },
						{ topic: 'Budget', value: 'Rp 3,5 jt max, for both of us' },
						{ topic: 'Dates', value: 'Sat 1 – Sun 2 Nov', isDefault: true },
						{ topic: 'What to cover', value: 'Trains, stay, food, budget, share with Sari' },
					]}
					assumption="You skipped Dates, so I picked the next free weekend. Your rent reminder is on the 25th, so this weekend is busy."
					onedit={() => {}}
					onback={() => {}}
					onconfirm={() => {}}
				/>
				<ClarifyAnswered count={4} summary="Whoosh · Rp 3,5 jt · 1–2 Nov · 5 topics" onopen={() => {}} />
				<PlanApproval
					options={[
						{ title: 'Run it, ask before anything leaves Nomi', detail: 'Research runs freely. Tara asks before sharing the doc; Dana only records.', recommended: true },
						{ title: 'Research only', detail: 'Find trains, stays and food. Nothing is drafted, shared or tracked.' },
					]}
					onrun={() => {}}
				/>
			</div>
		</GalleryBoard>
	</div>
</GallerySection>
