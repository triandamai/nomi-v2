<script lang="ts">
	import Reasoning from '$lib/components/ds/chat/Reasoning.svelte';
	import Thought from '$lib/components/ds/chat/Thought.svelte';
	import ThoughtSteps from '$lib/components/ds/chat/ThoughtSteps.svelte';
	import PlanCard from '$lib/components/ds/chat/PlanCard.svelte';
	import DraftPlanCard from '$lib/components/ds/chat/DraftPlanCard.svelte';
	import GallerySection from '../GallerySection.svelte';
	import GalleryBoard from '../GalleryBoard.svelte';

	let draft = $state([
		{ label: 'Read the rent transfers in your BCA statements', owner: { agent: 'maya' as const, name: 'Maya' } },
		{ label: "Record each month's rent as an expense", owner: { agent: 'dana' as const, name: 'Dana' } },
		{ label: 'Remind you on the 25th, three days before rent is due', owner: { agent: 'kala' as const, name: 'Kala' } },
		{ label: 'Share the plan with your housemate', owner: { agent: 'tara' as const, name: 'Tara' } },
		{ label: "File each month's receipt", owner: { agent: 'maya' as const, name: 'Maya' } },
	]);

	function move(from: number, to: number) {
		const next = [...draft];
		const [step] = next.splice(from, 1);
		next.splice(to, 0, step);
		draft = next;
	}
</script>

{#snippet reasoning()}
	<div class="flex flex-col gap-5">
		<Reasoning label="Thought for 8s" text="" />
		<Reasoning live label="Thinking" elapsed="00:04" text="Looking at October in Dana's records. Food is up against September, so I'm checking whether that's GoFood orders or groceries before I…" />
		<Reasoning
			label="Thought for 8s"
			open
			text={'You asked why October spending feels high. Dana has 142 transactions this month against 118 in September.\n\nFood is the jump: Rp 2,1 jt vs Rp 1,4 jt. Most of the difference is GoFood after 21:00 on weekdays, not groceries. Answer with the number first, then the pattern, then one thing to try.'}
		/>
	</div>
{/snippet}

{#snippet thought()}
	<div class="flex flex-col gap-5">
		<Thought text="You asked about this month only, so I'll leave last year's data out." />
		<ThoughtSteps
			label="Worked through 4 steps"
			meta="· 12s"
			steps={[
				{ label: 'Read October transactions', meta: 'dana.records · 142 rows · 1.1s', state: 'done' },
				{ label: 'Grouped by category and hour', meta: 'dana.categorize · 0.4s', state: 'done' },
				{ label: 'Compared with September', meta: 'dana.compare · 0.3s', state: 'done' },
				{ label: 'Drafting the answer', meta: 'nomi · now', state: 'running' },
			]}
		/>
	</div>
{/snippet}

{#snippet plan()}
	<PlanCard
		subject="Weekly money check"
		onstop={() => {}}
		steps={[
			{ label: 'Pull October transactions', meta: '142 rows', state: 'done', owner: { agent: 'dana', name: 'Dana' } },
			{ label: 'Find the categories that grew', meta: 'food +48% vs September', state: 'done', owner: { agent: 'dana', name: 'Dana' } },
			{ label: 'Draft a weekly food budget', meta: 'running · 00:06', state: 'running', owner: { agent: 'nomi', name: 'Nomi' } },
			{ label: 'Set a Sunday check-in reminder', meta: 'waits for step 3', state: 'waiting', owner: { agent: 'kala', name: 'Kala' } },
		]}
	/>
{/snippet}

{#snippet draftPlan()}
	<DraftPlanCard
		intro="How I'd set up the kos rent tracker. Reorder, reassign or remove steps; nothing runs until you say so."
		steps={draft}
		estimate="~2 min · Maya, Dana, Kala"
		onmove={move}
		onremove={(index) => (draft = draft.filter((_, at) => at !== index))}
	/>
{/snippet}

<GallerySection title="Thinking & planning" description="How the crew shows its work. Collapsed by default once the answer lands; live while it runs. Never theatrical.">
	<div class="grid items-start gap-x-8 gap-y-6 lg:grid-cols-2">
		<GalleryBoard title="Reasoning" note="collapsed · live · expanded">{@render reasoning()}</GalleryBoard>
		<GalleryBoard title="Thought" note="single · with steps">{@render thought()}</GalleryBoard>
		<GalleryBoard title="Plan" note="running · step statuses">{@render plan()}</GalleryBoard>
		<GalleryBoard title="Draft plan" note="editable · nothing runs yet">{@render draftPlan()}</GalleryBoard>
	</div>
</GallerySection>
