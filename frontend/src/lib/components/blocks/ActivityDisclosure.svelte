<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	// How a reply came about: the agent's thinking, the steps it took (searching, reading a page,
	// writing a file, handing over) and the pages it used. One quiet line by default, opening in
	// place into a timeline with the sources at the end.
	import { sourceSite, stepLabel } from '$lib/agentLabels';
	import type { ReplySource, ReplyStep } from '$lib/types';

	let {
		thinking = [],
		steps = [],
		sources = [],
		live = false,
	}: {
		/** Thinking, oldest first. */
		thinking?: string[];
		steps?: ReplyStep[];
		sources?: ReplySource[];
		/** No reply yet: the agent may still be thinking. */
		live?: boolean;
	} = $props();

	let open = $state(false);
	let expanded = $state<Record<number, boolean>>({});
	let outputs = $state<Record<number, boolean>>({});
	const isLong = (text: string) => text.length > 240 || text.split('\n').length > 3;
	const uid = $props.id();

	const summary = $derived(
		[
			live ? m.activity_thinking() : thinking.length > 0 ? m.activity_thought() : null,
			steps.length === 1 ? m.activity_step_one() : steps.length > 1 ? m.activity_steps({ count: steps.length }) : null,
			sources.length === 1 ? m.activity_source_one() : sources.length > 1 ? m.activity_sources({ count: sources.length }) : null,
		].filter(Boolean),
	);
</script>

