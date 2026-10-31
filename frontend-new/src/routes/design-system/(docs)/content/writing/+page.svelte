<script lang="ts">
	import Button from '$lib/ds/actions/Button.svelte';
	import ErrorCallout from '$lib/ds/conversation/ErrorCallout.svelte';
	import DocPage from '$lib/docs/ui/DocPage.svelte';
	import DocSection from '$lib/docs/ui/DocSection.svelte';
	import DoDont from '$lib/docs/ui/DoDont.svelte';
	import Example from '$lib/docs/ui/Example.svelte';
	import Note from '$lib/docs/ui/Note.svelte';
	import { docHref } from '$lib/docs/href';

	// Content → Writing style. AGENTS.md §2 (Type, Copy, Interaction and states, Don't), §9 on
	// apostrophes, and the pen's own labels and errors (pen/export, chat--states-errors above all).
	const buttons: [kind: string, labels: string[]][] = [
		['Starting or confirming', ['Start building', 'Build it', 'Run plan', 'Approve & send', 'Save & send', 'Join and build', 'Make the plan']],
		['Changing course', ['Keep planning', 'Ask for changes', 'Review in preview', 'Choose another file']],
		['Fixing it somewhere else', ['Add credits on DeepSeek', 'Reconnect Gmail', 'Use Nomi credits', 'Update key']],
		['Destroying, said in full', ['Delete project', 'Delete reply', 'Delete chat']],
		['Declining, said plainly', ['Don’t send', 'Not now', 'Skip this step', 'Discard']],
	];

	const errors: [what: string, sentence: string, next: string][] = [
		['401 · key rejected', 'DeepSeek didn’t accept your API key, so nothing was sent. It may have been revoked or mistyped.', 'Update key · Use Nomi credits'],
		['402 · out of credits', 'Your DeepSeek account is out of credits for this request. Add credits with DeepSeek, then send again.', 'Retry · Add credits on DeepSeek'],
		['404 · model retired', 'DeepSeek no longer offers deepseek-flash-0624. Pick a replacement and I’ll use it for this chat.', 'Choose a model'],
		['429 · rate limited', 'DeepSeek is limiting requests on your key. I’ll retry on my own when the window resets.', 'Retrying in 0:24 · Try another model'],
		['503 · provider down', 'DeepSeek’s servers are overloaded. Your message is saved; retry or switch models.', 'Retry · Try Claude Sonnet'],
		['504 · timed out', 'No reply arrived within 60 seconds, so I stopped waiting.', 'Retry · Try a faster model'],
		['400 · chat too long', 'This chat is longer than deepseek-flash can read at once. I can summarise the older part and keep going.', 'Summarise & continue · Use a 128K model'],
		['413 · file too large', 'bank-statements-2026.zip is 48 MB; chats take files up to 20 MB. Split it or share a Drive link.', 'Choose another file · Share a Drive link'],
		['Offline', 'I couldn’t reach Nomi. Your message is in the outbox and sends when you reconnect.', 'Retry now'],
		['A tool signed out', 'Tara couldn’t send the email because Gmail signed Nomi out. The draft is saved.', 'Reconnect Gmail · Skip this step'],
		['A failed test', '1 test failed: no weekend pickup slots after 11:00. Nothing went live. Customers still see deploy #2.', 'Ask Koda to fix'],
	];

	const punctuation: [mark: string, use: string, example: string][] = [
		['’', 'Typographic apostrophes in every message (the pen’s are straight; the code doesn’t copy that).', 'You’ll see the plan first.'],
		['·', 'A middle dot, with a space each side, between the parts of a meta line.', 'deploy #1 · 08:18'],
		['…', 'One ellipsis character: for something in progress, and at the end of a placeholder.', 'Writing… · Tell Koda what to change…'],
		['–', 'An en dash for ranges.', '1–7 Oct · 07:00–11:00'],
		['›', 'Between the steps of a path through Settings.', 'Settings › Connections'],
		['&', 'Joins two parts of one short label or name.', 'Approve & send · Plan & usage'],
	];

	const noop = () => {};
</script>

<DocPage
	eyebrow="Content"
	title="Writing style"
	lede="Sentence case, short labels, buttons that say what they do, and errors that say what happened. The pen’s copy is the reference: it is built word for word."
	wide
