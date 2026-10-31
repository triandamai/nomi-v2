<script lang="ts">
	import Status from '$lib/ds/status/Status.svelte';
	import LiveStep from '$lib/ds/conversation/LiveStep.svelte';
	import Handoff from '$lib/ds/chat/Handoff.svelte';
	import Boundary from '$lib/ds/chat/Boundary.svelte';
	import DocPage from '$lib/docs/ui/DocPage.svelte';
	import DocSection from '$lib/docs/ui/DocSection.svelte';
	import DoDont from '$lib/docs/ui/DoDont.svelte';
	import Example from '$lib/docs/ui/Example.svelte';
	import Note from '$lib/docs/ui/Note.svelte';
	import { docHref } from '$lib/docs/href';

	// Content → Voice & tone. AGENTS.md §2 (Copy, Honest states, errors) and the crew's own words
	// on the Chat responses and Build sheets (pen/export) and in messages/en.json.
	const qualities: [title: string, body: string, example: string][] = [
		['Plain', 'Everyday words, one idea per sentence. Say it the way you would say it to the person next to you.', 'Your session ended, so this message wasn’t sent.'],
		['Specific', 'Name the real thing: the crew member, the file, the deploy, the amount, the time.', 'Koda is building · writing src/routes/menu/+page.svelte · 01:12'],
		['Calm', 'No hype and no alarm. Say what is happening and what happens next.', 'DeepSeek is slow right now (replies ~14s). Your chats still work.'],
		['Honest', 'Never pretend: no fake progress, no guessed cost, no promise the product can’t keep.', 'Nothing that left Nomi was undone.'],
	];

	const moments: [moment: string, how: string, lines: string[]][] = [
		['Working', 'Present tense, the real step, the real clock. Never a percentage.', ['Koda is building', 'Step 2 of 6 · menu page · ~3 min left', '3 crew working on this chat', 'You can leave this chat. I’ll post the result here and in Inbox.']],
		['Asking', 'One clear question. Choices with their trade-offs written out. Nothing runs until you say so.', ['Tara wants to send an email', 'When should Kala remind you about rent?', 'Ready to run this plan?', 'Nothing runs yet. You’ll see the plan first.']],
		['Failing', 'What happened, in plain words, then the provider’s own reason, then the next move.', ['DeepSeek didn’t accept your API key, so nothing was sent. It may have been revoked or mistyped.', '1 test failed: no weekend pickup slots after 11:00', 'Nothing went live. Customers still see deploy #2.']],
		['Done', 'The outcome and its real numbers, then out of the way.', ['Built in 5m 40s · used your free allowance', 'Tara sent the email to Bu Sri', 'Approved by you · sent 14:06']],
		['Saying no', 'The limit, the reason, and what the crew can do instead.', ['I can’t cancel subscriptions for you', 'Nomi only records money. It never signs in to Netflix or your bank, so cancelling stays with you.']],
		['Handing off', 'Who answers now and why, with a way back.', ['Nomi handed this to Dana', 'she has your transaction records', 'Ask Nomi instead']],
		['Remembering', 'Memory is never silent: say what was used or saved, and let the person change it.', ['From memory: Sari is vegetarian', 'Saved to memory: you and Sari prefer trains to driving']],
	];

	const noop = () => {};
</script>

<DocPage
	eyebrow="Content"
	title="Voice & tone"
	lede="Nomi sounds the way it looks: an instrument, not a toy. Plain, specific and calm, naming the real thing. The voice stays the same; the tone follows what is happening."
	wide
