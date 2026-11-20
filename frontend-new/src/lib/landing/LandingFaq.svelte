<script lang="ts" module>
	export type FaqItem = { question: string; answer: string };
	export type AskCard = { title: string; note: string; prompt: string; start: string };
</script>

<script lang="ts">
	import { Minus, Plus } from '@lucide/svelte';
	import Icon from '$lib/ds/foundations/Icon.svelte';
	import LandingMascot from './LandingMascot.svelte';

	// Figma, Landing v2 → FAQ (170:3016), from its layer metadata and the page's render (the design
	// context wasn't read: the Figma MCP's limit). Left, 440: the title and "Ask Nomi yourself", a
	// green card (24 inset, 16 apart) whose button starts a chat with its prompt, through sign in.
	// Right: the questions, 8 apart, one open at a time (the first to start, as drawn). Closed, a
	// white row 20 / 24 inset; open, it gains a 1.5 edge (drawn inside, so the words move 1.5 in, as
	// the design's 25.5 / 21.5) and its answer 12 below. Only the first answer is the design's; the
	// others say what Nomi does (docs/ux/work/landing).
	type Props = { id: string; title: string; ask: AskCard; action: string; items: FaqItem[] };
	let { id, title, ask, action, items }: Props = $props();
	const uid = $props.id();
	let open = $state(0);
</script>

<section {id} class="faq" aria-labelledby="{id}-title">
	<div class="inner">
		<div class="left">
			<h2 id="{id}-title">{title}</h2>
			<form class="ask" method="POST" {action}>
				<span class="who">
					<LandingMascot agent="nomi" size={44.59} />
					<span class="who-text"><span class="who-title">{ask.title}</span><span class="who-note">{ask.note}</span></span>
				</span>
				<p class="prompt">“{ask.prompt}”</p>
				<input type="hidden" name="text" value={ask.prompt} />
				<button type="submit" class="start">{ask.start}</button>
			</form>
		</div>
		<ul class="questions">
			{#each items as item, index (item.question)}
				{@const expanded = open === index}
				<li class={['item', expanded && 'open']}>
					<h3>
						<button type="button" class="row" aria-expanded={expanded} aria-controls="{uid}-{index}" onclick={() => (open = expanded ? -1 : index)}>
							<span class="question">{item.question}</span>
							<span class="toggle"><Icon icon={expanded ? Minus : Plus} size={20} /></span>
						</button>
					</h3>
					<div id="{uid}-{index}" class="answer" hidden={!expanded}>{item.answer}</div>
				</li>
			{/each}
		</ul>
	</div>
</section>

<style>
	.faq {
		background: var(--lp-bg-subtle);
	}

	.inner {
		display: flex;
		align-items: flex-start;
		gap: 64px;
		max-width: 1440px;
		margin: 0 auto;
		padding: 120px 64px;
	}

	.left {
		display: flex;
		flex-direction: column;
		gap: 20px;
		width: 440px;
		flex-shrink: 0;
	}

	h2 {
		margin: 0;
		font-size: clamp(34px, 4.4vw, 56px);
		line-height: 1.04;
		font-weight: 500;
		letter-spacing: -0.04em;
	}

	.ask {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 16px;
		padding: 24px;
		border-radius: var(--lp-radius-xl);
		background: var(--lp-green-subtle);
	}

	.who {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.who-text {
		display: flex;
		flex-direction: column;
	}

	.who-title {
		font-size: 16px;
		line-height: 22px;
		font-weight: 600;
		letter-spacing: -0.16px;
	}

	.who-note {
		font-family: var(--nm-mono);
		font-size: 12px;
		line-height: 16px;
		color: var(--lp-text-2);
	}

	.prompt {
		max-width: 360px;
		margin: 0;
		font-size: 16px;
		line-height: 24px;
		letter-spacing: -0.16px;
	}

	.start {
		display: inline-flex;
		align-items: center;
		height: 40px;
		padding: 0 16px;
		border: 0;
		border-radius: var(--lp-radius-sm);
		background: var(--lp-primary);
		color: var(--lp-primary-fg);
		font: inherit;
		font-size: 14px;
		line-height: 16px;
		font-weight: 500;
		letter-spacing: 0.28px;
		text-transform: uppercase;
		cursor: pointer;
	}

	.start:hover {
		opacity: 0.88;
	}

	.questions {
		display: flex;
		flex: 1 0 0;
		flex-direction: column;
		gap: 8px;
		min-width: 0;
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.item {
		padding: 20px 24px;
		border-radius: 20px;
		background: var(--lp-surface);
	}

	.open {
		padding: 21.5px 25.5px;
		outline: 1.5px solid var(--lp-border-strong);
		outline-offset: -1.5px;
	}

	h3 {
		margin: 0;
	}

	.row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
		width: 100%;
		padding: 0;
		border: 0;
		border-radius: 4px;
		background: none;
		font: inherit;
		color: var(--lp-text);
		text-align: left;
		cursor: pointer;
	}

	.row:focus-visible,
	.start:focus-visible {
		outline: 2px solid var(--lp-focus);
		outline-offset: 2px;
	}

	.toggle {
		display: flex;
		flex-shrink: 0;
	}

	.question {
		font-size: 16px;
		line-height: 22px;
		font-weight: 600;
		letter-spacing: -0.16px;
	}

	.answer {
		max-width: 640px;
		margin-top: 12px;
		font-size: 16px;
		line-height: 1.55;
		letter-spacing: -0.16px;
		color: var(--lp-text-2);
	}

	@media (max-width: 1023px) {
		.inner {
			flex-direction: column;
			padding: 80px 24px;
		}

		.left,
		.questions {
			width: 100%;
		}
	}

	@media (max-width: 599px) {
		.inner {
			padding: 64px 16px;
		}
	}
</style>
