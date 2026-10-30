<script lang="ts">
	// Questions an agent needs answered before it goes on (ClarifyCard), one at a time. The answers
	// go back as the person's next message, through the chat's own composer; then the card folds to
	// one line (ClarifyAnswered). Escape skips them all, which also says so in the chat.
	import { m } from '$lib/paraglide/messages';
	import ClarifyCard from '$lib/components/ds/chat/ClarifyCard.svelte';
	import ClarifyAnswered from '$lib/components/ds/chat/ClarifyAnswered.svelte';
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
{:else}
	<ClarifyCard
		agent={crewId(agent)}
		title={block.intro ?? m.questions_title()}
		{questions}
		onfinish={(answers) => reply(composeAnswers(block.questions, answers.map(picked), ''))}
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
