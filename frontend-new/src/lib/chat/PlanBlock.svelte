<script lang="ts">
	import { untrack } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import { checklist } from '$lib/adapters/blocks';
	import { crewIdOf, stepOwner } from '$lib/adapters/chat';
	import { addedSteps } from '$lib/adapters/phoneChat';
	import type { CrewId } from '$lib/ds/agents/crew';
	import PlanCard from '$lib/ds/chat/PlanCard.svelte';
	import PhoneDraftPlan from '$lib/ds/thread/PhoneDraftPlan.svelte';
	import TurnNote from '$lib/ds/thread/TurnNote.svelte';
	import { CircleAlert } from '@lucide/svelte';
	import { callJson } from './client';
	import type { PhoneFlows } from './phoneFlows.svelte';

	// A plan a crew member made in this chat (Flow / Chat — Plan → approve → act, HU6nc): its steps
	// from the newest version's checklist, each done or waiting, read from the chat's agent-plans.
	// While the crew is at work, Stop sends "stop", as the backend expects.
	// On a phone (Flow / Mobile Chat — Clarify → plan, steps 6 and 9; Plan → approve → act, step 1)
	// a plan nothing runs yet is the draft card: DRAFT, or V{n} with the steps the revision added
	// marked "+", each step owned by the crew member who made the plan (the backend doesn't assign
	// steps). The flows hear how it stands, so the run bar shows while it's proposed.
	type Props = {
		api: string;
		agentSessionId: string;
		planId: string;
		title: string;
		agent: CrewId;
		name: string;
		working: boolean;
		onstop: () => void;
		phone?: boolean;
		flows?: PhoneFlows;
		messageId?: string;
	};
	let { api, agentSessionId, planId, title, agent, name, working, onstop, phone = false, flows, messageId }: Props = $props();

	type Plan = { id: string; title: string; version: number; content: string | null; state: string };
	let plan = $state<Plan | null>(null);
	let previous = $state<Plan | null>(null);
	let failed = $state(false);

	$effect(() => {
		const url = `${api}/agent-plans/${agentSessionId}`;
		void callJson<{ plans: Plan[] }>(url).then((result) => {
			if (!result.ok) return void (failed = true);
			const plans = result.data.plans ?? [];
			plan = plans.at(-1) ?? plans.find((one) => one.id === planId) ?? null;
			previous = plans.length > 1 ? plans[plans.length - 2] : null;
		});
	});

	const steps = $derived(
		checklist(plan?.content).map((step) => ({ label: step.text, state: step.done ? ('done' as const) : ('waiting' as const), owner: { agent: phone ? crewIdOf(name) : agent, name } })),
	);
	const draft = $derived(phone && plan !== null && (plan.state === 'proposed' || plan.state === 'updated'));
	const added = $derived(addedSteps(previous ? checklist(previous.content).map((step) => step.text) : null, steps.map((step) => step.label)));

	$effect(() => {
		if (!phone || !flows || !messageId || !plan) return;
		const seen = { version: plan.version, state: plan.state, steps: steps.length, owners: [name] };
		untrack(() => (flows.plans = { ...flows.plans, [messageId]: seen }));
	});
</script>

{#if failed}
	<TurnNote icon={CircleAlert} tone="danger" text={m.plan_load_failed()} />
{:else if draft && steps.length > 0}
	<PhoneDraftPlan
		title={plan?.title ?? title}
		version={plan?.version}
		intro={m.phone_plan_intro()}
		steps={steps.map((step, index) => {
			const owner = stepOwner(step.label, name);
			return { label: owner.label, owner: { agent: owner.agent, name: owner.name }, added: added[index] };
		})}
	/>
{:else if steps.length > 0}
	<PlanCard subject={plan?.title ?? title} {steps} onstop={working && plan?.state === 'active' ? onstop : undefined} />
{/if}
