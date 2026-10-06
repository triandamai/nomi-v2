<script lang="ts">
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import WavyProgress from '$lib/components/m3/WavyProgress.svelte';
	import { agentLook } from '$lib/components/m3/shapes';
	import { m } from '$lib/paraglide/messages';
	import type { PlanItem } from '$lib/home';

	let { items }: { items: PlanItem[] } = $props();
</script>

<ul class="plans">
	{#each items as plan, i (i)}
		<li>
			<a class="plans__item" href="/chat/{plan.session_id}">
				<span class="plans__row">
					<AgentShape agent={plan.agent} size={20} />
					<span class="plans__name">{plan.title}</span>
					<span class="plans__count">{plan.total > 0 ? `${plan.done} / ${plan.total}` : m.home_plan_draft()}</span>
				</span>
				{#if plan.total > 0}
					<WavyProgress value={plan.done / plan.total} tone={agentLook(plan.agent).tone} label="{plan.title}: {plan.done} of {plan.total} done" />
				{/if}
			</a>
		</li>
	{/each}
</ul>

<style>
	ul {
		list-style: none;
		margin: 0;
		padding: 0;
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
