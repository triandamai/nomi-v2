<script lang="ts">
	import { ArrowUpRight, Bell, RotateCcw } from '@lucide/svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import LiveStep from '$lib/ds/conversation/LiveStep.svelte';
	import ErrorCallout from '$lib/ds/conversation/ErrorCallout.svelte';
	import MessageAgent from '$lib/ds/conversation/MessageAgent.svelte';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';
	import SkeletonRow from '$lib/ds/lists/SkeletonRow.svelte';
	import ProjectRow from '$lib/ds/lists/ProjectRow.svelte';
	import Meter from '$lib/ds/status/Meter.svelte';
	import Status from '$lib/ds/status/Status.svelte';
	import SectionLabel from '$lib/ds/status/SectionLabel.svelte';
	import ResponseActions from '$lib/ds/chat/ResponseActions.svelte';
	import DocPage from '$lib/docs/ui/DocPage.svelte';
	import DocSection from '$lib/docs/ui/DocSection.svelte';
	import DoDont from '$lib/docs/ui/DoDont.svelte';
	import Example from '$lib/docs/ui/Example.svelte';
	import Note from '$lib/docs/ui/Note.svelte';
	import { docHref } from '$lib/docs/href';

	// Foundations → Honest states. The rules from AGENTS.md §2 (Honest states, Interaction and states);
	// the examples are the real components with the gallery's copy (Nomi sheet, Chat → States & errors).
	const noop = () => {};
</script>

<DocPage
	eyebrow="Foundations"
	title="Honest states"
	lede="Show what is really happening: the real tool and target, the time so far, the step it is on. No fake progress, no spinners that pretend, no guessed costs. Every list ships with its empty, loading and error states."
	wide
