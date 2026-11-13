<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { checklist } from '$lib/adapters/blocks';
	import type { CrewId } from '$lib/ds/agents/crew';
	import PlanCard from '$lib/ds/chat/PlanCard.svelte';
	import TurnNote from '$lib/ds/thread/TurnNote.svelte';
	import { CircleAlert } from '@lucide/svelte';
	import { callJson } from './client';

	// A plan a crew member made in this chat (Flow / Chat — Plan → approve → act, HU6nc): its steps
	// from the newest version's checklist, each done or waiting, read from the chat's agent-plans.
	// While the crew is at work, Stop sends "stop", as the backend expects.
	type Props = { api: string; agentSessionId: string; planId: string; title: string; agent: CrewId; name: string; working: boolean; onstop: () => void };
	let { api, agentSessionId, planId, title, agent, name, working, onstop }: Props = $props();

	type Plan = { id: string; title: string; version: number; content: string | null; state: string };
	let plan = $state<Plan | null>(null);
	let failed = $state(false);

	$effect(() => {
		const url = `${api}/agent-plans/${agentSessionId}`;
		void callJson<{ plans: Plan[] }>(url).then((result) => {
			if (!result.ok) return void (failed = true);
			const plans = result.data.plans ?? [];
			plan = plans.at(-1) ?? plans.find((one) => one.id === planId) ?? null;
		});
	});

	const steps = $derived(
		checklist(plan?.content).map((step) => ({ label: step.text, state: step.done ? ('done' as const) : ('waiting' as const), owner: { agent, name } })),
	);
</script>

{#if failed}
	<TurnNote icon={CircleAlert} tone="danger" text={m.plan_load_failed()} />
{:else if steps.length > 0}
	<PlanCard subject={plan?.title ?? title} {steps} onstop={working && plan?.state === 'active' ? onstop : undefined} />
{/if}
