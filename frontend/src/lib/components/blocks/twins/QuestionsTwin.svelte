<script lang="ts">
	// Questions an agent needs answered before it goes on (ClarifyCard), one at a time. The answers
	// go back as the person's next message, through the chat's own composer; then the card folds to
	// one line (ClarifyAnswered). Escape skips them all, which also says so in the chat.
	import { m } from '$lib/paraglide/messages';
	import ClarifyCard from '$lib/components/ds/chat/ClarifyCard.svelte';
	import ClarifyAnswered from '$lib/components/ds/chat/ClarifyAnswered.svelte';
	import ClarifyReview from '$lib/components/ds/chat/ClarifyReview.svelte';
	import type { ClarifyAnswer, ClarifyQuestion } from '$lib/components/ds/chat/clarify';
	import { composeAnswers, useChatSend } from '$lib/chatSend';
	import { crewId } from '$lib/crewId';
	import type { ContentBlock } from '$lib/types';

	let {
		block,
		agent = null,
		answered = false,
	}: {
		block: Extract<ContentBlock, { kind: 'questions' }>;
		agent?: string | null;
		/** The person already replied after these questions. */
		answered?: boolean;
	} = $props();

	const send = useChatSend();
	let sent = $state<string | null>(null);
	let open = $state(false);
	const done = $derived(answered || sent !== null);

	const questions = $derived<ClarifyQuestion[]>(
		block.questions.map((question, index) => ({
			topic: m.questions_topic({ number: index + 1 }),
			title: question.question,
			multiple: question.multiple ?? false,
			options: question.options.map((option) => ({ title: option.label, detail: option.detail ?? undefined })),
			other: !question.multiple,
		})),
	);

	/** What each answer says, in the words the person picked or typed. */
	function picked(answer: ClarifyAnswer, index: number): string[] {
		const options = block.questions[index].options;
		if (!answer) return [];
		if ('choice' in answer) return [options[answer.choice].label];
		if ('choices' in answer) return answer.choices.map((at) => options[at].label);
		return [answer.other.trim()];
	}

	// Two questions or more end on a review of the answers before they go (Chat responses →
	// Clarify / Review); each can be changed from there.
	let reviewing = $state<ClarifyAnswer[] | null>(null);
	// What was answered, so going back from the review keeps it.
	let lastAnswers = $state<ClarifyAnswer[] | null>(null);
	let step = $state(0);

	function finish(answers: ClarifyAnswer[]) {
		if (block.questions.length < 2) reply(composeAnswers(block.questions, answers.map(picked), ''));
		else reviewing = answers;
	}

	const reviewed = $derived(
		(reviewing ?? []).map((answer, index) => ({ topic: questions[index].topic, value: picked(answer, index).join(', ') || m.questions_skipped() })),
	);

	function reply(text: string) {
		if (!send || !text) return;
		send(text);
		sent = text;
	}
</script>

{#if done}
	<ClarifyAnswered count={block.questions.length} summary={sent ?? block.intro ?? m.questions_answered()} onopen={() => (open = !open)} />
	{#if open}
		<ol class="asked">
			{#each block.questions as question, index (index)}<li>{question.question}</li>{/each}
		</ol>
	{/if}
{:else if reviewing}
	<ClarifyReview
		answers={reviewed}
		onedit={(index) => {
			step = index;
			reviewing = null;
		}}
		onback={() => {
			step = block.questions.length - 1;
			reviewing = null;
		}}
		onconfirm={() => reviewing && reply(composeAnswers(block.questions, reviewing.map(picked), ''))}
	/>
{:else}
	<ClarifyCard
		agent={crewId(agent)}
		title={block.intro ?? m.questions_title()}
		{questions}
		bind:step
		answers={lastAnswers ?? undefined}
		onfinish={(answers) => {
			lastAnswers = answers;
			finish(answers);
		}}
		onskipall={() => reply(m.questions_skip_reply())}
	/>
{/if}

<style>
	.asked {
		margin: 8px 0 0;
		padding-left: 20px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: var(--md-sys-typescale-body-small-size);
	}
</style>