>
	<DocSection title="Sentence case, everywhere">
		<p>
			Titles, buttons, menu items, labels and messages all start with a capital and continue in lower case: “Move chat to a project”, “Plan
			&amp; usage”, “Ready to run this plan?”. Names keep their own capitals: Koda, Bu Sri, Google Drive, GitHub, DeepSeek, WhatsApp.
		</p>
		<p>
			The only capitals are the small mono section labels (“NEEDS YOU”, “RECENTS”), and those are written in sentence case and set in capitals
			by the style. Title Case and ALL CAPS anywhere else are on the pen’s list of don’ts.
		</p>
	</DocSection>

	<DocSection title="Buttons say what they do" lede="A verb, and the thing it acts on when that isn’t obvious. The words on the button are the outcome.">
		<div class="overflow-x-auto rounded-md outline -outline-offset-1 outline-border">
			<table class="w-full min-w-170 border-collapse text-left">
				<tbody>
					{#each buttons as [kind, labels] (kind)}
						<tr class="border-b border-border align-top last:border-b-0">
							<th scope="row" class="w-72 px-4 py-3 text-left text-ui font-normal text-text-2">{kind}</th>
							<td class="px-4 py-3 text-ui leading-normal text-text">{labels.join(' · ')}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
		<Example title="Labels from the pen" description="Each says what happens when you press it. Which variant a button takes is on the Button page.">
			<Button label="Build it" />
			<Button variant="secondary" label="Keep planning" />
			<Button variant="danger" label="Delete project" />
		</Example>
		<ul>
			<li>While it works, a button says what is happening and for how long: “Deploying · 00:14”, “Publishing… 0:12”.</li>
			<li>A count goes in the label when it tells the person what will happen: “Publish 2 changes”, “Track 3”.</li>
			<li>A link says where it goes: “View in Gmail”, “Open in Sheets”, “Upgrade to Pro”.</li>
		</ul>
	</DocSection>

	<DocSection title="Errors: what happened, why, what next" lede="Errors say what happened in plain words, quote the provider’s own reason, and offer the next move. Never guess a cost on failure.">
		<Example title="401 · key rejected" description="The sentence, then the provider’s words in mono, then the moves." align="start">
			<div class="flex w-full max-w-160 flex-col gap-3">
				<p class="text-body leading-relaxed text-text">DeepSeek didn’t accept your API key, so nothing was sent. It may have been revoked or mistyped.</p>
				<ErrorCallout
					title="Key rejected"
					code="401 authentication_error"
					requestId="req_2b7f19aa"
					details={[
						{ key: 'reason', value: 'Authentication Fails, your api key ****a91f is invalid', strong: true },
						{ key: 'provider', value: 'api.deepseek.com' },
					]}
					actions={[
						{ label: 'Update key', onclick: noop },
						{ label: 'Use Nomi credits', onclick: noop },
					]}
				/>
			</div>
		</Example>
		<ol>
			<li><strong>What happened</strong>, and what it means for the person: “so nothing was sent”, “Your message is saved”, “Nothing went live.”</li>
			<li><strong>The provider’s reason</strong>, quoted as it came, in mono, with its code and request ID. Don’t rewrite it, even when it’s clumsy.</li>
			<li><strong>The next move</strong>, as one or two buttons. The first is the main one.</li>
		</ol>
		<div class="overflow-x-auto rounded-md outline -outline-offset-1 outline-border">
			<table class="w-full min-w-170 border-collapse text-left">
				<thead>
					<tr class="border-b border-border bg-surface font-mono text-label tracking-label text-text-3 uppercase">
						<th scope="col" class="w-40 px-4 py-2.5 font-normal">Case</th>
						<th scope="col" class="px-4 py-2.5 font-normal">What the person reads</th>
						<th scope="col" class="w-60 px-4 py-2.5 font-normal">Next move</th>
					</tr>
				</thead>
				<tbody>
					{#each errors as [what, sentence, next] (what)}
						<tr class="border-b border-border align-top last:border-b-0">
							<th scope="row" class="px-4 py-3 text-left text-caption leading-normal font-normal text-text-3">{what}</th>
							<td class="px-4 py-3 text-ui leading-normal text-text">{sentence}</td>
							<td class="px-4 py-3 text-ui leading-normal text-text-2">{next}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
		<h3>In a field</h3>
		<p>
			The error replaces the helper text under the field and says what to do: “Use lowercase and dashes, e.g. budget-keluarga.” Validation sits
			under the field, never in a toast.
		</p>
		<Note>
			<p>Cost is never guessed: providers don’t report a charge on failed calls, so a failure never shows a guessed charge.</p>
		</Note>
	</DocSection>

	<DocSection title="Keep it short">
		<ul>
			<li>One idea per sentence. A helper line is one line: “Lowercase, numbers and dashes. Becomes your URL.”</li>
			<li>Meta lines are fragments joined with a middle dot, not sentences: “Sari · next”, “Built in 5m 40s · used your free allowance”.</li>
			<li>Say where something lives instead of explaining it: “Change it in Settings › Profile.”</li>
			<li>Say what stays safe when something is about to change: “Your memory and money records stay personal.”, “Things the crew already did, like a sent email, are not undone.”</li>
		</ul>
	</DocSection>

	<DocSection title="Punctuation and symbols">
		<div class="overflow-x-auto rounded-md outline -outline-offset-1 outline-border">
			<table class="w-full min-w-170 border-collapse text-left">
				<thead>
					<tr class="border-b border-border bg-surface font-mono text-label tracking-label text-text-3 uppercase">
						<th scope="col" class="w-20 px-4 py-2.5 font-normal">Mark</th>
						<th scope="col" class="px-4 py-2.5 font-normal">Use</th>
						<th scope="col" class="w-72 px-4 py-2.5 font-normal">Example</th>
					</tr>
				</thead>
				<tbody>
					{#each punctuation as [mark, use, example] (mark)}
						<tr class="border-b border-border align-top last:border-b-0">
							<th scope="row" class="px-4 py-3 text-left text-heading font-normal text-text">{mark}</th>
							<td class="px-4 py-3 text-ui leading-normal text-text-2">{use}</td>
							<td class="px-4 py-3 text-ui leading-normal text-text">{example}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
		<p>
			Spelling is British, as the pen and the code write it: colour, behaviour, summarise. Numbers, money, times and IDs have their own page:
			<a href={docHref('/design-system/content/formatting')}>Numbers, time &amp; money</a>.
		</p>
	</DocSection>

	<DocSection title="Best practices">
		<DoDont
			dos={[
				'Sentence case for every title, label and button.',
				'Start a button with a verb; name the object when it isn’t obvious.',
				'Quote the provider’s reason as it came.',
				'Offer the next move with every error.',
			]}
			donts={[
				'Title Case or ALL CAPS outside the mono label.',
				'“Magic”, “effortless” or “AI-powered”.',
				'Emoji in UI chrome.',
				'An error that only says something went wrong.',
			]}
		/>
	</DocSection>
</DocPage>
