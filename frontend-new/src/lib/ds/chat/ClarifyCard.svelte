<script lang="ts">
	import { ArrowRight, Check, Circle, CornerDownLeft } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';
	import ChoiceOption from './ChoiceOption.svelte';
	import { initialAnswer, isAnswered, type ClarifyAnswer, type ClarifyQuestion } from './clarify';

	// Chat responses → Clarify: a few quick questions before planning, one at a time. Tabs show
	// which are answered; the number keys pick, Enter goes on, Escape skips them all. The last
	// step leads to the review.
	type Props = {
		agent: CrewId;
		title: string;
		questions: ClarifyQuestion[];
		/** Where it starts, and what's already answered (when it's reopened from the review). */
		step?: number;
		answers?: ClarifyAnswer[];
		onfinish: (answers: ClarifyAnswer[]) => void;
		onskipall: () => void;
	};
	let { agent, title, questions, step = $bindable(0), answers: given, onfinish, onskipall }: Props = $props();

	const id = $props.id();
	const initial = () => questions.map((question, index) => (given && index < given.length ? given[index] : initialAnswer(question)));
	let answers = $state<ClarifyAnswer[]>(initial());
	// A question counts as answered once you've been to it (the recommendation alone doesn't).
	const initialVisits = () => questions.map((_, index) => index < step);
	let visited = $state(initialVisits());
	let otherField = $state<HTMLInputElement>();

	const question = $derived(questions[step]);
	const answer = $derived(answers[step]);
	const last = $derived(step === questions.length - 1);
	const keys = $derived(question.options.length + (question.other && !question.multiple ? 1 : 0));
	const otherText = $derived(answer && 'other' in answer ? answer.other : '');

	function choose(index: number) {
		answers[step] = { choice: index };
	}

	function toggle(index: number, on: boolean) {
		const current = answer && 'choices' in answer ? answer.choices : [];
		answers[step] = { choices: on ? [...new Set([...current, index])].sort((a, b) => a - b) : current.filter((at) => at !== index) };
	}

	function go(index: number) {
		visited[step] = true;
		step = index;
	}

	function next() {
		if (last) onfinish($state.snapshot(answers));
		else go(step + 1);
	}

	function skip() {
		if (!question.multiple) answers[step] = null;
		next();
	}

	function onkeydown(event: KeyboardEvent) {
		const target = event.target as HTMLElement;
		const typing = target instanceof HTMLInputElement && target.type === 'text';
		if (event.key === 'Escape') {
			event.preventDefault();
			onskipall();
		} else if (event.key === 'Enter' && !(target instanceof HTMLButtonElement)) {
			event.preventDefault();
			next();
		} else if (!typing && /^[1-9]$/.test(event.key)) {
			const index = Number(event.key) - 1;
			if (index >= keys) return;
			event.preventDefault();
			if (index === question.options.length) otherField?.focus();
			else if (question.multiple) toggle(index, !(answer && 'choices' in answer && answer.choices.includes(index)));
			else choose(index);
		}
	}
</script>

<!-- Keys work anywhere inside the card; each control is also reachable on its own. -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<section class="flex w-full flex-col overflow-hidden rounded-md bg-bg outline -outline-offset-1 outline-border-strong" aria-labelledby="{id}-title" {onkeydown}>
	<header class="flex items-center gap-2.5 border-b border-border bg-surface px-3.5 pt-2.5 pb-2.25">
		<Mascot {agent} size={20} />
		<p id="{id}-title" class="min-w-0 flex-1 text-ui font-medium text-text">{title}</p>
		<span class="font-mono text-label text-text-3">{m.clarify_progress({ n: String(step + 1), total: String(questions.length) })}</span>
	</header>
	<nav aria-label={m.clarify_topics()} class="px-3.5 pt-3">
		<ol class="flex flex-wrap gap-1.5">
			{#each questions as item, index (index)}
				{@const current = index === step}
				{@const done = !current && visited[index] && isAnswered(answers[index])}
				<li>
					<button
						type="button"
						aria-current={current ? 'step' : undefined}
						class={[
							'inline-flex h-6 cursor-pointer items-center gap-1.5 rounded-full border px-2.25 text-caption outline-offset-1 pointer-coarse:min-h-11',
							current ? 'border-border-strong bg-surface-2 text-text' : 'border-border hover:text-text',
							!current && (done ? 'text-text-2' : 'text-text-3'),
						]}
						onclick={() => go(index)}
					>
						<span class={['inline-flex', done && 'text-accent']}><Icon icon={done ? Check : Circle} size={12} /></span>{item.topic}
					</button>
				</li>
			{/each}
		</ol>
	</nav>
	<div class="flex flex-col gap-1 px-3.5 pt-3.5 pb-1.5">
		<h3 id="{id}-question" class="text-body font-medium text-text">{question.title}</h3>
		{#if question.helper}<p class="text-caption text-text-3">{question.helper}</p>{/if}
	</div>
	<div role={question.multiple ? 'group' : 'radiogroup'} aria-labelledby="{id}-question" class="flex flex-col gap-1.5 px-3.5 pt-2 pb-3.5">
		{#each question.options as option, index (`${step}-${index}`)}
			<ChoiceOption
				type={question.multiple ? 'checkbox' : 'radio'}
				name="{id}-{step}"
				key={index + 1}
				checked={question.multiple ? !!answer && 'choices' in answer && answer.choices.includes(index) : !!answer && 'choice' in answer && answer.choice === index}
				title={option.title}
				detail={option.detail}
				tag={option.recommended ? m.choice_recommended() : undefined}
				onchange={(checked) => (question.multiple ? toggle(index, checked) : checked && choose(index))}
			/>
		{/each}
		{#if question.other && !question.multiple}
			<div class="flex items-center gap-3 rounded-sm border border-border-strong py-1.25 pr-1.25 pl-2.75 focus-within:outline-2 focus-within:outline-offset-2 focus-within:outline-focus">
				<span aria-hidden="true" class="inline-flex size-5 shrink-0 items-center justify-center rounded-xs bg-surface-2 font-mono text-label text-text-2">{question.options.length + 1}</span>
				<input
					bind:this={otherField}
					type="text"
					aria-label={m.choice_other()}
					placeholder={m.choice_other()}
					value={otherText}
					oninput={(event) => (answers[step] = { other: event.currentTarget.value })}
					class="h-7.5 min-w-0 flex-1 bg-transparent px-1 text-ui text-text outline-none placeholder:text-text-3"
				/>
				<span class="inline-flex pr-1.5 text-text-3"><Icon icon={CornerDownLeft} size={14} /></span>
			</div>
		{/if}
	</div>
	<footer class="flex flex-wrap items-center gap-2 border-t border-border px-3.5 pt-2.25 pb-2.5">
		<span class="min-w-0 flex-1 basis-48 font-mono text-label text-text-3">
			{question.multiple
				? m.clarify_keys_multi({ count: String(answer && 'choices' in answer ? answer.choices.length : 0) })
				: m.clarify_keys({ count: String(keys) })}
		</span>
		<span class="ml-auto flex items-center gap-2">
			<Button label={m.action_skip()} variant="ghost" onclick={skip} />
			<Button label={m.action_back()} variant="secondary" disabled={step === 0} onclick={() => go(step - 1)} />
			<Button label={last ? m.clarify_review() : m.clarify_next()} iconEnd={ArrowRight} onclick={next} />
		</span>
	</footer>
</section>
