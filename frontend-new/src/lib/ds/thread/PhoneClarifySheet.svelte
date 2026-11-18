<script lang="ts" module>
	import { initialAnswer, type ClarifyAnswer, type ClarifyQuestion } from '../chat/clarify';

	/** Where the questions stand: the question on screen (`questions.length` is the review), each answer, and which were skipped. */
	export type ClarifyProgress = { step: number; answers: ClarifyAnswer[]; skipped: boolean[] };

	/** A fresh start: each question's own starting answer (its recommendation, or nothing). */
	export function startProgress(questions: ClarifyQuestion[]): ClarifyProgress {
		return { step: 0, answers: questions.map((question) => initialAnswer(question)), skipped: questions.map(() => false) };
	}

	/** What a question's answer reads as, in words ("Whoosh train"; ticked ones joined). */
	export function answerText(question: ClarifyQuestion, answer: ClarifyAnswer): string {
		if (!answer) return '';
		if ('other' in answer) return answer.other.trim();
		if ('choices' in answer) return answer.choices.map((choice) => question.options[choice]?.title ?? '').join(', ');
		return question.options[answer.choice]?.title ?? '';
	}
</script>

<script lang="ts">
	import { ArrowRight, Info, List, Pencil } from '@lucide/svelte';
	import { tick } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Tag from '../status/Tag.svelte';
	import BottomSheet from '../overlays/BottomSheet.svelte';
	import PhoneOption from '../mobile/PhoneOption.svelte';
	import PhoneButton from '../mobile/PhoneButton.svelte';
	import Progress from '../mobile/Progress.svelte';
	import { isAnswered } from '../chat/clarify';

	// Flow / Mobile Chat — Clarify → plan (GCvfD), steps 1–5: the crew's questions before a plan, one
	// at a time in a sheet over the thread. "Before I plan" with the chat and how far along, the
	// segmented progress, the question at 20 / 600 (its helper at 14 when there is one), the options
	// as Mobile / Option rows 8 apart (numbered; a recommended one picked to start and tagged), and
	// on a one-of-many question "Something else…", which turns into the person's own answer and folds
	// the options to "Show the {n} suggestions" so the field and Next stay above the keyboard. Many of
	// many ticks boxes and counts them on Review. Skip takes the recommendation (a DEFAULT in the
	// review) or leaves it skipped. The review ("Your answers · Nothing runs yet") lists each answer;
	// tapping one goes back to it. Make the plan hands the answers over. Closing keeps where the
	// person was (`progress`), so the page can bring them back to the same question.
	type Props = {
		open?: boolean;
		/** The chat, for the progress line ("Bandung weekend · 1 of 4"). */
		chat: string;
		questions: ClarifyQuestion[];
		progress?: ClarifyProgress;
		onfinish: (progress: ClarifyProgress) => void;
		onclose?: () => void;
	};
	let { open = $bindable(false), chat, questions, progress = $bindable(startProgress(questions)), onfinish, onclose }: Props = $props();

	const id = $props.id();
	let typing = $state(false);
	let body: HTMLElement | undefined = $state();
	let announcement = $state('');

	const total = $derived(questions.length);
	const reviewing = $derived(progress.step >= total);
	const question = $derived(questions[Math.min(progress.step, total - 1)]);
	const answer = $derived(progress.answers[progress.step] ?? null);
	const other = $derived(answer && 'other' in answer ? answer.other : '');
	const ticked = $derived(answer && 'choices' in answer ? answer.choices.length : 0);

	function set(next: Partial<ClarifyProgress>) {
		progress = { ...progress, ...next };
	}
	function setAnswer(value: ClarifyAnswer, skipped = false) {
		const answers = [...progress.answers];
		const skips = [...progress.skipped];
		answers[progress.step] = value;
		skips[progress.step] = skipped;
		set({ answers, skipped: skips });
	}
	async function go(step: number) {
		typing = Boolean(questions[step] && progress.answers[step] && 'other' in progress.answers[step]!);
		set({ step });
		await tick();
		if (step < total) announcement = m.phone_clarify_announce({ n: String(step + 1), total: String(total), question: questions[step].title });
		focusFirst();
	}
	function focusFirst() {
		const target = body?.querySelector<HTMLElement>('input[type="text"], input:checked, input');
		target?.focus();
	}
	function next() {
		if (!isAnswered(answer)) return;
		void go(progress.step + 1);
	}
	function skip() {
		const recommended = question.options.findIndex((option) => option.recommended);
		setAnswer(recommended >= 0 && !question.multiple ? { choice: recommended } : null, true);
		void go(progress.step + 1);
	}
	function toggle(choice: number) {
		const current = answer && 'choices' in answer ? answer.choices : [];
		setAnswer({ choices: current.includes(choice) ? current.filter((one) => one !== choice) : [...current, choice].sort((a, b) => a - b) });
	}
	async function openOther() {
		typing = true;
		if (!(answer && 'other' in answer)) setAnswer({ other: '' });
		await tick();
		body?.querySelector<HTMLInputElement>('input[type="text"]')?.focus();
	}
	async function showSuggestions() {
		typing = false;
		if (answer && 'other' in answer && !answer.other.trim()) setAnswer(initialAnswer(question));
		await tick();
		body?.querySelector<HTMLInputElement>('input[type="radio"]')?.focus();
	}

	$effect(() => {
		if (open) void tick().then(focusFirst);
	});
