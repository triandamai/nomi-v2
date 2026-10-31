<script lang="ts">
	import DocPage from '$lib/docs/ui/DocPage.svelte';
	import DocSection from '$lib/docs/ui/DocSection.svelte';
	import Note from '$lib/docs/ui/Note.svelte';
	import { docHref } from '$lib/docs/href';

	// Content → Vocabulary. The product's words as the pen (pen/export) and messages/en.json use
	// them. "Avoid" is filled in only where the pen or the rules say so.
	type Term = { term: string; means: string; says: string[]; avoid?: string };

	const groups: { title: string; lede?: string; terms: Term[] }[] = [
		{
			title: 'Chats and replies',
			terms: [
				{ term: 'chat', means: 'One conversation with Nomi and the crew. It can be renamed, pinned, moved into a project, shared, exported or deleted.', says: ['New chat', 'All chats', 'Delete chat', 'Share whole chat'], avoid: '“thread” in the interface: it is the design’s word in notes, not on screen.' },
				{ term: 'message', means: 'What the person sends.', says: ['Message Nomi', 'Edit message', 'Too long for one message.'] },
				{ term: 'reply', means: 'What a crew member sends back. Each reply ends with the same bar of actions.', says: ['Good reply', 'Delete reply', 'Thanks. Feedback sent with this reply only.'] },
				{ term: 'version', means: 'An edited message sends as a new version; the old reply stays.', says: ['Version 2 of 2', 'Sends as a new version. The old reply stays.'] },
				{ term: 'memory', means: 'What Nomi remembers about the person. Using or saving it is always shown, with a way to manage it.', says: ['From memory: Sari is vegetarian', 'Saved to memory: …', 'Memory & context'] },
				{ term: 'Inbox', means: 'Where results and reminders also arrive, so the person can leave the chat.', says: ['I’ll post the result here and in Inbox.', 'Kala pings you in Inbox and on your phone'] },
			],
		},
		{
			title: 'The crew',
			terms: [
				{ term: 'crew, crew member', means: 'The agents who do the work, each with a name, a colour and a mascot.', says: ['Ask Nomi, or hand something to the crew…', 'Wrong crew member', '3 crew working on this chat'], avoid: '“agent” on screen. It is the design’s and the code’s word; people read “crew” or the member’s name.' },
				{ term: 'Nomi', means: 'The product, and the crew member you talk to first, who hands work to the others.', says: ['Back to Nomi', 'Nomi handed this to Dana'] },
				{ term: 'Koda', means: 'Builds apps.', says: ['Koda is building', 'Ask Koda to fix'] },
				{ term: 'Dana', means: 'Money: transactions, budgets, subscriptions. Only records.', says: ['Dana records it when you pay.'] },
				{ term: 'Kala', means: 'Reminders on time.', says: ['When should Kala remind you about rent?'] },
				{ term: 'Maya', means: 'Reads attachments.', says: ['attachment · read by Maya'] },
				{ term: 'Tara', means: 'Acts through connected tools: Gmail, Google Drive, Google Calendar.', says: ['Tara wants to send an email'] },
				{ term: 'Supervisor', means: 'Stops the crew’s running tasks.', says: ['Supervisor stopped 3 tasks'] },
				{ term: 'Rena, Raga, Personality', means: 'Named in the pen, each with a colour and a mascot. What they do isn’t drawn in the sheets yet.', says: [] },
				{ term: 'hand off', means: 'One crew member passing a question to another, shown with the reason and a way back.', says: ['Nomi handed this to Dana', 'Ask Nomi instead'] },
			],
		},
		{
			title: 'Plans and approvals',
			terms: [
				{ term: 'plan', means: 'The crew’s steps for a job, each owned by a member. Drafted first; nothing runs until the person runs it.', says: ['Draft plan', 'Run plan', 'Ready to run this plan?'], avoid: 'Confusing it with the person’s plan (below). Say “Free plan” or “Plan & usage” for that one.' },
				{ term: 'approval', means: 'Anything that sends, spends, deletes or writes outside Nomi stops and waits. Read-only work never asks.', says: ['Needs you', 'Approve & send', 'Approved by you · sent 14:06'] },
				{ term: 'decline, reject', means: 'Saying no: Decline to an approval, Reject to a proposed code change, Don’t send to an email.', says: ['Decline', 'Reject', 'Don’t send'] },
				{ term: 'connection', means: 'A tool the person has connected (Gmail, Google Calendar, Google Drive). Nothing is created or changed without asking.', says: ['Connect Google Calendar', 'Settings › Connections'] },
			],
		},
		{
			title: 'Building',
			terms: [
				{ term: 'project', means: 'One app being built, with its own chat, preview, code and live link. Chats can also be moved into a project.', says: ['Projects', 'Project name', 'Delete project', 'Move chat to a project'] },
				{ term: 'build', means: 'Koda making the app from the plan: real steps, real files, a real clock.', says: ['Build it', 'Koda is building', '1 build · 4–7 min'] },
				{ term: 'workspace', means: 'Two meanings. The screen inside a project, chat beside preview (“Open workspace”). And a group of people who share chats and projects (“Keluarga workspace”).', says: ['Open workspace', 'Members of Keluarga will see this chat.'], avoid: 'Leaving it unclear which one you mean.' },
				{ term: 'preview', means: 'The app before it goes live, in the workspace. Changes stay here until the person publishes.', says: ['Preview', 'Review in preview', 'Everything is built in the preview first.'] },
				{ term: 'publish', means: 'The person’s action that puts the preview’s changes live.', says: ['Publish', 'Publish 2 changes', 'Changes after this need Publish'], avoid: '“Deploy” for the person’s action: the button is Publish.' },
				{ term: 'deploy', means: 'One release to the live link, numbered. Publishing makes one; a failed one leaves the last live deploy running.', says: ['deploy #2', 'live · deploy #2', 'Deploy failed'] },
				{ term: 'live, live link, live URL', means: 'The app’s public address, and the state of being there. “Live link” in sentences to people; the address itself in mono.', says: ['Live', 'a live link customers can open today', 'warung-kopi-menu-3f9a12.trian.space'] },
				{ term: 'export', means: 'Taking everything Koda built: the repo, a database dump, the names of the env vars. Never behind a plan.', says: ['Export warung-kopi-menu', 'Everything Koda built is yours. Included on Free, no fee.'] },
				{ term: 'build together, invite', means: 'Other people in a project’s chat, each with a role: Can build, Can prompt or Can view.', says: ['Invite people', 'Join and build', 'Can prompt'] },
				{ term: 'ask (in line)', means: 'A request to Koda from someone in a shared project. Asks queue, one change at a time.', says: ['2 asks in line', 'One change at a time, so edits never collide.'] },
			],
		},
		{
			title: 'Plans, credits and money',
			terms: [
				{ term: 'plan (yours)', means: 'The person’s subscription: Free or Pro, with a monthly allowance.', says: ['Free plan', 'Plan & usage', 'Upgrade to Pro'] },
				{ term: 'allowance', means: 'What a plan includes each month; builds use it.', says: ['used your free allowance', '82% of this month’s allowance used · resets 1 Nov.'] },
				{ term: 'tokens', means: 'The unit models are measured in, counted against the allowance.', says: ['21.8K / 1M tokens this month', 'deepseek-flash · 1.4s · 812 tokens'] },
				{ term: 'Nomi credits, your key', means: 'Two ways to pay for models: Nomi’s credits, or the person’s own provider key.', says: ['Use Nomi credits', 'Use my own key', 'Update key'] },
				{ term: 'cost', means: 'Always visible, never the headline, never guessed on a failure.', says: ['About 5 min · included in your free plan'] },
			],
		},
	];
