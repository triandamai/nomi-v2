import type { ComponentDoc } from '../types';

export default [
	{
		file: 'conversation/ActivityBlock.svelte',
		summary:
			'What an agent did, summed up in one line ("Wrote 3 files · +424") with the real files under it. A mint check when done, a spinner while working, an alert icon if it failed.',
		use: ['In an agent\'s message, after it has changed files.'],
		avoid: ['The one step happening right now: use LiveStep.', 'Something that happened to the project, like a deploy: use EventRow.'],
		dos: ['Say what happened in the title, in the past tense once done: "Wrote 3 files".', 'List the real paths, each with its change.', 'Give the total in mono: "+424".'],
		donts: ['A summary with no files under it, like "Made some changes".'],
		content: ['Title: a verb and a count, sentence case: "Wrote 3 files".', 'Paths relative to the project, exactly as they are.'],
		accessibility: [
			'The state icon has no label, so the title has to say the state in words.',
			'The spinner stops when people ask for reduced motion.',
		],
		related: ['conversation/FileRow.svelte', 'conversation/LiveStep.svelte', 'build/FileList.svelte'],
	},
	{
		file: 'conversation/ErrorCallout.svelte',
		summary:
			'A failure in a conversation, named plainly, with the provider\'s own words in mono (status code, request id, reason) and the next move one click away.',
		use: ['A provider refusing a request, like "402 insufficient_balance".', 'Any failure tied to one message that people can act on.'],
		avoid: [
			'Something about what\'s around the chat (connection, plan, key) rather than one message: use Banner.',
			'News about something that just finished elsewhere: use Toast.',
			'A field people filled in wrong: use the TextField error.',
		],
		dos: [
			'Quote the provider\'s reason exactly as it came: "Insufficient Balance".',
			'Show the request id, so support can find it.',
			'Make the first action the main one (inverse); any other is secondary.',
			'Mark links to the provider\'s site external.',
		],
		donts: ['Guess a cost on failure.', '"Something went wrong" with no reason and no next move.'],
		content: [
			'The title names the kind of failure: "Provider error".',
			'The message above it says in plain words what happened and what to do next.',
			'Actions are verbs: "Retry", "Add credits on DeepSeek".',
		],
		accessibility: [
			'role="alert", so screen readers announce it when it appears.',
			'The details are a description list: each key is read with its value.',
			'Actions are real buttons or links; external links open in a new tab.',
		],
		related: ['chat/Banner.svelte', 'overlays/Toast.svelte', 'conversation/MessageAgent.svelte'],
	},
	{
		file: 'conversation/EventRow.svelte',
		summary:
			'Something that happened to a project, logged in one mono line with its time: a deploy, a rollback, a domain or a database change ("deploy #14 · main@a3f9c21 · built in 41s").',
		use: ['A project\'s history.', 'In a conversation, right after a deploy.'],
		avoid: ['Work still happening: use LiveStep.', 'Which files changed: use ActivityBlock.', 'A fact that holds, not an event: use KVRow.'],
		dos: ['Use the real numbers: the deploy number, branch@commit, how long it took.', 'Give the time in 24-hour form: "15:38".'],
		donts: ['Round or hide figures to make it read friendlier.'],
		content: ['Lower case, in mono, parts joined with " · ".'],
		accessibility: ['The icon is decorative. Long text is cut off on one line only visually; screen readers hear all of it.'],
		related: ['conversation/KVRow.svelte', 'conversation/ActivityBlock.svelte'],
	},
	{
		file: 'conversation/FileRow.svelte',
		summary:
			'A file an agent touched, in mono, with its change ("+214"). While a build writes files it also shows where each one is: written (a mint check), being written ("writing"), or still to come (faded).',
		use: ['Inside ActivityBlock.', 'In a build\'s list of files as they are written.'],
		avoid: ['A file someone attached to a message: MessageUser shows those.'],
		dos: ['Use the real path, relative to the project.', 'Set state only while a build is writing files.'],
		content: ['Paths exactly as they are: "src/routes/+page.svelte". Changes as "+214".'],
		accessibility: ['Icons are decorative; "writing" is said in words.', 'The writing spinner stops when people ask for reduced motion.'],
		related: ['conversation/ActivityBlock.svelte', 'build/FileList.svelte'],
	},
	{
		file: 'conversation/KVRow.svelte',
		summary: 'A plain key with a machine value in mono, on one line: "Code   23 files · main@a3f9c21".',
		use: ['Facts about a project in a panel or card.'],
		avoid: ['Something that happened at a time: use EventRow.'],
		dos: ['Keep keys to one short word: the key column is 72px wide.', 'Give the value exactly as the machine reports it.'],
		content: ['Key in sentence case ("Code"); value in mono.'],
		accessibility: ['Plain text; the icon is decorative. A long value is cut off only visually.'],
		related: ['conversation/EventRow.svelte'],
	},
	{
		file: 'conversation/LiveStep.svelte',
		summary:
			'The step an agent is on right now, as it really is ("Writing src/routes/menu/+page.svelte"), with the time so far and an optional Stop. No progress bar when nobody knows how long it takes.',
		use: ['While an agent works, such as during a build.'],
		avoid: ['Work that is finished: use ActivityBlock.', 'Who is working, on a card or row: use Status (working).'],
		dos: ['Name the real tool and target.', 'Show the time so far in mono: "01:12".', 'Offer Stop when the work can be stopped.'],
		donts: ['A progress bar that creeps toward 90%.', 'Vague words like "Working on it…".'],
		content: ['A verb in -ing and the real target: "Writing src/routes/menu/+page.svelte".', 'The stop button says "Stop".'],
		accessibility: [
			'The step text is role="status", so a new step is announced.',
			'The spinner is decorative and stops when people ask for reduced motion.',
			'Stop is a ghost button with an icon and a word.',
		],
		related: ['conversation/ActivityBlock.svelte', 'build/BuildRunning.svelte', 'status/Status.svelte'],
	},
	{
		file: 'conversation/MessageAgent.svelte',
		summary:
			'An answer from a crew member: who gave it and when ("14:02 · deepseek-flash"), then the prose. Rich blocks such as tables, plans or sources come as children, only when the answer has that shape.',
		use: ['Every reply from Nomi or the crew.'],
		avoid: ['What the person wrote: use MessageUser.'],
		dos: ['Lead with prose; add rich blocks only when the answer is a table, a plan or has sources.', 'Show the time and the model in the meta.'],
		donts: ['Put the reply in a bubble: only the person\'s messages are bubbles.'],
		content: ['Plain sentences that say what happened and what to do next.'],
		accessibility: ['An <article>. Line breaks in the text are kept.', 'The byline\'s mascot is decorative; the name says who answered.'],
		related: ['conversation/MessageUser.svelte', 'identity/AgentByline.svelte', 'chat/ReplyText.svelte'],
	},
	{
		file: 'conversation/MessageUser.svelte',
		summary:
			'What the person said: on the right in a surface-2 bubble, with any files above it. Until it is sent it is faded and says where it is; once sent, hovering shows copy, edit and the time.',
		use: ['Every message a person sends.'],
		avoid: ['A reply from the crew: use MessageAgent.', 'A teammate\'s message in a shared project: use TeammateMessage.'],
		dos: [
			'Show the real state: "sending…", "queued · sends when you\'re back online", or "not sent" with Retry and Edit.',
			'Show "edited" with the time, and the version switcher, after an edit.',
		],
		donts: ['Hide a message that failed to send.'],
		content: ['The text exactly as the person wrote it.'],
		accessibility: [
			'Copy and edit are labelled buttons ("Copy message", "Edit message"); copy says "Copied" once done.',
			'The hover actions also show when anything in the message has keyboard focus, and always on touch screens.',
			'Attached files are a list labelled "Attached files".',
			'The sending spinner stops when people ask for reduced motion.',
		],
		related: ['conversation/MessageAgent.svelte', 'chat/MessageEditor.svelte', 'chat/VersionSwitcher.svelte', 'mobile/UserMessage.svelte'],
	},
] satisfies ComponentDoc[];
