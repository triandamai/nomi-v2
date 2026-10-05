<script lang="ts">
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import WavyProgress from '$lib/components/m3/WavyProgress.svelte';
	import { agentLook, GRADIENT_STOPS, TONE_ACCENT } from '$lib/components/m3/shapes';
	import type { HomeSummary } from '$lib/types';

	// Home's three status cards (design: Home artboard): what moved while the user was away,
	// what's due today, and plans still in progress.

	let { summary }: { summary: HomeSummary } = $props();

	function time(iso: string): string {
		try {
			return new Intl.DateTimeFormat(undefined, { hour: '2-digit', minute: '2-digit', hourCycle: 'h23', timeZone: summary.timezone }).format(
				new Date(iso),
			);
		} catch {
			return new Date(iso).toTimeString().slice(0, 5);
		}
	}

	function agentName(agent: string): string {
		if (agent === 'chitchat') return 'Nomi';
		return agent.charAt(0).toUpperCase() + agent.slice(1);
	}

	const RECURRENCE: Record<string, string> = { daily: 'Daily', weekly: 'Weekly', monthly: 'Monthly' };

	function tint(agent: string): string {
		return GRADIENT_STOPS[agentLook(agent).tone].at(-1) ?? '#5be08f';
	}
	function accent(agent: string): string {
		return TONE_ACCENT[agentLook(agent).tone];
	}
	function chatHref(sessionId: string | null): string | undefined {
		return sessionId ? `/chat/${sessionId}` : undefined;
	}
</script>

<div class="cards">
	<section class="card" aria-labelledby="out-heading">
		<h2 id="out-heading" class="card__title">While you were out</h2>
		{#if summary.while_you_were_out.length === 0}
			<p class="card__empty">All quiet. Nothing new from the crew since you last looked.</p>
		{:else}
			<ul class="out">
				{#each summary.while_you_were_out as item, i (i)}
					<li>
						<a class="out__item" href={chatHref(item.session_id)} style="--tint: {tint(item.agent)}; --accent: {accent(item.agent)}">
							<span class="out__dot" class:out__dot--urgent={item.kind === 'needs_you'} aria-hidden="true"></span>
							<span class="out__text">
								<span class="out__title">{item.title}</span>
								{#if item.detail}<span class="out__detail">{item.detail}</span>{/if}
							</span>
						</a>
					</li>
				{/each}
			</ul>
		{/if}
	</section>

	<section class="card" aria-labelledby="today-heading">
		<div class="card__head">
			<h2 id="today-heading" class="card__title">Today</h2>
			<a class="card__link" href="/reminders">Reminders</a>
		</div>
		{#if summary.today.length === 0}
			<p class="card__empty">Nothing scheduled for today. Ask Nomi to remind you of something, or add it on Reminders.</p>
		{:else}
			<ul class="today">
				{#each summary.today as item (item.id)}
					<li class="today__row">
						<span class="today__time">{time(item.run_at)}</span>
						<span class="today__text">
							<span class="today__label">{item.label}</span>
							<span class="today__meta">{item.recurrence ? RECURRENCE[item.recurrence] : `From ${agentName(item.agent)}`}</span>
						</span>
					</li>
				{/each}
			</ul>
		{/if}
	</section>

	<section class="card" aria-labelledby="plans-heading">
		<h2 id="plans-heading" class="card__title">Plans in progress</h2>
		{#if summary.plans.length === 0}
			<p class="card__empty">No plans on the go. Ask Planning to draft one and its progress shows up here.</p>
		{:else}
			<ul class="plans">
				{#each summary.plans as plan, i (i)}
					<li>
						<a class="plans__item" href="/chat/{plan.session_id}">
							<span class="plans__row">
								<AgentShape agent={plan.agent} size={20} />
								<span class="plans__name">{plan.title}</span>
								<span class="plans__count">{plan.done} / {plan.total}</span>
							</span>
							<WavyProgress
								value={plan.done / plan.total}
								tone={agentLook(plan.agent).tone}
								label="{plan.title}: {plan.done} of {plan.total} done"
							/>
						</a>
					</li>
				{/each}
			</ul>
		{/if}
	</section>
</div>

<style>
	.cards {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(min(100%, 300px), 1fr));
		gap: 16px;
		align-items: start;
	}
	.card {
		display: flex;
		flex-direction: column;
		gap: 12px;
		padding: 24px;
		border-radius: var(--md-sys-shape-corner-extra-large-increased);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
		min-width: 0;
	}
	.card__head {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 12px;
	}
	.card__title {
		margin: 0;
		font-size: 1.125rem;
		font-weight: 700;
	}
	.card__link {
		color: var(--md-sys-color-primary);
		font-weight: 600;
		font-size: 0.875rem;
		text-decoration: none;
		padding: 4px 2px;
	}
	.card__empty {
		margin: 0;
		font-size: 0.9375rem;
		line-height: 1.5;
		color: var(--md-sys-color-on-surface-variant);
	}
	ul {
		list-style: none;
		margin: 0;
		padding: 0;
	}

	.out {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.out__item {
		display: flex;
		gap: 12px;
		padding: 14px;
		border-radius: 20px;
		background: color-mix(in srgb, var(--tint) 16%, var(--md-sys-color-surface-container-lowest));
		color: inherit;
		text-decoration: none;
		transition: border-radius var(--nomi-motion-spatial-fast);
	}
	a.out__item:hover {
		border-radius: var(--md-sys-shape-corner-medium);
	}
	.out__dot {
		flex: none;
		width: 10px;
		height: 10px;
		margin-top: 6px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--accent);
	}
	.out__dot--urgent {
		box-shadow: 0 0 0 4px color-mix(in srgb, var(--accent) 22%, transparent);
	}
	.out__text {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.out__title {
		font-weight: 600;
		font-size: 0.9375rem;
	}
	.out__detail {
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
		overflow: hidden;
		text-overflow: ellipsis;
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
	}

	.today__row {
		display: grid;
		grid-template-columns: 56px minmax(0, 1fr);
		gap: 12px;
		padding: 12px 0;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.today__row:last-child {
		border-bottom: none;
	}
	.today__time {
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.8125rem;
		color: var(--md-sys-color-primary);
		padding-top: 2px;
		font-variant-numeric: tabular-nums;
	}
	.today__text {
		display: flex;
		flex-direction: column;
		min-width: 0;
	}
	.today__label {
		font-weight: 600;
		font-size: 0.9375rem;
	}
	.today__meta {
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}

	.plans {
		display: flex;
		flex-direction: column;
		gap: 18px;
	}
	.plans__item {
		display: flex;
		flex-direction: column;
		gap: 8px;
		color: inherit;
		text-decoration: none;
	}
	.plans__row {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 0.9375rem;
	}
	.plans__name {
		flex: 1;
		min-width: 0;
		font-weight: 600;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.plans__item:hover .plans__name {
		text-decoration: underline;
	}
	.plans__count {
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
		color: var(--md-sys-color-on-surface-variant);
		font-variant-numeric: tabular-nums;
	}
</style>