>
	<DocSection title="The voice" lede="Four qualities, each with a line the product really says.">
		<div class="grid gap-3 md:grid-cols-2">
			{#each qualities as [title, body, example] (title)}
				<div class="flex flex-col gap-2 rounded-md border border-border px-4 pt-3.5 pb-4">
					<h3 class="text-ui font-medium text-text">{title}</h3>
					<p class="text-ui leading-normal text-text-2">{body}</p>
					<p class="border-l-2 border-border-strong pl-3 text-ui leading-normal text-text">{example}</p>
				</div>
			{/each}
		</div>
		<p>
			The rule behind them, from AGENTS.md: <strong>plain, specific, sentence case. Name the real thing</strong> (“Koda is building”, “deploy
			#2”). No “magic”, “effortless” or “AI-powered”.
		</p>
	</DocSection>

	<DocSection title="Who is speaking">
		<ul>
			<li><strong>The crew speak as themselves</strong>, in the first person, inside the chat: “I can’t cancel subscriptions for you”, “Pick a replacement and I’ll use it for this chat.”</li>
			<li><strong>The product speaks about the crew</strong> by name, in the third person: “Koda is building”, “Tara wants to send an email”, “Kala pings you in Inbox and on your phone.”</li>
			<li><strong>The person is “you”</strong>, and what they did is said back plainly: “Approved by you”, “stopped by you · 00:03”, “You deleted Nomi’s reply.”</li>
			<li><strong>“We”</strong> is rare, and only for Nomi as a service: “What should we call you?”, “We’ll notify you when it’s live.”</li>
		</ul>
	</DocSection>

	<DocSection title="Tone by moment" lede="How the crew sounds when working, asking, failing and finishing.">
		<div class="overflow-x-auto rounded-md outline -outline-offset-1 outline-border">
			<table class="w-full min-w-170 border-collapse text-left">
				<thead>
					<tr class="border-b border-border bg-surface font-mono text-label tracking-label text-text-3 uppercase">
						<th scope="col" class="w-32 px-4 py-2.5 font-normal">Moment</th>
						<th scope="col" class="w-72 px-4 py-2.5 font-normal">How it sounds</th>
						<th scope="col" class="px-4 py-2.5 font-normal">In the product</th>
					</tr>
				</thead>
				<tbody>
					{#each moments as [moment, how, lines] (moment)}
						<tr class="border-b border-border align-top last:border-b-0">
							<th scope="row" class="px-4 py-3 text-left text-ui font-medium text-text">{moment}</th>
							<td class="px-4 py-3 text-ui leading-normal text-text-2">{how}</td>
							<td class="px-4 py-3">
								<ul class="flex flex-col gap-1.5">
									{#each lines as line (line)}<li class="text-ui leading-normal text-text">{line}</li>{/each}
								</ul>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	</DocSection>

	<DocSection title="The crew at work" lede="The same lines, in the components that carry them.">
		<Example title="Working, handing off, saying no" align="start">
			<div class="flex w-full max-w-160 flex-col gap-3">
				<Status kind="working" agent="koda" label="Koda is building" />
				<LiveStep step="Writing src/routes/menu/+page.svelte" elapsed="01:12" stopLabel="Stop" onstop={noop} />
				<Handoff from={{ agent: 'nomi', name: 'Nomi' }} to={{ agent: 'dana', name: 'Dana' }} reason="she has your transaction records" onback={noop} />
				<Boundary
					title="I can’t cancel subscriptions for you"
					reason="Nomi only records money. It never signs in to Netflix or your bank, so cancelling stays with you. Here’s what I can do instead:"
					alternatives={[
						{ agent: 'kala', title: 'Remind me 3 days before the next charge', detail: 'Kala · 2 Nov, 09:00', onclick: noop },
						{ agent: 'dana', title: 'Stop tracking it as a subscription', detail: 'Dana · keeps past records', onclick: noop },
					]}
				/>
			</div>
		</Example>
		<p>
			More of the crew’s behaviour, as the sheet draws it: <a href={docHref('/design-system/patterns/chat/crew-behaviour')}>Crew behaviour</a>,
			<a href={docHref('/design-system/patterns/chat/states-errors')}>States &amp; errors</a> and
			<a href={docHref('/design-system/patterns/chat/thinking-planning')}>Thinking &amp; planning</a>.
		</p>
	</DocSection>

	<DocSection title="What Nomi never sounds like">
		<ul>
			<li><strong>Theatrical.</strong> The crew shows its work (“Thought for 8s”, “Worked through 4 steps”), collapsed once the answer lands. No performance of thinking.</li>
			<li><strong>Breathless.</strong> Nomi’s own copy, in the pen and in both message files, has no exclamation marks, no “oops” and no “sorry”.</li>
			<li><strong>Vague about cost or progress.</strong> Cost is always visible and progress is the real step in words: “About 5 min · included in your free plan”, “Step 2 of 6 · menu page · ~3 min left”. No percentage, no bar.</li>
			<li><strong>Sold.</strong> No “magic”, “effortless” or “AI-powered”: say what it does instead.</li>
		</ul>
		<Note>
			<p>
				Words quoted from a provider keep their own voice, in mono, so it’s clear who said them: “Server overloaded, please try again later”
				is DeepSeek’s reason, not Nomi’s sentence.
			</p>
		</Note>
	</DocSection>

	<DocSection title="Best practices">
		<DoDont
			dos={[
				'Name who is working and what they are doing.',
				'Say what happened, the provider’s reason, and the next move.',
				'Say what was remembered or saved, every time.',
				'Offer what the crew can do when it can’t do what was asked.',
			]}
			donts={[
				'“Magic”, “effortless”, “AI-powered”.',
				'Exclamation marks or “oops” in place of the reason.',
				'A percentage or a progress bar when nobody knows how long it takes.',
				'A guessed cost when a call failed.',
			]}
		/>
	</DocSection>
</DocPage>