</script>

<DocPage
	eyebrow="Content"
	title="Vocabulary"
	lede="The words for chats, builds, the crew and money, as the pen and the message files use them. One word for one thing, the same in every screen."
	wide
>
	{#each groups as group (group.title)}
		<DocSection title={group.title}>
			<div class="overflow-x-auto rounded-md outline -outline-offset-1 outline-border">
				<table class="w-full min-w-200 border-collapse text-left">
					<thead>
						<tr class="border-b border-border bg-surface font-mono text-label tracking-label text-text-3 uppercase">
							<th scope="col" class="w-44 px-4 py-2.5 font-normal">Term</th>
							<th scope="col" class="px-4 py-2.5 font-normal">What it means</th>
							<th scope="col" class="w-72 px-4 py-2.5 font-normal">As the product says it</th>
						</tr>
					</thead>
					<tbody>
						{#each group.terms as item (item.term)}
							<tr class="border-b border-border align-top last:border-b-0">
								<th scope="row" class="px-4 py-3 text-left text-ui font-medium text-text">{item.term}</th>
								<td class="px-4 py-3">
									<span class="flex flex-col gap-1.5">
										<span class="text-ui leading-normal text-text-2">{item.means}</span>
										{#if item.avoid}<span class="text-caption leading-normal text-danger">Avoid: <span class="text-text-2">{item.avoid}</span></span>{/if}
									</span>
								</td>
								<td class="px-4 py-3">
									<ul class="flex flex-col gap-1">
										{#each item.says as line (line)}<li class="text-caption leading-normal text-text">{line}</li>{/each}
									</ul>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</DocSection>
	{/each}

	<DocSection title="Words we don’t use">
		<ul>
			<li><strong>“Magic”, “effortless”, “AI-powered”.</strong> Say what it does: “Nomi builds real web apps from a description.”</li>
			<li><strong>“Agent” on screen.</strong> Use the member’s name, or “crew”.</li>
			<li><strong>“Deploy” as the button.</strong> The person publishes; each publish is a numbered deploy.</li>
		</ul>
		<Note>
			<p>
				A word that isn’t here yet follows the pen. If the pen uses two words for one thing, ask for one to be chosen there, then add it to
				this page. How each one is written in Bahasa Indonesia is on <a href={docHref('/design-system/content/languages')}>Languages</a>.
			</p>
		</Note>
	</DocSection>
</DocPage>