>
	<DocSection title="Progress in words" lede="When nobody knows how long something takes, say what it is doing and how long it has been at it.">
		<p>
			A loading state names the real step and its target and counts the elapsed time: <code>Deploying · 00:14</code>, <code>Writing
			src/routes/menu/+page.svelte · 01:12</code>. A running step carries its own Stop.
		</p>
		<Example title="A button and a step, both in words" description="The button says what it is doing and is busy until it is done; the live step names the file and the time." align="start">
			<div class="flex w-full max-w-160 flex-col items-start gap-4">
				<Button label="Deploying · 00:14" variant="inverse" loading />
				<LiveStep step="Writing src/routes/menu/+page.svelte" elapsed="01:12" stopLabel="Stop" onstop={noop} />
			</div>
		</Example>
		<h3>Spinners only for the current step</h3>
		<p>
			A spinner marks the step that is running now, and nothing else. Every spinner in the design system stops turning when the person has
			asked for reduced motion (<code>motion-reduce:animate-none</code>); the words still say what is happening.
		</p>
		<h3>No fake progress bars</h3>
		<p>
			No bar that creeps toward 90% while nobody knows the real figure. A bar is only for what can really be measured, like a month's usage, and
			then it shows the real number in words and never goes past full.
		</p>
		<Example title="A bar that is true" description="Usage is measured, so it gets a meter, with the figure in mono." align="start">
			<div class="w-72 max-w-full"><Meter label="Free plan" value={0.02} caption="21.8K / 1M tokens this month" /></div>
		</Example>
	</DocSection>

	<DocSection title="Errors in plain words" lede="Say what happened, quote the provider's own reason, and offer the next move.">
		<ul>
			<li>Say it the way a person would: “Your DeepSeek account is out of credits for this request.”</li>
			<li>Quote the provider exactly, in mono: the status and code, the request id, its own reason, which host said it.</li>
			<li>Give the next move as a button, the main one first: retry, add credits, choose another model.</li>
			<li>Never guess a cost on failure: providers don't report a charge on failed calls.</li>
		</ul>
		<Example title="Out of credits" description="The reply explains; the callout quotes the provider and offers what to do." align="start">
			<div class="flex w-full max-w-180 flex-col gap-4">
				<MessageAgent
					agent="nomi"
					name="Nomi"
					meta="14:02 · deepseek-flash"
					text="I couldn't reply because your DeepSeek account is out of credits for this request. Add credits or raise the API key's limit with DeepSeek, then send your message again."
				/>
				<ErrorCallout
					title="Provider error"
					code="402 insufficient_balance"
					requestId="req_9d41e0b2"
					details={[
						{ key: 'reason', value: 'Insufficient Balance', strong: true },
						{ key: 'provider', value: 'api.deepseek.com' },
					]}
					actions={[
						{ label: 'Retry', icon: RotateCcw, onclick: noop },
						{ label: 'Add credits on DeepSeek', icon: ArrowUpRight, onclick: noop },
					]}
				/>
			</div>
		</Example>
		<p>Every error the chat can meet, from a rejected key to a timeout, is drawn on the <a href={docHref('/design-system/chat', 'states-errors')}>Chat responses sheet</a>.</p>
	</DocSection>

	<DocSection title="Cost always visible" lede="What something costs is never hidden, and never the headline.">
		<p>
			Under every reply, what answered, how long it took and how many tokens it used stay on the right of the action bar, in mono. A build's
			estimate shows its cost as a quiet line under the plan before anything runs. Usage against the plan is a meter with the real figure.
		</p>
		<Example title="Under a reply" description="The model, the time and the tokens, every time." align="start">
			<div class="w-full max-w-160">
				<ResponseActions text="Food is October's biggest jump." meta="deepseek-flash · 1.4s · 812 tokens" onlike={noop} ondislike={noop} onshare={noop} onregenerate={noop} ondelete={noop} />
			</div>
		</Example>
	</DocSection>

	<DocSection title="Four states, shipped together" lede="Empty, loading, error and populated are designed and built at the same time, never one later.">
		<p>
			An empty list says so plainly, with the one thing to do about it. A loading list says what it is loading and stays still: no shimmer. An
			error says what went wrong and what to do. Only then the populated list.
		</p>
		<Example title="One of each" description="The real components, with the Nomi sheet's copy." align="start">
			<div class="grid w-full gap-6 md:grid-cols-2">
				<div class="flex min-w-0 flex-col gap-2">
					<SectionLabel label="Empty" />
					<EmptyState icon={Bell} title="Reminders" count={0} body="Nothing scheduled today. Ask Kala, or add one yourself." action={{ label: 'Add reminder', href: '#reminders' }} />
				</div>
				<div class="flex min-w-0 flex-col gap-2">
					<SectionLabel label="Loading" />
					<SkeletonRow label="loading 4 projects…" />
				</div>
				<div class="flex min-w-0 flex-col gap-2">
					<SectionLabel label="Error" />
					<ErrorCallout
						title="Can't connect"
						code="network_error"
						details={[
							{ key: 'reason', value: 'Failed to fetch: no internet connection', strong: true },
							{ key: 'provider', value: 'nomi.app' },
						]}
						actions={[{ label: 'Retry now', icon: RotateCcw, onclick: noop }]}
					/>
				</div>
				<div class="flex min-w-0 flex-col gap-2">
					<SectionLabel label="Populated" />
					<ProjectRow
						href="#salah"
						name="salah-daily-tracker"
						description="Daily salah tracker with countdown and streaks"
						tone="nomi"
						status={{ kind: 'live', label: 'Live' }}
						url="salah-daily-tracker-0ab8f8.trian.space"
						updated="2d"
					/>
				</div>
			</div>
		</Example>
	</DocSection>

	<DocSection title="State in words, not colour alone" lede="A status is a dot and a word. Whoever is working shows their mascot.">
		<Example title="Statuses" description="Each colour comes with its word; none of them pulse." align="start">
			<Status kind="live" label="Live" />
			<Status kind="ready" label="ready" />
			<Status kind="idle" label="idle" />
			<Status kind="failed" label="Deploy failed" />
			<Status kind="attention" label="Needs attention" />
			<Status kind="working" label="Koda is building" agent="koda" />
		</Example>
		<Note>
			<p>The words themselves (naming the real thing, no "magic") are in <a href={docHref('/design-system/content/voice-and-tone')}>Voice &amp; tone</a>.</p>
		</Note>
	</DocSection>

	<DocSection title="Best practices">
		<DoDont
			dos={[
				'Show elapsed time and the current step when the duration is unknown.',
				'Quote the provider’s own reason, and offer the next move.',
				'Keep cost and usage in view, in mono.',
				'Ship empty, loading, error and populated states together.',
			]}
			donts={[
				'Progress bars that creep toward 90%.',
				'More than one spinner, or a spinner that ignores reduced motion.',
				'A guessed cost on a failed call.',
				'Shimmering skeletons, or an error with no reason and no next move.',
			]}
		/>
	</DocSection>
</DocPage>