</script>

<BottomSheet
	bind:open
	title={reviewing ? m.clarify_review_title() : m.phone_clarify_eyebrow()}
	subtitle={reviewing ? m.phone_clarify_review_sub() : m.phone_clarify_progress({ chat, n: String(progress.step + 1), total: String(total) })}
	{onclose}
>
	<p class="sr-only" aria-live="polite">{announcement}</p>
	<div bind:this={body} class={['flex flex-col px-4 pb-2', !reviewing && 'gap-2']}>
		{#if reviewing}
			{#each questions as one, index (index)}
				{@const text = answerText(one, progress.answers[index] ?? null)}
				<button
					type="button"
					aria-label={m.phone_clarify_change({ topic: one.topic, answer: text || m.phone_clarify_skipped() })}
					class="flex w-full cursor-pointer items-center gap-3 border-b border-border pt-3.5 pb-3.25 text-left -outline-offset-2"
					onclick={() => void go(index)}
				>
					<span class="flex min-w-0 flex-1 flex-col gap-0.5">
						<span class="truncate text-ui text-text-3">{one.title}</span>
						<span class="text-phone text-text">{text || m.phone_clarify_skipped()}</span>
					</span>
					{#if progress.skipped[index] && text}<Tag kind="neutral" label={m.clarify_default()} />{/if}
					<span class="inline-flex shrink-0 text-text-3"><Icon icon={Pencil} /></span>
				</button>
			{/each}
			{#each questions as one, index (index)}
				{@const text = answerText(one, progress.answers[index] ?? null)}
				{#if progress.skipped[index] && text}
					<p class="flex items-start gap-2 pt-1.5 pb-2 text-ui text-text-3">
						<span class="inline-flex shrink-0 pt-0.5"><Icon icon={Info} size={13} /></span>
						{m.phone_clarify_default_note({ topic: one.topic.toLowerCase(), answer: text })}
					</p>
				{/if}
			{/each}
			<div class="flex gap-2 pt-2">
				<PhoneButton variant="secondary" label={m.action_back()} onclick={() => void go(total - 1)} />
				<PhoneButton label={m.clarify_make_plan()} onclick={() => onfinish(progress)} />
			</div>
		{:else}
			<Progress current={progress.step + 1} {total} />
			<div class="flex flex-col gap-1 pt-2.5 pb-1.5">
				<h3 id="{id}-question" class="text-heading font-semibold text-text">{question.title}</h3>
				{#if question.helper}<p class="text-phone-sm leading-cozy text-text-3">{question.helper}</p>{/if}
			</div>
			{#if question.multiple}
				<div role="group" aria-labelledby="{id}-question" class="flex flex-col gap-2">
					{#each question.options as option, index (index)}
						{@const checked = Boolean(answer && 'choices' in answer && answer.choices.includes(index))}
						<label class={['flex w-full cursor-pointer items-center gap-3 rounded-xl border p-3.25', checked ? 'border-accent bg-accent-bg' : 'border-border-strong']}>
							<span aria-hidden="true" class="inline-flex size-6.5 shrink-0 items-center justify-center rounded-full bg-surface-2 font-mono text-caption text-text-2">{index + 1}</span>
							<span class="flex min-w-0 flex-1 flex-col gap-0.75">
								<span class="text-phone font-medium text-text">{option.title}</span>
								{#if option.detail}<span class="text-ui leading-snug text-text-3">{option.detail}</span>{/if}
								{#if option.recommended}<span><Tag kind="go" label={m.choice_recommended()} /></span>{/if}
							</span>
							<input type="checkbox" {checked} onchange={() => toggle(index)} class="size-5.5 shrink-0 cursor-pointer rounded-xs accent-accent outline-offset-2" />
						</label>
					{/each}
				</div>
			{:else}
				{#if !typing}
					<!-- Fresh radios for each question: a radio the person checked keeps its own state. -->
					{#key progress.step}
					<div role="radiogroup" aria-labelledby="{id}-question" class="flex flex-col gap-2">
						{#each question.options as option, index (index)}
							<PhoneOption
								name="{id}-q{progress.step}"
								key={index + 1}
								title={option.title}
								detail={option.detail}
								recommended={option.recommended}
								checked={Boolean(answer && 'choice' in answer && answer.choice === index)}
								onselect={() => setAnswer({ choice: index })}
							/>
						{/each}
					</div>
					{/key}
				{/if}
				{#if typing}
					<label class="flex w-full items-center gap-3 rounded-xl border border-accent bg-accent-bg p-3.25">
						<span aria-hidden="true" class="inline-flex size-6.5 shrink-0 items-center justify-center rounded-full bg-surface-2 font-mono text-caption text-text-2">{question.options.length + 1}</span>
						<span class="sr-only">{m.phone_clarify_own_answer()}</span>
						<input
							type="text"
							value={other}
							placeholder={m.phone_clarify_other()}
							oninput={(event) => setAnswer({ other: event.currentTarget.value })}
							onkeydown={(event) => event.key === 'Enter' && (event.preventDefault(), next())}
							class="min-w-0 flex-1 bg-transparent text-phone text-text caret-accent outline-none placeholder:text-text-3"
						/>
					</label>
					<button type="button" class="flex cursor-pointer items-center gap-1.5 self-start rounded-xs text-ui text-text-2 outline-offset-2 pointer-coarse:min-h-11" onclick={() => void showSuggestions()}>
						<span class="inline-flex text-text-3"><Icon icon={List} size={12} /></span>
						{m.phone_clarify_show_suggestions({ count: String(question.options.length) })}
					</button>
				{:else}
					<button type="button" class="flex w-full cursor-pointer items-center gap-3 rounded-xl border border-border-strong p-3.25 text-left outline-offset-2" onclick={() => void openOther()}>
						<span aria-hidden="true" class="inline-flex size-6.5 shrink-0 items-center justify-center rounded-full bg-surface-2 font-mono text-caption text-text-2">{question.options.length + 1}</span>
						<span class="text-phone text-text-3">{m.phone_clarify_other()}</span>
					</button>
				{/if}
			{/if}
			<div class="flex gap-2 pt-2">
				{#if typing}
					<PhoneButton variant="secondary" label={m.action_back()} onclick={() => void showSuggestions()} />
				{:else if question.multiple && progress.step > 0}
					<PhoneButton variant="secondary" label={m.action_back()} onclick={() => void go(progress.step - 1)} />
				{:else}
					<PhoneButton variant="secondary" label={m.action_skip()} onclick={skip} />
				{/if}
				{#if question.multiple}
					<PhoneButton label={m.phone_clarify_review_count({ count: String(ticked) })} icon={ArrowRight} disabled={ticked === 0} onclick={next} />
				{:else}
					<PhoneButton label={m.clarify_next()} icon={ArrowRight} disabled={!isAnswered(answer)} onclick={next} />
				{/if}
			</div>
		{/if}
	</div>
</BottomSheet>
