<script lang="ts">
	import { tick, untrack } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import { answersText, replyBlocks, type ReplyBlock } from '$lib/adapters/blocks';
	import { ownerList } from '$lib/adapters/phoneChat';
	import { isAnswered } from '$lib/ds/chat/clarify';
	import PhoneClarifySheet, { answerText, startProgress, type ClarifyProgress } from '$lib/ds/thread/PhoneClarifySheet.svelte';
	import PhoneFlowBar from '$lib/ds/thread/PhoneFlowBar.svelte';
	import PhoneRunSheet from '$lib/ds/thread/PhoneRunSheet.svelte';
	import type { ThreadEntry } from './thread';
	import type { PhoneFlows } from './phoneFlows.svelte';

	// What waits on the person in a chat on a phone, over its composer (Flow / Mobile Chat — Clarify →
	// plan GCvfD, Plan → approve → act G1tK6L): the crew's questions in their sheet, which opens on
	// its own when the latest reply asks them and parks as "{n} questions waiting · Answer" when
	// closed (UX contract G1); and a plan nothing runs yet, as its bar ("Plan ready · nothing runs
	// yet", "Plan v2 ready") opening "Run this plan?", which names the steps and their owners. The answers and the
	// way to run go back as the person's message, as the backend reads them (D1).
	type Props = {
		flows: PhoneFlows;
		entries: ThreadEntry[];
		/** The chat's name, for the questions' progress line. */
		chat: string;
		language: string;
		/** Something is sending or running: nothing new is offered meanwhile. */
		working: boolean;
		onsend: (text: string) => void;
		/** Puts the person in the composer (Keep planning, after Run plan). */
		onfocus: () => void;
	};
	let { flows, entries, chat, language, working, onsend, onfocus }: Props = $props();

	let answerButton: HTMLButtonElement | undefined = $state();
	let announcement = $state('');

	type Found<K extends ReplyBlock['kind']> = { block: Extract<ReplyBlock, { kind: K }>; messageId: string };
	const latest = $derived(entries.filter((entry): entry is Extract<ThreadEntry, { type: 'reply' }> => entry.type === 'reply' && entry.latest && !entry.failure));
	function find<K extends ReplyBlock['kind']>(kind: K): Found<K> | null {
		for (const entry of [...latest].reverse()) {
			const block = replyBlocks(entry.message, { language }).find((one): one is Extract<ReplyBlock, { kind: K }> => one.kind === kind);
			if (block) return { block, messageId: entry.message.id };
		}
		return null;
	}
	const asked = $derived.by(() => {
		const found = find('clarify');
		return found && !flows.answered[found.block.key] ? found : null;
	});
	const progress = $derived(asked ? (flows.progress[asked.block.key] ?? startProgress(asked.block.questions)) : null);
	const planned = $derived.by(() => {
		const found = find('plan');
		const seen = found ? flows.plans[found.messageId] : undefined;
		return found && seen?.state === 'proposed' && !flows.ran[found.messageId] ? { ...found, seen } : null;
	});

	// The questions open on their own once, when the latest reply asks them.
	$effect(() => {
		const key = asked?.block.key;
		if (!key || working) return;
		untrack(() => {
			if (flows.opened.has(key)) return;
			flows.opened.add(key);
			flows.asking = key;
			flows.sheetOpen = true;
		});
	});

	function setProgress(next: ClarifyProgress) {
		if (asked) flows.progress = { ...flows.progress, [asked.block.key]: next };
	}

	function finish(done: ClarifyProgress) {
		if (!asked) return;
		const { block } = asked;
		const picked = block.questions.map((question, index) => [answerText(question, done.answers[index] ?? null)].filter(Boolean));
		const defaults = done.skipped.filter((skipped, index) => skipped && isAnswered(done.answers[index] ?? null)).length;
		flows.answered = { ...flows.answered, [block.key]: { count: block.questions.length, defaults } };
		flows.sheetOpen = false;
		onsend(answersText(block.asked.map((question) => ({ question })), picked));
	}

	async function parked() {
		flows.sheetOpen = false;
		await tick();
		answerButton?.focus();
	}

	const RUN = $derived([
		{ title: m.phone_run_option_run(), recommended: true },
		// What its research steps are isn't said yet (BACKEND_TODO: a plan's steps); what it won't do is.
		{ title: m.phone_run_option_research(), detail: m.phone_run_option_research_detail() },
		{ title: m.phone_run_option_keep(), detail: m.phone_run_option_keep_detail() },
	]);

	function run(index: number) {
		if (!planned) return;
		flows.runOpen = false;
		if (index === 2) {
			flows.notice = m.phone_plan_changing({ version: String(planned.seen.version) });
		} else {
			flows.ran = { ...flows.ran, [planned.messageId]: true };
			onsend(RUN[index].title);
			announcement = m.phone_run_sent();
		}
		onfocus();
	}

	const remaining = $derived(asked && progress ? Math.max(1, asked.block.questions.length - Math.min(progress.step, asked.block.questions.length - 1)) : 0);
</script>

<p class="sr-only" aria-live="polite">{announcement}</p>
{#if asked && progress && !flows.sheetOpen && !working}
	<PhoneFlowBar bind:button={answerButton} label={remaining === 1 ? m.phone_clarify_waiting_one() : m.phone_clarify_waiting({ count: String(remaining) })} action={m.phone_clarify_answer()} onclick={() => (flows.sheetOpen = true)} />
{:else if planned && !flows.notice && !working}
	{@const plan = planned.seen}
	<!-- One bar for a plan nothing runs yet (ux-lead, gate 3 answer 1): it opens the sheet, it runs nothing. -->
	<PhoneFlowBar
		label={plan.version > 1 ? m.phone_plan_ready_version({ version: String(plan.version) }) : m.phone_plan_ready()}
		action={m.phone_plan_review_run()}
		onclick={() => (flows.runOpen = true)}
	/>
{/if}

{#if asked && progress}
	{#key asked.block.key}
		<PhoneClarifySheet
			open={flows.sheetOpen}
			{chat}
			questions={asked.block.questions}
			onfinish={finish}
			onclose={() => {
				// Closed with Close, Escape or the scrim: kept where it was, with the bar to come back.
				if (flows.sheetOpen) void parked();
			}}
			bind:progress={() => progress, setProgress}
		/>
	{/key}
{/if}
{#if planned}
	<PhoneRunSheet
		open={flows.runOpen}
		subtitle={m.phone_plan_steps_owners({ count: String(planned.seen.steps), owners: ownerList(planned.seen.owners) })}
		options={RUN}
		onrun={run}
		onclose={() => (flows.runOpen = false)}
	/>
{/if}
