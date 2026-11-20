<script lang="ts" module>
	export type StepsCopy = {
		title: string;
		lead: string;
		steps: [{ title: string; body: string }, { title: string; body: string }, { title: string; body: string }];
		ask: { file: string; meta: string; message: string; reply: string };
		build: { head: string; badge: string; tasks: [string, string, string, string]; working: string };
		live: { url: string; badge: string; app: string; note: string; items: [string, string][]; share: string; export: string };
	};
</script>

<script lang="ts">
	import LandingBadge from './LandingBadge.svelte';
	import LandingMascot from './LandingMascot.svelte';
	import done from './assets/status-done.svg?url';
	import working from './assets/status-working.svg?url';
	import pending from './assets/pending-ring.png';

	// Figma, Landing v2 → How it works (170:2850): the section head, then three steps side by side,
	// each a tinted panel (32 inset, radius 32) with its Step Card (170:2752: the number, the crew
	// member, the title and what happens) over a picture of that moment. The pictures are pictures:
	// their buttons and the task list are decoration, so they're hidden from screen readers (the
	// step's own words say it).
	type Props = { id: string; copy: StepsCopy };
	let { id, copy }: Props = $props();
	const MASCOTS = ['nomi', 'koda', 'tara'] as const;
	const TINTS = ['green', 'yellow', 'blue'] as const;
</script>