<div class="activity" class:activity--open={open} class:activity--live={live}>
	<button type="button" class="activity__toggle" aria-expanded={open} aria-controls="{uid}-body" onclick={() => (open = !open)}>
		<svg class="activity__icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
			<path d="M9 18h6M10 21h4M12 3a6 6 0 0 0-3.5 10.9c.6.4 1 1.1 1 1.8V16h5v-.3c0-.7.4-1.4 1-1.8A6 6 0 0 0 12 3Z" />
		</svg>
		<span class="activity__label">{summary[0]}</span>
		{#each summary.slice(1) as part (part)}
			<span class="activity__part">· {part}</span>
		{/each}
		<svg class="activity__chevron" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m6 9 6 6 6-6" /></svg>
	</button>

	<div id="{uid}-body" class="activity__body" hidden={!open}>
		<ol class="activity__timeline">
			{#each thinking as text, i (i)}
				{@const long = isLong(text)}
				<li class="activity__item activity__item--thought">
					<span class="activity__dot" aria-hidden="true"></span>
					<div class="activity__content">
						<p class="activity__thought" class:activity__thought--clamped={long && !expanded[i]}>{text}</p>
						{#if long}
							<button type="button" class="activity__more" aria-expanded={!!expanded[i]} onclick={() => (expanded[i] = !expanded[i])}>
								{expanded[i] ? m.reasoning_show_less() : m.reasoning_show_more()}
							</button>
						{/if}
					</div>
				</li>
			{/each}
			{#each steps as step, i (i)}
				<li class="activity__item" class:activity__item--failed={!step.ok}>
					<span class="activity__dot" aria-hidden="true"></span>
					<div class="activity__content">
						<span class="activity__step">{stepLabel(step.tool, step.detail)}</span>
						{#if step.detail && step.tool !== 'delegate_to_agent'}<code class="activity__detail">{step.detail}</code>{/if}
						{#if !step.ok}<span class="activity__failed">{m.activity_failed()}</span>{/if}
						{#if step.output}
							<button type="button" class="activity__more" aria-expanded={!!outputs[i]} aria-controls="{uid}-out-{i}" onclick={() => (outputs[i] = !outputs[i])}>
								{outputs[i] ? m.activity_hide_output() : m.activity_show_output()}
							</button>
						{/if}
					</div>
					{#if step.output && outputs[i]}
						<pre id="{uid}-out-{i}" class="activity__output">{step.output}</pre>
					{/if}
				</li>
			{/each}
		</ol>
		{#if sources.length > 0}
			<div class="activity__sources">
				<span class="activity__sources-title">{m.activity_sources_title()}</span>
				<ul>
					{#each sources as source (source.url)}
						<li>
							<a class="source" href={source.url} target="_blank" rel="noopener noreferrer" title={source.url}>
								<span class="source__mark" aria-hidden="true">{sourceSite(source.url).charAt(0).toUpperCase()}</span>
								<span class="source__text">
									<span class="source__title">{source.title ?? sourceSite(source.url)}</span>
									{#if source.title}<span class="source__site">{sourceSite(source.url)}</span>{/if}
								</span>
							</a>
						</li>
					{/each}
				</ul>
			</div>
		{/if}
	</div>
</div>

<style>
	.activity {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 6px;
		max-width: 100%;
	}
	.activity__toggle {
		display: inline-flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 6px;
		min-height: 36px;
		padding: 0 12px 0 10px;
		border: none;
		border-radius: 18px;
		background: var(--md-sys-color-surface-container);
		color: var(--md-sys-color-on-surface-variant);
		font: inherit;
		font-size: 0.875rem;
		font-weight: 600;
		text-align: left;
		cursor: pointer;
		transition:
			border-radius var(--nomi-motion-spatial-fast),
			background-color var(--nomi-motion-effects-fast);
	}
	.activity__toggle:hover {
		background: var(--md-sys-color-surface-container-high);
	}
	.activity--open .activity__toggle {
		border-radius: var(--md-sys-shape-corner-medium);
	}
	.activity__toggle:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.activity__part {
		font-weight: 500;
	}
	.activity__chevron {
		transition: transform var(--nomi-motion-spatial-fast);
	}
	.activity--open .activity__chevron {
		transform: rotate(180deg);
	}
	.activity--live .activity__icon {
		animation: think 1.6s ease-in-out infinite;
	}
	@keyframes think {
		50% {
			opacity: 0.45;
		}
	}

	.activity__body {
		width: 100%;
		padding: 4px 0 4px 6px;
	}
	.activity__timeline {
		position: relative;
		display: flex;
		flex-direction: column;
		gap: 10px;
		margin: 0;
		padding: 0 0 0 20px;
		list-style: none;
	}
	/* The line through every step. */
	.activity__timeline::before {
		content: '';
		position: absolute;
		top: 6px;
		bottom: 6px;
		left: 4px;
		width: 2px;
		border-radius: 1px;
		background: var(--md-sys-color-outline-variant);
	}
	.activity__item {
		position: relative;
	}
	.activity__dot {
		position: absolute;
		top: 6px;
		left: -20px;
		width: 10px;
		height: 10px;
		border: 2px solid var(--md-sys-color-surface);
		border-radius: 50%;
		background: var(--md-sys-color-primary);
		box-sizing: border-box;
	}
	.activity__item--thought .activity__dot {
		background: var(--md-sys-color-outline);
	}
	.activity__item--failed .activity__dot {
		background: var(--md-sys-color-error);
	}
	.activity__content {
		display: flex;
		flex-wrap: wrap;
		align-items: baseline;
		gap: 4px 8px;
		min-width: 0;
		font-size: 0.875rem;
		line-height: 1.5;
	}
	.activity__thought {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		white-space: pre-wrap;
	}
	.activity__thought--clamped {
		display: -webkit-box;
		overflow: hidden;
		-webkit-box-orient: vertical;
		-webkit-line-clamp: 3;
		line-clamp: 3;
	}
	.activity__more {
		padding: 0;
		border: none;
		background: none;
		color: var(--md-sys-color-primary);
		font: inherit;
		font-size: 0.8125rem;
		font-weight: 600;
		cursor: pointer;
	}
	.activity__step {
		color: var(--md-sys-color-on-surface);
		font-weight: 600;
	}
	.activity__detail {
		max-width: 100%;
		overflow: hidden;
		padding: 1px 6px;
		border-radius: var(--md-sys-shape-corner-extra-small);
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.activity__output {
		max-height: 260px;
		margin: 6px 0 0;
		padding: 10px 12px;
		overflow: auto;
		border-radius: var(--md-sys-shape-corner-medium);
		background: #0f1a14;
		color: #d7e6dc;
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
		line-height: 1.5;
		white-space: pre-wrap;
		word-break: break-word;
	}
	.activity__failed {
		color: var(--md-sys-color-error);
		font-size: 0.8125rem;
	}

	.activity__sources {
		margin-top: 12px;
	}
	.activity__sources-title {
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.6875rem;
		letter-spacing: 0.08em;
		text-transform: uppercase;
	}
	.activity__sources ul {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		margin: 6px 0 0;
		padding: 0;
		list-style: none;
	}
	.source {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		max-width: 260px;
		padding: 6px 12px 6px 6px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container);
		color: var(--md-sys-color-on-surface);
		text-decoration: none;
		transition:
			background-color var(--nomi-motion-effects-fast),
			border-radius var(--nomi-motion-spatial-fast);
	}
	.source:hover {
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface-container-high);
	}
	.source:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.source__mark {
		display: inline-flex;
		flex: none;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border-radius: 50%;
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
		font-size: 0.75rem;
		font-weight: 700;
	}
	.source__text {
		display: flex;
		flex-direction: column;
		min-width: 0;
		line-height: 1.25;
	}
	.source__title,
	.source__site {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.source__title {
		font-size: 0.8125rem;
		font-weight: 600;
	}
	.source__site {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.6875rem;
	}
	@media (prefers-reduced-motion: reduce) {
		.activity--live .activity__icon {
			animation: none;
		}
	}
</style>
