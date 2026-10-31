import type { PatternDoc } from '../types';

// The Chat responses sheet, section by section: what each pattern does for the person and the
// rules for it, drawn from the sheet's own notes (pen/export/chat--*.html) and the components that
// build it. Quoted copy is the pen's.

export default [
	{
		sheet: 'chat',
		id: 'thinking-planning',
		summary:
			'How the crew shows its work: the reasoning behind an answer, the steps it took and the plan it will run. Live while it runs, collapsed by default once the answer lands, and never theatrical.',
		use: [
			'The model is reasoning before it answers (“Thinking 00:04”).',
			'An answer took real steps worth showing, such as reading 142 transactions and comparing two months.',
			'A job needs several crew members, so Nomi lays out a plan with an owner for each step.',
			'Nomi proposes a plan for you to edit before anything runs (Draft plan).',
		],
		dos: [
			'Show reasoning live with its elapsed time, then fold it to one line (“Thought for 8s”) once the answer lands. It opens on request.',
			'Give each step its real tool, size and time, in mono: “dana.records · 142 rows · 1.1s”.',
			'Give each plan step to the crew member who owns it, with its real state: what it produced, how long it has been running, or what it waits for.',
			'Say how far a running plan has got in words (“2 of 4 done”) and keep Stop plan in reach.',
			'Let people reorder, reassign or remove steps in a draft plan, and add their own.',
		],
		donts: [
			'Don’t leave the reasoning open after the answer arrives.',
			'Don’t dramatise the thinking or show steps the crew didn’t take.',
			'Don’t start a draft plan’s steps before the person presses Run plan.',
		],
		content: [
			'Label reasoning by its state and time: “Thinking”, “Thought for 8s”, “Worked through 4 steps · 12s”.',
			'Write steps as short verb phrases: “Pull October transactions”, “Draft a weekly food budget”.',
			'Put machine detail in the step’s meta, in mono: “food +48% vs September”, “running · 00:06”, “waits for step 3”.',
			'A one-line thought states a decision and its reason: “You asked about this month only, so I’ll leave last year’s data out.”',
			'A draft plan says plainly that nothing runs yet: “Reorder, reassign or remove steps; nothing runs until you say so.”',
		],
	},
	{
		sheet: 'chat',
		id: 'rich-content',
		summary:
			'Replies are prose first. Structure appears only when the content has it: tables for comparable rows, cards for things you act on, code and diffs in mono, and numbered sources right after the claims they back.',
		use: [
			'An answer compares the same figures across rows, such as food spending by week.',
			'The crew has prepared something you act on yourself, such as a grocery cart to check out on Sayurbox.',
			'Koda shows code, or wants to change a file.',
			'An answer leans on a document or a web page the reader should be able to check.',
			'A message mentions a crew member or a file, or sets a reminder.',
		],
		dos: [
			'Use a table only for comparable rows: the first column names the row, numbers sit in mono and right-aligned, and where the data came from sits underneath.',
			'Show a cart’s real prices with the time they were read, say plainly what’s out of stock, and offer a swap when there is one.',
			'Number citations right after the claim they back, and list each source with where it came from: an attachment and who read it, or a site’s domain.',
			'Show a diff with old and new line numbers and its +/− counts, and apply it only when the person approves.',
			'Keep crew tints on crew-owned bits only: an @mention, Kala’s reminder tile.',
		],
		donts: [
			'Don’t turn prose into a table or cards when the content has no structure.',
			'Don’t check out for the person. They pay at the store; Nomi only records it.',
			'Don’t tint file mentions. Files aren’t crew, so they stay neutral and mono.',
			'Don’t copy line numbers with the code: Copy takes the code alone.',
		],
		content: [
			'Title a table or card by what it holds: “Food spending by week”, “Weekly groceries”.',
			'Amounts are mono with Indonesian separators (“1.173.000”). When a table leaves out “Rp”, its source line names the unit: “Rp · from Dana · 142 transactions”.',
			'Name what an action does and where: “Open in Sheets”, “Checkout on Sayurbox”, “Apply change”.',
			'A diff says who wants what: “Koda wants to change 1 file”.',
			'A due reminder offers the next move in the person’s terms: “Snooze 1h”, “Mark paid”.',
		],
	},
	{
		sheet: 'chat',
		id: 'actions',
		summary:
			'Anything that sends, spends, deletes or decides for you stops here and waits. Each card has one clear primary, and once you act it collapses to a one-line outcome that stays in the thread.',
		use: [
			'A crew member is about to send, spend, delete or write outside Nomi (Approval).',
			'Nomi needs one choice with real tradeoffs (Decision), or several picks from a list (Selection).',
			'Nomi needs details it can mostly fill in from the chat, such as a reminder’s date and time (Form).',
			'Nomi needs answers before it can plan: a few quick questions, one at a time, then a review and how the plan may run.',
			'An answer invites a next question (Follow-ups), or Nomi has just asked something with short answers (Quick replies).',
		],
		dos: [
			'Show exactly what will go out before approval: who it goes to, who it’s from, the subject, the message and what’s attached.',
			'Write out the tradeoff for every option in a Decision and start with the crew’s suggestion picked. Keep a running count and total in a Selection.',
			'Prefill forms from the chat, and put validation under the field it belongs to.',
			'Ask clarifying questions one at a time, and mark an answer the crew filled in for you as Default, with the reason.',
			'Collapse the card to its outcome once the person acts: “Approved by you · sent 14:06”.',
		],
		donts: [
			'Don’t ask for approval on read-only work.',
			'Don’t put more than one primary button on a card.',
			'Don’t show form validation in a toast.',
			'Don’t run a plan straight from the answers: “Nothing runs yet. You’ll see the plan first.”',
			'Don’t use Quick replies for optional next questions; those are Follow-ups. Quick replies answer the last question directly.',
		],
		content: [
			'Approval titles say who wants to do what: “Tara wants to send an email”.',
			'Primary buttons name the outcome: “Approve & send”, “Use 3 days before”, “Track 3”, “Save reminder”, “Run plan”.',
			'Declining says what won’t happen: “Don’t send”. The outcome confirms it: “You chose not to send · nothing left your Gmail”.',
			'Keyboard hints are short and mono: “1–4 to pick · ↵ next · esc skip all”.',
		],
	},
	{
		sheet: 'chat',
		id: 'response-feedback',
		summary:
			'Every reply ends with the same quiet bar: like, dislike, share and delete, with the model, time and tokens beside them. Like is one tap; dislike asks why; share masks your numbers by default; delete confirms and can be undone.',
		use: [
			'Someone wants to rate a reply, or say what was wrong with it.',
			'Someone shares one reply or the whole chat as a link.',
			'Someone deletes a reply and may want it back.',
			'A reply missed and the person wants another try, perhaps with another model.',
		],
		dos: [
			'Keep what answered visible on the bar: “deepseek-flash · 1.4s · 812 tokens”.',
			'Ask for reasons first on a dislike (any number of them), then an optional note.',
			'Send only this reply and the note with feedback; the whole chat goes only if the person turns it on, and amounts and names stay masked either way.',
			'Mask amounts and names in shares by default (“Rp •••” and initials), and say who can open the link and when it expires.',
			'Confirm a delete by saying what leaves, what stays and what can’t be undone, then offer Undo.',
		],
		donts: [
			'Don’t ask why on a like.',
			'Don’t suggest a delete undoes what the crew already did, like a sent email.',
			'Don’t hand out a share link without a way to revoke it.',
		],
		content: [
			'Say exactly what was sent: “Thanks. Feedback sent with this reply only.”',
			'Dislike reasons are short and plain: “Not accurate”, “Didn’t do what I asked”, “Wrong crew member”.',
			'Helper text states the default: “Off by default. Only this reply and your note are sent; amounts and names stay masked.”',
			'The delete dialog names what isn’t undone: “Things the crew already did, like a sent email, are not undone.”',
			'Toasts say what happened and offer one action: “Reply deleted · Undo within 10 seconds”, “Couldn’t create the link · You’re offline. Try again when you reconnect.”',
		],
	},
	{
		sheet: 'chat',
		id: 'states-errors',
		summary:
			'What a message is doing, and what went wrong when it fails. Errors say what happened in plain words, quote the provider’s own reason and offer the next move. Cost is never guessed: providers don’t report a charge on failed calls.',
		use: [
			'A message is sending, waiting in the outbox while offline, or wasn’t sent.',
			'A reply was stopped by you or cut off by the provider.',
			'Something around the chat needs attention: the connection, a slow provider, a key about to expire, the month’s allowance.',
			'A provider call fails: a rejected key, no credits, a retired model, a rate limit, an outage, a timeout, a chat too long, a blocked reply, a file too large.',
			'A tool fails mid-task, or the month’s Nomi credits run out.',
		],
		dos: [
			'Lead with one plain sentence on what happened and what it means: “DeepSeek didn’t accept your API key, so nothing was sent.”',
			'Quote the provider’s code, request id and reason in mono: “402 insufficient_balance”, “req_9d41e0b2”, “Insufficient Balance”.',
			'Make the fix the first action, with an alternative beside it: “Update key” and “Use Nomi credits”.',
			'Say what is safe: “Your message is saved”, “The draft is saved”, “Your chats still work”.',
			'Show banners above the thread, one at a time.',
		],
		donts: [
			'Don’t guess a cost on a failed call.',
			'Don’t replace the provider’s reason with your own words; quote it.',
			'Don’t drop what the person wrote: a queued or failed message keeps its text, with Cancel, Retry or Edit.',
			'Don’t leave a stopped or cut-off reply at a dead end: offer Continue or Start over.',
		],
		content: [
			'Name the provider and the model: “DeepSeek no longer offers deepseek-flash-0624.”',
			'Say the case in the person’s terms, then the fix: “This chat is longer than deepseek-flash can read at once. I can summarise the older part and keep going.”',
			'Message states are short lower-case meta: “sending…”, “queued · sends when you’re back online”, “not sent”, “stopped by you · 00:03”.',
			'Buttons name the fix: “Update key”, “Add credits on DeepSeek”, “Summarise & continue”, “Reconnect Gmail”.',
			'Limits give both numbers: “bank-statements-2026.zip is 48 MB; chats take files up to 20 MB.”',
		],
	},
	{
		sheet: 'chat',
		id: 'message-controls-composer',
		summary:
			'Every message can be stopped, edited, regenerated or copied. Edits branch instead of overwriting, and attachments and voice show progress and limits before you send.',
		use: [
			'Someone wants to fix a question they’ve already sent.',
			'A reply should be tried again: the same, shorter, with more detail, as a list, or with another model.',
			'A reply is streaming and the person wants it to stop.',
			'Someone attaches a file, a photo, a Drive file or a link, or records a voice message.',
			'A message is over the length limit.',
		],
		dos: [
			'Send an edit as a new version, and keep the old one and its reply reachable from the version switcher (“2 / 2”).',
			'Show what each model in the regenerate menu is paid with: “your key”, “Nomi credits”.',
			'Show upload progress in real bytes (“62% · 3,1 of 5 MB”) and the limits in the attach menu (“PDF, images, sheets, zip · max 20 MB”).',
			'Say what was copied: “Copied as Markdown”.',
			'Show the language a recording transcribes in, and let the text be edited before it’s sent.',
		],
		donts: [
			'Don’t overwrite a message when it’s edited.',
			'Don’t estimate upload progress; count what has arrived.',
			'Don’t let an over-long message fail on send. Say how far over it is and offer the way round: “Send as a file instead”.',
		],
		content: [
			'Edit hints say what will happen: “Sends as a new version. The old reply stays in 1 / 2.”',
			'Mark an edited message with when: “edited · 14:05”.',
			'Limits give the count and the way round: “12.480 / 10.000”, “Too long for one message.”, “Send as a file instead”.',
			'Shortcuts sit beside their action, in mono: “Try again ⌘R”, “Stop reply esc”.',
		],
	},
	{
		sheet: 'chat',
		id: 'chat-management-long-threads',
		summary:
			'The chat itself can be renamed, pinned, moved into a project, exported or deleted. Long threads keep your place and tell you when something new arrives.',
		use: [
			'Someone wants to rename, pin, export, share or delete a chat from the top bar’s ··· menu.',
			'A chat belongs in a project, possibly in a shared workspace.',
			'New replies arrive while someone is reading further up.',
			'Someone comes back to a chat that moved on while they were away.',
			'A long thread loads earlier messages, or crosses into another day.',
		],
		dos: [
			'Set Delete chat apart from the other menu items, in danger.',
			'Rename in place in the top bar: Enter saves, Escape cancels.',
			'Say who will see a chat moved into a shared workspace, and what stays personal.',
			'Count what came in: “2 new replies” above the composer, “2 replies while you were away” where you left off.',
			'Divide days quietly: “Yesterday”, “Mon 27 Oct”.',
		],
		donts: [
			'Don’t save an empty chat name.',
			'Don’t move the reader when new replies arrive; offer Jump to latest instead.',
			'Don’t reuse the general search placeholder in the move dialog. It searches projects: “Search projects”.',
		],
		content: [
			'Menu items are short verbs, with the shortcut or formats in mono: “Rename F2”, “Export md · pdf”.',
			'The move dialog states the consequence: “Members of Keluarga will see this chat. Your memory and money records stay personal.”',
			'Inline hints pair keys with verbs: “↵ save · esc cancel”.',
			'Loading says what is loading: “Loading older messages…”.',
		],
	},
	{
		sheet: 'chat',
		id: 'crew-behaviour',
		summary:
			'How the crew behaves around an answer. Hand-offs are visible, limits are explained with an alternative, memory is never silent, and long work can run without you watching.',
		use: [
			'Nomi passes a question to the crew member who has what it needs.',
			'Someone asks for something the crew won’t do, such as cancelling a subscription.',
			'An answer uses something remembered, or the crew saves something new to memory.',
			'A crew member needs a tool connected before it can go on.',
			'Work runs in the background, the supervisor stops tasks, or the session ends mid-chat.',
		],
		dos: [
			'Say who answers and why (“Nomi handed this to Dana · she has your transaction records”), with a way to ask Nomi instead.',
			'When the crew can’t do something, give the reason and what it can do instead, each with the crew member who’d do it.',
			'Show memory where it’s used or saved, with Manage, Edit and Undo right there.',
			'When asking to connect a tool, say who needs it and why, exactly what it can read, that it changes nothing without asking, and offer a way on without it.',
			'When tasks are stopped, list each one’s state: done before the stop, stopped part-way and how far it got, or not started.',
		],
		donts: [
			'Don’t use or save memory silently.',
			'Don’t rewrite a person’s edits to a draft: “Your edits are sent as written. Tara won’t rewrite them.”',
			'Don’t claim a stop undid what already left Nomi.',
			'Don’t lose an unsent message when the session ends. Keep it on the device and send it after sign-in.',
		],
		content: [
			'Limits start with what the crew can’t do, then why: “I can’t cancel subscriptions for you”, “It never signs in to Netflix or your bank, so cancelling stays with you.”',
			'Memory lines start with where it came from: “From memory: Sari is vegetarian”, “Saved to memory: you and Sari prefer trains to driving”.',
			'Background notes say you can leave and where the result goes: “You can leave this chat. I’ll post the result here and in Inbox.”',
			'Mark the crew’s own estimates as rough: “~3 min”.',
		],
	},
	{
		sheet: 'chat',
		id: 'mobile-chat',
		summary:
			'The chat at 390 pt, on native patterns: sheets instead of popovers, 44 pt targets, 16 pt body text, the composer above the keyboard, banners at the top. Desktop cards are reused only where they fit without losing anything.',
		use: [
			'Someone chats with the crew on a phone.',
			'A reply’s actions, regenerate or delete, happen on a touch screen.',
			'An approval, an option or an error has to fit one column.',
			'The crew needs the person while the app is closed (a notification).',
			'Someone browses their chats on a phone.',
		],
		dos: [
			'Use bottom sheets instead of popovers and menus.',
			'Give every target at least 44 pt, and make buttons full width.',
			'Set body text at 16 pt, so iOS doesn’t zoom, and keep the composer above the keyboard.',
			'Turn a wide table into a stat list: each row’s label, total and change on one line, with the breakdown underneath.',
			'Put toasts and banners at the top of the screen.',
		],
		donts: [
			'Don’t squeeze a desktop card onto a phone if it loses anything; use its phone version.',
			'Don’t drop the provider’s words from an error on a phone: quote them in mono, with the model, key and request.',
			'Don’t show Send before there’s text. While a reply streams, Stop takes its place.',
		],
		content: [
			'Keep titles short enough for the nav bar: “Food spending in October”.',
			'Notifications say who needs what, and how far along it is: “Bandung plan: Tara needs your OK”, “4 of 5 done. Share the itinerary with Sari?”',
			'Shorten amounts where space is tight: “480rb”, “Rp 2,1 jt”.',
			'Error cards say whose words they quote: “DeepSeek said”, then “Insufficient Balance”.',
		],
	},
] satisfies PatternDoc[];
