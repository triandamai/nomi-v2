<!-- Questions an agent asked before carrying on (Rena before planning an app, Koda when the plan
     leaves something open), with answers to tap and room for the person's own words. Sending
     posts their answers as an ordinary message, through the chat's composer. -->
<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import { composeAnswers, useChatSend } from '$lib/chatSend';
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
	// svelte-ignore state_referenced_locally
	let picked = $state<string[][]>(block.questions.map(() => []));
	let note = $state('');
	let sent = $state(false);
	const done = $derived(answered || sent);
	const message = $derived(composeAnswers(block.questions, picked, note));

	function toggle(q: number, label: string) {
		if (done) return;
		const current = picked[q];
		const multiple = block.questions[q].multiple;
		picked[q] = current.includes(label) ? current.filter((l) => l !== label) : multiple ? [...current, label] : [label];
	}

	function submit(event: SubmitEvent) {
		event.preventDefault();
		if (!message || done || !send) return;
		send(message);
		sent = true;
	}
</script>

<form class="questions" class:questions--done={done} onsubmit={submit}>
	<header class="questions__head">
		<AgentShape {agent} size={28} face />
		<span class="questions__title">{block.intro ?? m.questions_title()}</span>
	</header>

	{#each block.questions as question, q (q)}
		<fieldset class="question" disabled={done}>
			<legend class="question__text">{question.question}</legend>
			{#if question.multiple}<span class="question__hint">{m.questions_pick_any()}</span>{/if}
			<div class="question__options">
				{#each question.options as option (option.label)}
					{@const selected = picked[q].includes(option.label)}
					<button
						type="button"
						class="option"
						class:option--selected={selected}
						aria-pressed={selected}
						onclick={() => toggle(q, option.label)}
					>
						<span class="option__label">
							{#if selected}<svg width="16" height="16" viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12.5l4.5 4.5L19 7.5" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" /></svg>{/if}
							{option.label}
						</span>
						{#if option.detail}<span class="option__detail">{option.detail}</span>{/if}
					</button>
				{/each}
			</div>
		</fieldset>
	{/each}

	{#if done}
		<p class="questions__sent">{m.questions_answered()}</p>
	{:else}
		<label class="questions__note">
			<span class="sr-only">{m.questions_note_label()}</span>
			<textarea rows="2" bind:value={note} placeholder={m.questions_note_placeholder()}></textarea>
		</label>
		<div class="questions__actions">
			<button type="submit" class="questions__send" disabled={!message || !send}>{m.questions_send()}</button>
		</div>
	{/if}
</form>

<style>
	.questions {
		display: flex;
		flex-direction: column;
		gap: 14px;
		padding: 16px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container);
		color: var(--md-sys-color-on-surface);
	}
	.questions__head {
		display: flex;
		align-items: center;
		gap: 10px;
	}
	.questions__title {
		font-weight: 700;
		font-size: 1rem;
		line-height: 1.35;
	}
	.question {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin: 0;
		padding: 0;
		border: none;
		min-width: 0;
	}
	.question__text {
		padding: 0;
		font-weight: 600;
		line-height: 1.4;
	}
	.question__hint {
		margin-top: -4px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.question__options {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}
	.option {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 2px;
		max-width: 100%;
		min-height: 40px;
		padding: 8px 12px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface-container-low);
		color: inherit;
		font: inherit;
		text-align: left;
		cursor: pointer;
		transition: background-color var(--nomi-motion-effects-fast), border-color var(--nomi-motion-effects-fast);
	}
	.option:hover:not(:disabled) {
		background: var(--md-sys-color-surface-container-high);
	}
	.option:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.option--selected {
		border-color: transparent;
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.option:disabled {
		cursor: default;
	}
	.questions--done .option:not(.option--selected) {
		opacity: 0.55;
	}
	.option__label {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		font-weight: 600;
		font-size: 0.9375rem;
	}
	.option__detail {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
		line-height: 1.35;
	}
	.option--selected .option__detail {
		color: inherit;
		opacity: 0.85;
	}
	.questions__note textarea {
		box-sizing: border-box;
		width: 100%;
		padding: 12px 12px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface-container-lowest);
		color: inherit;
		font: inherit;
		resize: vertical;
	}
	.questions__note textarea:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 1px;
	}
	.questions__actions {
		display: flex;
		justify-content: flex-end;
	}
	.questions__send {
		min-height: 40px;
		padding: 0 16px;
		border: none;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
		font: inherit;
		font-weight: 600;
		cursor: pointer;
	}
	.questions__send:disabled {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 12%, transparent);
		color: color-mix(in srgb, var(--md-sys-color-on-surface) 38%, transparent);
		cursor: default;
	}
	.questions__send:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.questions__sent {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	@media (prefers-reduced-motion: reduce) {
		.option {
			transition: none;
		}
	}
</style>