<section {id} class="how" aria-labelledby="{id}-title">
	<div class="head">
		<h2 id="{id}-title">{copy.title}</h2>
		<p>{copy.lead}</p>
	</div>
	<ol class="steps">
		{#each copy.steps as step, index (index)}
			<li class={['step', TINTS[index]]}>
				<div class="card">
					<div class="top">
						<span class="number" aria-hidden="true">{index + 1}</span>
						<LandingMascot agent={MASCOTS[index]} size={72} />
					</div>
					<h3>{step.title}</h3>
					<p>{step.body}</p>
				</div>
				<div class="visual" aria-hidden="true">
					{#if index === 0}
						<div class="attachment">
							<span class="photo"></span>
							<span class="meta"><span class="file">{copy.ask.file}</span><span class="mono quiet">{copy.ask.meta}</span></span>
						</div>
						<p class="message">{copy.ask.message}</p>
						<span class="reply"><LandingMascot agent="nomi" size={22.3} /><span class="mono">{copy.ask.reply}</span></span>
					{:else if index === 1}
						<span class="row"><span class="mono quiet">{copy.build.head}</span><LandingBadge label={copy.build.badge} tone="yellow" /></span>
						{#each copy.build.tasks as task, taskIndex (task)}
							<span class="task">
								{#if taskIndex < 2}
									<img src={done} alt="" width="20" height="20" />
									<span>{task}</span>
								{:else if taskIndex === 2}
									<img src={working} alt="" width="20" height="20" />
									<span>{task}</span>
									<span class="mono brand">{copy.build.working}</span>
								{:else}
									<span class="ring"><img src={pending} alt="" width="15" height="15" /></span>
									<span class="quiet-text">{task}</span>
								{/if}
							</span>
						{/each}
					{:else}
						<span class="row"><span class="mono">{copy.live.url}</span><LandingBadge label={copy.live.badge} tone="green" /></span>
						<span class="preview">
							<span class="app">{copy.live.app}</span>
							<span class="note">{copy.live.note}</span>
							{#each copy.live.items as [item, price] (item)}
								<span class="item"><span>{item}</span><span class="mono">{price}</span></span>
							{/each}
						</span>
						<span class="buttons"><span class="button primary">{copy.live.share}</span><span class="button secondary">{copy.live.export}</span></span>
					{/if}
				</div>
			</li>
		{/each}
	</ol>
</section>

<style>
	.how {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 64px;
		max-width: 1440px;
		margin: 0 auto;
		padding: 120px 64px;
	}

	.head {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 16px;
		text-align: center;
	}

	/* Display/XL: 56 / 1.04, −2.24 */
	h2 {
		max-width: 1000px;
		margin: 0;
		font-size: clamp(34px, 4.4vw, 56px);
		line-height: 1.04;
		font-weight: 500;
		letter-spacing: -0.04em;
		text-wrap: balance;
	}

	.head p {
		max-width: 560px;
		margin: 0;
		font-size: 18px;
		line-height: 1.55;
		letter-spacing: -0.18px;
		color: var(--lp-text-2);
	}

	.steps {
		display: flex;
		gap: 24px;
		width: 100%;
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.step {
		display: flex;
		flex: 1 0 0;
		flex-direction: column;
		gap: 32px;
		min-width: 0;
		padding: 32px;
		border-radius: var(--lp-radius-2xl);
	}

	.green {
		background: var(--lp-green-subtle);
	}

	.yellow {
		background: var(--lp-yellow-subtle);
	}

	.blue {
		background: var(--lp-blue-subtle);
	}

	.card {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.top {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.number {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 56px;
		height: 56px;
		border-radius: 999px;
		background: var(--lp-primary);
		color: var(--lp-text-inverse);
		font-size: 24px;
		line-height: 30px;
		font-weight: 600;
		letter-spacing: -0.48px;
	}

	/* Heading/L: 24 / 30 semibold */
	h3 {
		margin: 0;
		font-size: 24px;
		line-height: 30px;
		font-weight: 600;
		letter-spacing: -0.48px;
	}

	.card p {
		margin: 0;
		font-size: 16px;
		line-height: 1.55;
		letter-spacing: -0.16px;
		color: var(--lp-text-2);
	}

	.visual {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 12px;
		padding: 15px;
		border: 1px solid var(--lp-border-subtle);
		border-radius: var(--lp-radius-xl);
		background: var(--lp-surface);
	}

	.mono {
		font-family: var(--nm-mono);
		font-size: 12px;
		line-height: 16px;
		color: var(--lp-text);
		white-space: nowrap;
	}

	.quiet {
		color: var(--lp-text-3);
	}

	.brand {
		color: var(--lp-text-brand);
	}

	.attachment {
		display: flex;
		align-items: center;
		gap: 12px;
		width: 100%;
		padding: 8px;
		border-radius: var(--lp-radius-md);
		background: var(--lp-bg-muted);
	}

	.photo {
		width: 36px;
		height: 36px;
		flex-shrink: 0;
		border-radius: 6px;
		background: var(--lp-yellow-solid);
	}

	.meta {
		display: flex;
		flex-direction: column;
	}

	.file {
		font-size: 14px;
		line-height: 18px;
		font-weight: 500;
		letter-spacing: -0.14px;
	}

	.message {
		margin: 0;
		font-size: 16px;
		line-height: 1.5;
		letter-spacing: -0.16px;
	}

	.reply {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.reply .mono {
		color: var(--lp-text-2);
	}

	.row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		width: 100%;
	}

	.task {
		display: flex;
		align-items: center;
		gap: 12px;
		font-size: 16px;
		line-height: 22px;
		letter-spacing: -0.16px;
	}

	.task img {
		display: block;
		flex-shrink: 0;
	}

	.ring {
		position: relative;
		width: 20px;
		height: 20px;
		flex-shrink: 0;
	}

	.ring img {
		position: absolute;
		top: 2.5px;
		left: 2.5px;
	}

	.quiet-text {
		color: var(--lp-text-2);
	}

	.preview {
		display: flex;
		flex-direction: column;
		gap: 8px;
		width: 100%;
		padding: 16px;
		border-radius: var(--lp-radius-md);
		background: var(--lp-bg-subtle);
	}

	.app {
		font-size: 16px;
		line-height: 22px;
		font-weight: 600;
		letter-spacing: -0.16px;
	}

	.note {
		font-size: 12px;
		line-height: 16px;
		color: var(--lp-text-2);
	}

	.item {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		font-size: 14px;
		line-height: 20px;
		letter-spacing: -0.14px;
	}

	.item .mono {
		color: var(--lp-text-2);
	}

	.buttons {
		display: flex;
		gap: 8px;
	}

	.button {
		display: inline-flex;
		align-items: center;
		height: 32px;
		padding: 0 12px;
		border-radius: var(--lp-radius-sm);
		font-size: 14px;
		line-height: 16px;
		font-weight: 500;
		letter-spacing: 0.28px;
		text-transform: uppercase;
		white-space: nowrap;
	}

	.primary {
		background: var(--lp-primary);
		color: var(--lp-primary-fg);
	}

	.secondary {
		padding: 0 11px;
		border: 1px solid var(--lp-border);
		background: var(--lp-surface);
	}

	@media (max-width: 1023px) {
		.how {
			padding: 80px 24px;
		}

		.steps {
			flex-direction: column;
		}
	}

	@media (max-width: 599px) {
		.how {
			padding: 64px 16px;
		}

		.step {
			padding: 24px;
		}
	}
</style>
