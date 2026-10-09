<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import LoadingIndicator from '$lib/components/m3/LoadingIndicator.svelte';
	import PlanCard from '$lib/components/PlanCard.svelte';
	import type { PlansForUser } from '$lib/types';

	// The plans on offer (Admin → Plans), loaded when the sheet opens, with the person's own
	// marked. Plans are changed by the Nomi team for now; the sheet says how to ask.
	let { open = $bindable(false) }: { open?: boolean } = $props();

	let plans = $state<PlansForUser | null>(null);
	let failed = $state(false);

	$effect(() => {
		if (!open || plans) return;
		failed = false;
		fetch('/plans')
			.then((r) => (r.ok ? (r.json() as Promise<PlansForUser>) : Promise.reject()))
			.then((loaded) => (plans = loaded))
			.catch(() => (failed = true));
	});
</script>

<BottomSheet bind:open>
	<div class="plans-sheet">
		<div class="plans-sheet__head">
			<AgentShape agent="nomi" face size={48} />
			<div>
				<h2 class="plans-sheet__title">{m.plans_title()}</h2>
				<p class="plans-sheet__lede">{m.plans_lede()}</p>
			</div>
		</div>
		{#if plans}
			<div class="plans-sheet__grid">
				{#each plans.plans as plan (plan.id)}
					<PlanCard {plan} current={plan.id === plans.current_plan_id} />
				{/each}
			</div>
		{:else if failed}
			<p class="plans-sheet__lede">{m.plans_failed()}</p>
		{:else}
			<div class="plans-sheet__loading"><LoadingIndicator size={36} label={m.plans_title()} /></div>
		{/if}
		<p class="plans-sheet__note">{m.plans_how_to_change()}</p>
		<div class="plans-sheet__actions">
			<Button variant="filled" size="m" onclick={() => (open = false)}>{m.pro_got_it()}</Button>
		</div>
	</div>
</BottomSheet>

<style>
	.plans-sheet {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}
	.plans-sheet__head {
		display: flex;
		align-items: center;
		gap: 14px;
	}
	.plans-sheet__title {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.625rem;
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}
	.plans-sheet__lede,
	.plans-sheet__note {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.plans-sheet__note {
		font-size: 0.875rem;
	}
	.plans-sheet__grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
		gap: 12px;
	}
	.plans-sheet__loading {
		display: flex;
		justify-content: center;
		padding: 24px;
	}
	.plans-sheet__actions {
		display: flex;
		justify-content: flex-end;
	}
</style>
