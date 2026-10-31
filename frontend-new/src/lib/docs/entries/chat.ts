import type { ComponentDoc } from "../types";

// Chat responses: everything a reply can hold, and the controls around it. Written from each
// component's own comments, the Chat responses sheet's board notes and the gallery's copy.

export default [
  {
    file: "chat/ApprovalCard.svelte",
    summary:
      "The stop before any send, spend, delete or external write. It shows exactly what will go out (fields, the message, what is attached) and waits; once you act, it folds to a one-line outcome that stays in the thread.",
    use: [
      "A crew member is about to send, spend, delete or write somewhere outside Nomi.",
      "The person should see the exact draft before it leaves, and be able to edit it.",
    ],
    avoid: [
      "Read-only work, such as reading records or searching: it never asks.",
      "Picking one of several options: use DecisionCard.",
      "On a phone: use the mobile ApprovalCard.",
    ],
    dos: [
      "Show everything that will go out: who it goes to, who it comes from, the subject, the message and the attachment.",
      'Name the action on both buttons: "Approve & send" and "Don\'t send".',
      "After the person acts, switch to approved or rejected and keep the outcome with where to see it or how to undo it.",
      'Offer a remember choice only for one named recipient: "Don\'t ask again for Bu Sri".',
    ],
    donts: [
      "Summarise the draft instead of showing it.",
      "Change the person's edits: in editing, their words go out exactly as written.",
      "Leave a pending card in the thread after the person has acted.",
    ],
    content: [
      'The title says who wants to do what: "Tara wants to send an email". After acting it says what happened: "Tara sent the email to Bu Sri".',
      'The outcome says what happened and what didn\'t: "Approved by you · sent 14:06", "You chose not to send · nothing left your Gmail".',
      "Field keys are short, lower case and in mono: to, from, subject.",
    ],
    accessibility: [
      "While editing, the text field is named by the card's title.",
      "The remember choice is a real checkbox inside its label.",
      "An outcome link to another site opens in a new tab.",
    ],
    related: [
      "mobile/ApprovalCard.svelte",
      "chat/Diff.svelte",
      "chat/DecisionCard.svelte",
      "chat/PlanApproval.svelte",
    ],
  },
  {
    file: "chat/AttachMenu.svelte",
    summary:
      "The menu off the composer's paperclip: upload a file, a photo or screenshot, a Google Drive file (through the crew member who has Drive) or a link, with the limits underneath.",
    use: ["Behind the attach button in the composer."],
    avoid: [
      "Showing a file that is already attached: use AttachmentChip or AttachmentImage.",
    ],
    dos: [
      "Pass driveBy only when a crew member has Google Drive: without it, Drive isn't offered.",
      'Pass the real largest file a chat takes, e.g. "20 MB"; it shows in the footer.',
    ],
    accessibility: [
      "Built on Menu: the arrows and Home/End move, Enter or Space picks, Escape closes.",
    ],
    related: [
      "inputs/Composer.svelte",
      "chat/AttachmentChip.svelte",
      "chat/AttachmentImage.svelte",
      "overlays/Menu.svelte",
    ],
  },
  {
    file: "chat/AttachmentChip.svelte",
    summary:
      "A file attached in the composer, before you send. While it uploads it shows the real progress (bytes in, never a guess); once ready it shows the type and size.",
    use: [
      "A document is uploading or attached to a message that hasn't been sent yet.",
    ],
    avoid: [
      "Pictures: use AttachmentImage.",
      "Files in a message that was sent: MessageUser shows them.",
    ],
    dos: [
      'Write the detail from real numbers: "62% · 3,1 of 5 MB" while uploading, "PDF · 4,2 MB" when ready.',
      "Drop progress once the upload is done, so the chip shows as ready.",
      "Pass onremove so the upload can be cancelled or the file removed.",
    ],
    donts: ["Estimate or animate progress: it is bytes in, never a guess."],
    content: [
      "Use the file's own name; it shows in mono and truncates if it's long.",
    ],
    accessibility: [
      "While uploading, the bar is a progressbar named after the file, from 0 to 100.",
      'The remove button says what it does: "Cancel upload of …" while uploading, "Remove …" after.',
      "On touch screens the remove button is a 44px target.",
    ],
    related: [
      "chat/AttachmentImage.svelte",
      "chat/AttachMenu.svelte",
      "chat/ComposerLimit.svelte",
      "inputs/Composer.svelte",
    ],
  },
  {
    file: "chat/AttachmentImage.svelte",
    summary:
      "A picture attached in the composer, as a 44px square. Its remove button shows on hover or focus, and always on touch screens.",
    use: [
      "A photo or screenshot is attached to a message that hasn't been sent yet.",
    ],
    avoid: ["Documents: use AttachmentChip."],
    dos: [
      'Write alt text that says what the picture is, e.g. "Receipt photo": it also names the remove button.',
    ],
    accessibility: [
      'The remove button is named "Remove" plus the alt text, and appears on keyboard focus as well as hover.',
    ],
    related: ["chat/AttachmentChip.svelte", "chat/AttachMenu.svelte"],
  },
  {
    file: "chat/BackgroundNote.svelte",
    summary:
      "Says long work keeps running without you watching: you can leave the chat, and the result comes back here and to Inbox. The estimate is the crew's own, marked as rough with ~.",
    use: [
      "Work that takes minutes, where the person shouldn't have to wait in the chat.",
    ],
    avoid: [
      "Showing who is working right now, with a way to stop them: use CrewWorking.",
    ],
    dos: [
      'Mark the estimate as rough: "~3 min".',
      "Leave the estimate out when there isn't one.",
    ],
    donts: [
      "Show a progress bar or a countdown for work whose length nobody knows.",
    ],
    accessibility: [
      "It is a status, so screen readers announce it.",
      "The spinner stops when reduced motion is on.",
    ],
    related: [
      "chat/CrewWorking.svelte",
      "chat/PlanCard.svelte",
      "chat/SupervisorStopped.svelte",
    ],
  },
  {
    file: "chat/Banner.svelte",
    summary:
      "One line above the thread about what is around the chat (the connection, the provider, your key, your plan) rather than one message. Offline is a warning, back online is good news, and the rest sit on a quiet bordered surface.",
    use: [
      "The connection drops or comes back.",
      "The provider is slow, a key is about to expire, or the plan's allowance is running out.",
    ],
    avoid: [
      "A failure of one reply: use ErrorCallout in the thread.",
      "Who is working on the chat: use CrewWorking.",
    ],
    dos: [
      "Show one at a time, above the thread.",
      'Say what still works and what happens next: "New messages wait in the outbox and send when you reconnect."',
      "Give it one action when there is a next move.",
    ],
    donts: [
      "Stack several banners.",
      "Use it for news about a single message.",
    ],
    content: [
      'Plain and specific, with the real numbers: "DeepSeek is slow right now (replies ~14s). Your chats still work."',
      'The action names the move: "Retry now", "Update key", "See usage".',
    ],
    accessibility: [
      "It is a status, so screen readers announce it.",
      "An external action opens in a new tab.",
      "An action button is 44px tall on touch screens.",
    ],
    related: [
      "chat/CrewWorking.svelte",
      "conversation/ErrorCallout.svelte",
      "overlays/Toast.svelte",
    ],
  },
  {
    file: "chat/Boundary.svelte",
    summary:
      "Something the crew won't do, said plainly with the reason, and what it can do instead, each alternative owned by the crew member who would do it.",
    use: [
      "A request is outside what Nomi does, e.g. cancelling a subscription it never signs in to.",
    ],
    avoid: [
      "Something that failed: use ErrorCallout.",
      "A tool that needs connecting first: use ConnectPrompt.",
    ],
    dos: [
      "Say what won't happen in the title: \"I can't cancel subscriptions for you\".",
      'Give the real reason, then lead into the alternatives: "Here\'s what I can do instead:".',
      'Give each alternative who does it and when or where: "Kala · 2 Nov, 09:00".',
    ],
    donts: ["Say no without an alternative: limits are explained with one."],
    content: [
      'Alternatives are actions in plain words: "Remind me 3 days before the next charge", "Open Netflix\'s cancel page".',
    ],
    accessibility: [
      "Each alternative is a real link or button.",
      "An alternative on another site opens in a new tab and shows an arrow.",
    ],
    related: [
      "chat/ConnectPrompt.svelte",
      "chat/Handoff.svelte",
      "conversation/ErrorCallout.svelte",
    ],
  },
  {
    file: "chat/Cart.svelte",
    summary:
      "A basket priced at a real store. You check out at the store; Nomi only records it. Prices carry the time they were read, what is out of stock is said plainly, and a swap is offered when there is one.",
    use: ["The crew has put together items to buy from a named store."],
    avoid: [
      "Comparing numbers across rows: use DataTable.",
      "Picking which of several things to keep: use SelectionCard.",
    ],
    dos: [
      'Say where and when prices were read: "Prices from Sayurbox at 14:05."',
      'Mark what the store doesn\'t have and offer a swap: "Swap for tongkol".',
      'Format prices before passing them; use "—" for what can\'t be bought.',
    ],
    donts: ["Check out inside Nomi: the button opens the store."],
    content: [
      'The note says what happens next: "Dana records it when you pay."',
    ],
    accessibility: [
      'Each quantity is a named group ("Quantity of …") with "Fewer …" and "More …" buttons.',
      "Quantity buttons are disabled for items that are out of stock.",
      "Checkout opens the store in a new tab.",
    ],
    related: ["chat/DataTable.svelte", "chat/SelectionCard.svelte"],
  },
  {
    file: "chat/ChatMenu.svelte",
    summary:
      'The menu off the top bar\'s "···": everything you can do to the chat itself. Rename, pin, move to a project, export, share and delete, with delete set apart in danger.',
    use: ["The chat's own options, in the top bar."],
    avoid: ["Actions on one reply: use ResponseActions."],
    dos: [
      'Pass pinned, so the row reads "Unpin from sidebar" once the chat is pinned.',
    ],
    accessibility: [
      "Built on Menu: the arrows and Home/End move, Enter or Space picks, Escape closes.",
    ],
    related: [
      "chat/RenameField.svelte",
      "chat/MoveDialog.svelte",
      "chat/ShareDialog.svelte",
      "overlays/Menu.svelte",
    ],
  },
  {
    file: "chat/ChoiceOption.svelte",
    summary:
      "One answer in a Decision, Clarify or Plan Approval card: its number (the key that picks it), title, tradeoff and the recommendation, with a real radio or checkbox. A part of those cards.",
    dos: [
      "Write the tradeoff in detail, so each option can be judged on its own.",
      "Tag only the crew's recommendation.",
    ],
    accessibility: [
      "A real radio or checkbox inside a label; at least 44px tall on touch screens.",
      "The chosen radio takes the accent; checked boxes don't, because many can be.",
      "The number is hidden from screen readers; the card around it handles the keys.",
    ],
    related: [
      "chat/DecisionCard.svelte",
      "chat/ClarifyCard.svelte",
      "chat/RunOptions.svelte",
      "mobile/PhoneOption.svelte",
    ],
  },
  {
    file: "chat/Citation.svelte",
    summary:
      "The number of a source, right after the claim it backs. It links to that source in the Sources list under the answer.",
    use: ["A claim in an answer comes from an attachment or a web page."],
    avoid: ["Linking words in a sentence: use Hyperlink."],
    dos: [
      "Put it straight after the claim, before the full stop.",
      "Use the same number as the source in Sources, and link to its id (`<id>-<n>`).",
    ],
    accessibility: [
      'As a link, screen readers hear which source it is: "Source 1: Peraturan Kos Mawar 2026.pdf".',
    ],
    related: [
      "chat/Sources.svelte",
      "chat/Hyperlink.svelte",
      "chat/LinkPreview.svelte",
    ],
  },
  {
    file: "chat/ClarifyAnswered.svelte",
    summary:
      "The clarify questions folded to one line once they are answered, so the thread stays readable. Picking it opens them again.",
    use: ["After the person has answered a ClarifyCard."],
    dos: [
      'Sum up the answers in the order asked, short and separated by ·: "Whoosh · Rp 3,5 jt · 1–2 Nov · 5 topics".',
    ],
    accessibility: ["One button; 44px tall on touch screens."],
    related: ["chat/ClarifyCard.svelte", "chat/ClarifyReview.svelte"],
  },
  {
    file: "chat/ClarifyCard.svelte",
    summary:
      "A few quick questions before planning, one at a time. Tabs show which are answered; the number keys pick, Enter goes on, Escape skips them all, and the last step leads to the review.",
    use: [
      "The crew needs a few answers before it can make a plan, e.g. how to travel, the budget and the dates.",
    ],
    avoid: ["One question with a few options: use DecisionCard."],
    dos: [
      'Give every question a short topic for its tab and the review: "Getting there", "Budget".',
      "Mark the recommended option; it starts picked.",
      'Use the helper to say why the question matters: "This decides the timing of everything else."',
      'Offer "Something else…" (other) when the options might not fit.',
    ],
    donts: [
      "Count the recommendation as an answer: a question is answered once the person has been to it.",
    ],
    content: [
      'The title says how many: "Before I plan: 4 quick questions".',
      'Each option\'s detail gives the tradeoff with real numbers: "45 min from Halim, about Rp 250rb each way per person."',
    ],
    accessibility: [
      "Number keys pick, Enter goes to the next question and Escape skips them all; every control is also reachable on its own.",
      'The question tabs sit in a nav named "Questions"; the current one is marked as the current step.',
      "Options are a radiogroup (or a group, for many of many) named by the question.",
    ],
    related: [
      "chat/ClarifyReview.svelte",
      "chat/ClarifyAnswered.svelte",
      "chat/ChoiceOption.svelte",
      "chat/DecisionCard.svelte",
    ],
  },
  {
    file: "chat/ClarifyReview.svelte",
    summary:
      "Your answers before anything is planned. An answer the crew filled in for you is marked Default, with the reason written underneath. Nothing runs yet: the plan comes next.",
    use: ["After the last clarify question, before the plan is made."],
    dos: [
      'Mark answers the crew picked as default, and say why in the assumption: "You skipped Dates, so I picked the next free weekend."',
      "Let every answer be changed.",
    ],
    donts: [
      "Run anything from here: confirming makes the plan, which the person still approves.",
    ],
    accessibility: [
      'Each pencil is named "Change …" with its topic, and is a 44px target on touch screens.',
    ],
    related: [
      "chat/ClarifyCard.svelte",
      "chat/ClarifyAnswered.svelte",
      "chat/PlanApproval.svelte",
    ],
  },
  {
    file: "chat/CodeBlock.svelte",
    summary:
      "Code in an answer: mono, numbered and coloured with the code tokens, with comments receding. Copy takes the code alone, never the line numbers.",
    use: ["An answer includes code the person may read or copy."],
    avoid: [
      "Changes to a file: use Diff.",
      "A project's files in Build: use CodeView.",
    ],
    dos: [
      "Name the language and the file when there is one: typescript, src/lib/budget.ts.",
      "Pass highlighted tokens for colour; without them the code shows plain.",
    ],
    donts: [
      "Put line numbers in the code: the block numbers its lines itself.",
    ],
    accessibility: [
      "Line numbers are hidden from screen readers and can't be selected.",
      'The copy button says "Copied" only once the clipboard has the code.',
      "Long lines scroll sideways rather than wrap.",
    ],
    related: [
      "chat/CodeToken.svelte",
      "chat/Diff.svelte",
      "build/CodeView.svelte",
    ],
  },
  {
    file: "chat/CodeToken.svelte",
    summary:
      "One highlighted piece of code, coloured by the nm-code-* tokens (comments recede to text-3, punctuation to text-2). A part of CodeBlock and of Build's CodeView.",
    related: ["chat/CodeBlock.svelte", "build/CodeView.svelte"],
  },
  {
    file: "chat/ComposerLimit.svelte",
    summary:
      "Under the composer when a message is over the limit: by how much, in characters as counted, and the way round it.",
    use: ["The message is longer than one message can be."],
    avoid: [
      "A file over the size limit: that is an upload error (ErrorCallout).",
    ],
    dos: [
      'Show the count and the limit as counted: "12.480 / 10.000".',
      'Pass onfile, so it offers "Send as a file instead".',
    ],
    accessibility: [
      "It is an alert, so screen readers announce it straight away.",
    ],
    related: ["inputs/Composer.svelte", "chat/AttachmentChip.svelte"],
  },
  {
    file: "chat/ConnectPrompt.svelte",
    summary:
      "A crew member needs a tool to go on. It says who needs it and why, exactly what it can read, that nothing changes without asking and it can be disconnected, and offers a way to go on without it.",
    use: [
      "A request needs a service that isn't connected yet, e.g. Google Calendar to check your weekends.",
    ],
    avoid: ["Something Nomi won't do at all: use Boundary."],
    dos: [
      'Say who needs it and why: "Tara needs it to check your weekends".',
      'Say exactly what it can read: "Read your events and free/busy times".',
      'Offer a way on without it: "Or tell me your free weekends".',
    ],
    accessibility: ["Connect opens the service in a new tab."],
    related: ["chat/Boundary.svelte", "chat/ApprovalCard.svelte"],
  },
  {
    file: "chat/CopiedNote.svelte",
    summary:
      'A small inverse note where you copied, saying what was copied, e.g. "Copied as Markdown".',
    use: ["Confirming a copy right where it happened."],
    dos: [
      'Say the form it was copied in when that matters: "Copied as Markdown". Without a label it says "Copied".',
    ],
    accessibility: ["It is a status, so screen readers announce it."],
    related: ["chat/ResponseActions.svelte", "overlays/Toast.svelte"],
  },
  {
    file: "chat/CrewWorking.svelte",
    summary:
      "A banner for who is working on this chat right now, with one way to stop them all.",
    use: ["One or more crew members are working on the chat."],
    avoid: [
      "Telling the person they can leave long work: use BackgroundNote.",
      "News about the connection, provider or plan: use Banner.",
    ],
    dos: [
      "Show every member who is working.",
      "Pass onstop, so there is always a way to stop.",
    ],
    accessibility: ["It is a status, so screen readers announce it."],
    related: [
      "chat/SupervisorStopped.svelte",
      "chat/BackgroundNote.svelte",
      "chat/Banner.svelte",
    ],
  },
  {
    file: "chat/DataTable.svelte",
    summary:
      "A table in an answer, for comparable rows only. The first column names the row; the others are numbers, in mono and right-aligned, with where the data came from underneath.",
    use: ["Rows that compare the same measures, e.g. food spending by week."],
    avoid: [
      "Things to act on: use a card, such as SelectionCard or Cart.",
      "On a phone: use the mobile StatList.",
    ],
    dos: [
      'Put the units and origin in source: "Rp · from Dana · 142 transactions".',
      "Close with a total row when the columns add up.",
      "Give a number a tone only when it is good or bad news (spending up is bad).",
    ],
    donts: [
      "Use a table for anything that isn't comparable rows.",
      "Colour numbers that are neither good nor bad news.",
    ],
    content: [
      "Keep column names short; they show in mono capitals.",
      'Format numbers before passing them, as the person reads them: "2.100.000", "+48%".',
    ],
    accessibility: [
      "A real table, named by its title, with column headers.",
      "Copy gives tab-separated rows that paste into a sheet; download gives a CSV. Both include the header and the total.",
      "The copy and download icons are named, and are 44px targets on touch screens.",
    ],
    related: [
      "chat/Cart.svelte",
      "chat/SelectionCard.svelte",
      "mobile/StatList.svelte",
    ],
  },
  {
    file: "chat/DateDivider.svelte",
    summary:
      'A quiet break between days in a thread, e.g. "Yesterday" or "Mon 27 Oct".',
    content: [
      'Write "Yesterday" or a short date like "Mon 27 Oct"; it shows in mono capitals.',
    ],
    accessibility: ["A separator named by its label."],
    related: ["chat/NewDivider.svelte", "chat/LoadingOlder.svelte"],
  },
  {
    file: "chat/DecisionCard.svelte",
    summary:
      "One of many, each option with its tradeoff written out. The crew's suggestion starts picked, the button names what you are choosing, and you can always answer in your own words in the composer instead.",
    use: [
      "The person picks one way forward from a few, e.g. when Kala should remind them about rent.",
    ],
    avoid: [
      "Picking several: use SelectionCard.",
      "Several questions before a plan: use ClarifyCard.",
    ],
    dos: [
      'Ask about the one thing being decided: "When should Kala remind you about rent?"',
      'Write each tradeoff with the real time or cost: "25th, 09:00. Time to top up BCA if needed."',
      "Mark one option as suggested; it starts picked.",
    ],
    donts: ["Leave out the tradeoffs: they are what the person decides on."],
    content: [
      'The button reads "Use" plus the option\'s title, so keep titles short: "Use 3 days before".',
    ],
    accessibility: [
      "The options are real radios in a radiogroup named by the question.",
    ],
    related: [
      "chat/SelectionCard.svelte",
      "chat/ClarifyCard.svelte",
      "chat/ChoiceOption.svelte",
      "chat/QuickReplies.svelte",
    ],
  },
  {
    file: "chat/DeleteReplyDialog.svelte",
    summary:
      "Confirms deleting a reply: what leaves (the reply and its cards), what stays (your message) and what can't be undone (things the crew already did, like a sent email).",
    use: ["Before a reply is deleted from its response actions."],
    dos: [
      "Pass the crew member's name: \"Nomi's reply and its cards leave this chat.\"",
    ],
    accessibility: [
      "A native modal dialog: focus stays inside until it closes, and Escape closes it.",
    ],
    related: [
      "chat/DeletedReply.svelte",
      "chat/ResponseActions.svelte",
      "overlays/Dialog.svelte",
    ],
  },
  {
    file: "chat/DeletedReply.svelte",
    summary:
      "What is left where a reply was, so the thread still makes sense. The reply can come back, or be asked again.",
    use: ["In place of a reply after it is deleted."],
    dos: ["Offer both Undo and Regenerate."],
    related: ["chat/DeleteReplyDialog.svelte", "chat/ResponseActions.svelte"],
  },
  {
    file: "chat/Diff.svelte",
    summary:
      "A crew member's edits to one file, applied only on approval. Old and new line numbers, the sign and a tinted background for each added or removed line.",
    use: ["Koda wants to change a file and needs the person's approval first."],
    avoid: [
      "Code that isn't a change: use CodeBlock.",
      "A send, spend or delete: use ApprovalCard.",
    ],
    dos: [
      "Say who is asking and how many files the request touches (request).",
      'Give each hunk its header as git writes it: "@@ -2,4 +2,6 @@ export function weeklyBudget".',
    ],
    donts: ["Apply a change before the person approves it."],
    accessibility: [
      'Screen readers hear the totals in words ("4 lines added, 1 removed") and each changed line as added or removed.',
      "Line numbers and signs are hidden from screen readers.",
    ],
    related: [
      "chat/CodeBlock.svelte",
      "chat/ApprovalCard.svelte",
      "build/CodeView.svelte",
    ],
  },
  {
    file: "chat/DislikeFeedback.svelte",
    summary:
      "What went wrong with a reply: any number of reasons first, then an optional note. Only this reply goes with it unless you include the whole chat, and amounts and names stay masked either way.",
    use: ["After the person dislikes a reply."],
    avoid: ["Liking a reply: that is one tap in ResponseActions."],
    dos: [
      "Reopen it with what was already picked and written (initialReasons, initialNote).",
    ],
    donts: [
      "Send the whole chat unless the person ticks it: it is off by default.",
    ],
    accessibility: [
      "Each reason is a toggle button that says whether it is pressed.",
      "The include-chat checkbox is described by its help text.",
      "Send feedback stays off until there is a reason or a note.",
    ],
    related: ["chat/FeedbackSent.svelte", "chat/ResponseActions.svelte"],
  },
  {
    file: "chat/DraftPlanCard.svelte",
    summary:
      "A plan before it runs, and editable: reorder, reassign or remove steps, or add one; then run it, ask for changes or discard it. Nothing runs until you say so.",
    use: [
      "The crew proposes work in several steps that the person should shape before it starts.",
    ],
    avoid: [
      "A plan that is running: use PlanCard.",
      "Choosing how much may run on its own: use PlanApproval.",
    ],
    dos: [
      'Say in the intro that nothing runs yet: "Reorder, reassign or remove steps; nothing runs until you say so."',
      "Give every step the crew member who owns it.",
      'Show the rough estimate and who is involved: "~2 min · Maya, Dana, Kala".',
    ],
    accessibility: [
      "Steps can be dragged, or moved with their handle and the up and down arrow keys.",
      'Handles and remove buttons are named by the step\'s number: "Move step 2", "Remove step 2".',
    ],
    related: [
      "chat/PlanCard.svelte",
      "chat/PlanApproval.svelte",
      "chat/ThoughtSteps.svelte",
    ],
  },
  {
    file: "chat/FeedbackSent.svelte",
    summary:
      "Thanks after feedback, and exactly what went with it: this reply only, or the whole chat. It can offer another try, e.g. with a different model.",
    use: ["After the person sends dislike feedback."],
    dos: [
      'Name the model in the retry: "Try again with Claude Sonnet".',
      "Pass withChat when the whole chat was sent.",
    ],
    accessibility: ["It is a status, so screen readers announce it."],
    related: ["chat/DislikeFeedback.svelte", "chat/ResponseActions.svelte"],
  },
  {
    file: "chat/FollowUps.svelte",
    summary: "Optional next questions under an answer. Picking one asks it.",
    use: ["There are useful next questions the person might ask."],
    avoid: ["Answers to the question just asked: use QuickReplies."],
    dos: [
      'Write each one as the person would ask it, and make it specific: "Set a weekly food budget of Rp 500rb".',
    ],
    donts: [
      "Use them for answers to the last question: those are quick replies.",
    ],
    accessibility: [
      'Each is a button, 44px tall on touch screens, under a "Follow up" label.',
    ],
    related: ["chat/QuickReplies.svelte", "chat/DecisionCard.svelte"],
  },
  {
    file: "chat/FormCard.svelte",
    summary:
      "A real form in the thread, prefilled from the chat. Validation sits under each field, never in a toast.",
    use: [
      "The crew needs a few details to create something, e.g. a new reminder.",
    ],
    avoid: ["One choice from a few: use DecisionCard."],
    dos: [
      'Prefill from what the person already said, and say so: "prefilled from chat".',
      'Say what happens after saving: "Kala pings you in Inbox and on your phone".',
      'Name the submit after the result: "Save reminder".',
    ],
    donts: ["Show validation in a toast: it belongs under its field."],
    accessibility: [
      "A real form: its submit button submits, and so does Enter in a text field.",
    ],
    related: [
      "inputs/TextField.svelte",
      "inputs/Segmented.svelte",
      "chat/DecisionCard.svelte",
      "chat/Reminder.svelte",
    ],
  },
  {
    file: "chat/Handoff.svelte",
    summary:
      "Who passed the question to whom, and why, so the answer's owner is never a surprise. You can send it back.",
    use: ["A crew member passes the question to another."],
    dos: [
      'Give the reason in a few words: "she has your transaction records".',
      'Pass onback, so the person can "Ask Nomi instead".',
    ],
    donts: ["Change who is answering without showing it."],
    accessibility: ["The send-back button is 44px tall on touch screens."],
    related: [
      "chat/Boundary.svelte",
      "chat/CrewWorking.svelte",
      "chat/MentionCrew.svelte",
    ],
  },
  {
    file: "chat/Hyperlink.svelte",
    summary:
      "A link inside an answer, a step brighter than the sentence around it and underlined on hover. It takes the sentence's size.",
    use: ["Linking words in a sentence of an answer."],
    avoid: [
      "Pointing to a source by number: use Citation.",
      "A page the answer is about: use LinkPreview.",
      "A link outside prose: use Link.",
    ],
    dos: [
      'Link the words that say where it goes: "house rules Bu Sri sent".',
      "Set external for another site, so it opens in a new tab.",
    ],
    related: [
      "chat/Citation.svelte",
      "chat/LinkPreview.svelte",
      "actions/Link.svelte",
    ],
  },
  {
    file: "chat/JumpToLatest.svelte",
    summary:
      "Floats above the composer once you have scrolled up, saying how many replies came in since.",
    use: ["The person has scrolled up a long thread."],
    dos: ['Pass the real count; with none it reads "Jump to latest".'],
    accessibility: ["44px tall on touch screens."],
    related: ["chat/NewDivider.svelte", "chat/LoadingOlder.svelte"],
  },
  {
    file: "chat/LinkPreview.svelte",
    summary:
      "A page the answer points to, as one card that opens it in a new tab: the site, the page's title, a line about it and the page's own picture.",
    use: ["The answer is about one specific page, e.g. a listing."],
    avoid: [
      "A link inside a sentence: use Hyperlink.",
      "Backing a claim: use Citation and Sources.",
    ],
    dos: ["Use the page's own title, description and picture."],
    donts: ["Use a picture that isn't the page's own."],
    accessibility: [
      "The picture is decorative (empty alt); the title says what the page is.",
      "It opens in a new tab.",
    ],
    related: [
      "chat/Hyperlink.svelte",
      "chat/Citation.svelte",
      "chat/Sources.svelte",
    ],
  },
  {
    file: "chat/LoadingOlder.svelte",
    summary: "At the top of a long thread while earlier messages load.",
    accessibility: [
      "It is a status, so screen readers announce it.",
      "The spinner stops when reduced motion is on.",
    ],
    related: [
      "chat/DateDivider.svelte",
      "chat/NewDivider.svelte",
      "chat/JumpToLatest.svelte",
    ],
  },
  {
    file: "chat/MemorySaved.svelte",
    summary:
      "Something new remembered, said where it happened, with edit and undo right there.",
    use: ["The crew saves something about the person from the chat."],
    avoid: ["An answer that used something remembered: use MemoryUsed."],
    dos: [
      'Write the fact as it reads after "Saved to memory:": "you and Sari prefer trains to driving".',
      "Offer edit and undo.",
    ],
    donts: ["Remember anything silently: memory is never silent."],
    accessibility: ["It is a status, so screen readers announce it."],
    related: ["chat/MemoryUsed.svelte"],
  },
  {
    file: "chat/MemoryUsed.svelte",
    summary:
      "A line under an answer that leaned on something remembered. Memory is never silent.",
    use: ["An answer used something the crew remembered about the person."],
    avoid: ["Something new being remembered: use MemorySaved."],
    dos: [
      'Write the fact as it reads after "From memory:": "Sari is vegetarian".',
      "Offer Manage, so the person can change what is remembered.",
    ],
    related: ["chat/MemorySaved.svelte"],
  },
  {
    file: "chat/MentionCrew.svelte",
    summary:
      '"@Dana" in that crew member\'s own tint: the only place in a message the crew colours appear.',
    use: ["A crew member is named in a message."],
    avoid: ["A file: use MentionFile."],
    related: ["chat/MentionFile.svelte", "chat/MentionPicker.svelte"],
  },
  {
    file: "chat/MentionFile.svelte",
    summary:
      "A file named in a message, in mono and neutral, because files aren't crew.",
    use: ["A file is named in a message."],
    avoid: ["A crew member: use MentionCrew."],
    related: ["chat/MentionCrew.svelte", "chat/MentionPicker.svelte"],
  },
  {
    file: "chat/MentionPicker.svelte",
    summary:
      'Opens under "@" in the composer: the crew first, then files. Focus stays in the composer while you pick.',
    use: ["The person types @ in the composer."],
    dos: [
      'Give each crew member their role: "Money and subscriptions".',
      'Give each file where it lives: "Drive".',
    ],
    accessibility: [
      "A listbox: the composer keeps focus and points at the active option with aria-activedescendant.",
      "nextOption moves the active option with the arrow keys, Home and End.",
      "A click picks without taking focus from the composer.",
    ],
    related: [
      "chat/MentionCrew.svelte",
      "chat/MentionFile.svelte",
      "inputs/Composer.svelte",
    ],
  },
  {
    file: "chat/MessageEditor.svelte",
    summary:
      "A sent message edited in place. Saving sends it as a new version; the old one and its reply stay, reachable from the version switcher.",
    use: ["The person edits a message they have sent."],
    dos: [
      'Pass the version, so the hint says where the old reply stays: "1 / 2".',
    ],
    donts: ["Overwrite the old message: edits branch instead."],
    accessibility: [
      "Focus goes to the end of the text when it opens.",
      "Escape cancels; ⌘ or Ctrl+Enter saves.",
      "Save & send stays off until the text has changed.",
    ],
    related: ["chat/VersionSwitcher.svelte", "conversation/MessageUser.svelte"],
  },
  {
    file: "chat/MoveDialog.svelte",
    summary:
      "Picks the project a chat belongs to. Moving into a shared workspace says who will see it, and what stays yours.",
    use: ['"Move to a project" from the chat menu.'],
    dos: [
      "Pass shared for a project in a shared workspace, so the dialog says who will see the chat.",
    ],
    accessibility: [
      'Projects are radios in a radiogroup; search narrows them, and "No projects match" says when nothing does.',
      "A native modal dialog: focus stays inside until it closes.",
    ],
    related: [
      "chat/ChatMenu.svelte",
      "overlays/Dialog.svelte",
      "inputs/Search.svelte",
    ],
  },
  {
    file: "chat/NewDivider.svelte",
    summary:
      "Where you left off in a thread, in mint, with how many replies came in since.",
    use: ["The person comes back to a chat that has new replies."],
    dos: ["Pass the real count."],
    accessibility: ["A separator named by its label."],
    related: ["chat/DateDivider.svelte", "chat/JumpToLatest.svelte"],
  },
  {
    file: "chat/PlanApproval.svelte",
    summary:
      "How much of a plan may run on its own, with the recommendation picked first, or keep planning with what you want changed. The button says what will happen.",
    use: ["A plan is ready and needs the person's go-ahead."],
    avoid: ["Changing the steps themselves: use DraftPlanCard."],
    dos: [
      "Mark one option as recommended; it starts picked.",
      'Say in each option what runs freely and what still asks: "Research runs freely. Tara asks before sharing the doc; Dana only records."',
      "Set who to the crew member the changes go to.",
    ],
    accessibility: [
      "Number keys pick and Enter runs; every control is also reachable on its own.",
      "The options are a radiogroup.",
    ],
    related: [
      "chat/RunOptions.svelte",
      "chat/DraftPlanCard.svelte",
      "chat/PlanCard.svelte",
      "build/BuildPlan.svelte",
    ],
  },
  {
    file: "chat/PlanCard.svelte",
    summary:
      "A plan while it runs: numbered steps with their real state, what each produced or waits for, the crew member who owns it, how far along it is, and a way to stop.",
    use: ["Work in several steps is running."],
    avoid: [
      "A plan that hasn't started: use DraftPlanCard.",
      "The steps behind one answer: use ThoughtSteps.",
    ],
    dos: [
      'Write each step\'s meta from what really happened: "142 rows", "running · 00:06", "waits for step 3".',
      "Pass onstop, so the plan can be stopped.",
    ],
    donts: ["Show progress nobody measured: the count is steps done."],
    accessibility: [
      "The running step's spinner stops when reduced motion is on.",
    ],
    related: [
      "chat/DraftPlanCard.svelte",
      "chat/ThoughtSteps.svelte",
      "chat/SupervisorStopped.svelte",
      "chat/PlanApproval.svelte",
    ],
  },
  {
    file: "chat/QuickReplies.svelte",
    summary:
      "Answers to the question just asked, one tap each. Round, so they read as replies rather than actions.",
    use: ["The last message asked something with a few likely answers."],
    avoid: [
      "Optional next questions: use FollowUps.",
      "Choices with tradeoffs to weigh: use DecisionCard.",
    ],
    dos: [
      'Write them as the person\'s answer: "Yes, set it", "Change the time", "Not now".',
    ],
    donts: ["Use them for actions: they are replies."],
    accessibility: ["Each is a button, 44px tall on touch screens."],
    related: ["chat/FollowUps.svelte", "chat/DecisionCard.svelte"],
  },
  {
    file: "chat/Reasoning.svelte",
    summary:
      'The model\'s thinking: live while it thinks ("Thinking 00:04" over the stream), then folded away once the answer lands ("Thought for 8s"), open on request.',
    use: ["The model reports its reasoning."],
    avoid: [
      "Steps with tools and timings: use ThoughtSteps.",
      "One decision in a line: use Thought.",
    ],
    dos: [
      "Fold it once the answer lands.",
      "Show the real time so far while it is live.",
      'Say how long it took once folded: "Thought for 8s".',
    ],
    donts: [
      "Leave it open after the answer lands: it is collapsed by default.",
    ],
    accessibility: [
      "Folded, it is a button that says whether it is expanded and which text it controls.",
      "The live spinner stops when reduced motion is on.",
    ],
    related: ["chat/Thought.svelte", "chat/ThoughtSteps.svelte"],
  },
  {
    file: "chat/RegenerateMenu.svelte",
    summary:
      "The menu off a reply's Regenerate button: the same again, shorter, with more detail, as a list, or with another model (and what it is paid with).",
    use: ["Behind Regenerate in the response actions."],
    dos: [
      'List each model with what it is paid with: "your key", "Nomi credits".',
    ],
    accessibility: [
      "Built on Menu: the arrows and Home/End move, Enter or Space picks, Escape closes.",
    ],
    related: ["chat/ResponseActions.svelte", "overlays/Menu.svelte"],
  },
  {
    file: "chat/Reminder.svelte",
    summary:
      "A reminder Kala keeps, in Kala's tint. Scheduled, it says when and who keeps it, with a way to edit; due, it turns to warning and offers snooze and done.",
    use: ["A reminder is set, or comes due, in the chat."],
    avoid: ["Setting up a reminder with several details: use FormCard."],
    dos: [
      'Write when with its repeat: "25 Nov · 09:00 · every month".',
      'Name what done means for this one: "Mark paid".',
    ],
    accessibility: ['The edit button is named "Edit reminder".'],
    related: ["chat/FormCard.svelte", "chat/DecisionCard.svelte"],
  },
  {
    file: "chat/RenameField.svelte",
    summary:
      "The chat's name, edited in place in the top bar. Enter saves (not when empty), Escape cancels.",
    use: ["Rename, from the chat menu."],
    accessibility: [
      'Named "Chat name"; it takes focus with the name selected when it opens.',
    ],
    related: ["chat/ChatMenu.svelte"],
  },
  {
    file: "chat/ReplyText.svelte",
    summary:
      "A reply's prose, and whether it is still coming (a mint caret), was stopped by you (and after how long), or was cut off (the reason, in the provider's terms, and what to do next).",
    use: ["The text of every reply."],
    avoid: ["A request that failed: use ErrorCallout."],
    dos: [
      'Quote the provider\'s reason when it is cut off: "Reply cut off after 18s. DeepSeek closed the stream."',
      'Say how long it ran when stopped: "stopped by you · 00:03".',
      "Offer Continue, and Start over after an interruption.",
    ],
    donts: ["Guess why it stopped: quote the provider."],
    accessibility: [
      'While streaming it is marked busy and screen readers hear "Writing…"; the caret is hidden from them.',
      "The caret stops blinking when reduced motion is on.",
    ],
    related: [
      "chat/Reasoning.svelte",
      "chat/ResponseActions.svelte",
      "chat/StopHint.svelte",
      "conversation/MessageAgent.svelte",
      "conversation/ErrorCallout.svelte",
    ],
  },
  {
    file: "chat/ResponseActions.svelte",
    summary:
      "The same quiet bar under every reply: copy, like, dislike, share, regenerate and delete, with what answered, how long it took and how many tokens it cost on the right.",
    use: ["Under every finished reply."],
    dos: [
      'Fill meta from the real call: "deepseek-flash · 1.4s · 812 tokens".',
      "Pass feedback, so like or dislike shows as pressed.",
    ],
    donts: ["Hide the model, time or tokens: they stay visible."],
    accessibility: [
      'A group named "Reply actions"; every icon button has a name.',
      "Like and dislike are toggle buttons that say whether they are pressed.",
      "44px targets on touch screens.",
    ],
    related: [
      "chat/RegenerateMenu.svelte",
      "chat/DislikeFeedback.svelte",
      "chat/ShareDialog.svelte",
      "chat/DeleteReplyDialog.svelte",
    ],
  },
  {
    file: "chat/RunOptions.svelte",
    summary:
      "How a plan may run, as numbered choices with the recommendation picked first, and a last one to keep planning with what you want changed (typing in it picks it). A part of PlanApproval and Build's BuildPlan.",
    related: [
      "chat/PlanApproval.svelte",
      "build/BuildPlan.svelte",
      "chat/ChoiceOption.svelte",
    ],
  },
  {
    file: "chat/SelectionCard.svelte",
    summary:
      "Many of many: each item with what it is and what it costs, and a running count and total of what you picked. Unpicked rows recede.",
    use: [
      "The person picks any number from a list, e.g. which subscriptions Dana should track.",
    ],
    avoid: ["One of many: use DecisionCard."],
    dos: [
      'Update the total as picks change (onchange), formatted: "Rp 285.990 / month".',
      'Name the confirm after what it does, with the count: "Track 3".',
      "Offer Skip when picking is optional.",
    ],
    accessibility: [
      "Each row is a label with a real checkbox; rows are 44px tall on touch screens.",
      "The confirm button stays off until something is picked.",
    ],
    related: [
      "chat/DecisionCard.svelte",
      "chat/Cart.svelte",
      "chat/ClarifyCard.svelte",
    ],
  },
  {
    file: "chat/SessionExpired.svelte",
    summary:
      "The sign-in ran out mid-chat. Nothing is lost: the unsent message stays on this device and goes out after signing in, and the crew keeps working.",
    use: ["The person's session ends while they are in a chat."],
    dos: ["Show the kept draft, so the person can see it is safe."],
    related: ["chat/Banner.svelte"],
  },
  {
    file: "chat/ShareDialog.svelte",
    summary:
      "The share panel as a modal on the overlay scrim: this reply or the whole chat, with amounts and names hidden by default.",
    use: ["Share, from a reply's actions or the chat menu."],
    accessibility: [
      "A native modal dialog named by its heading; it closes with its own button or Escape.",
    ],
    related: [
      "chat/SharePanel.svelte",
      "chat/ResponseActions.svelte",
      "chat/ChatMenu.svelte",
    ],
  },
  {
    file: "chat/SharePanel.svelte",
    summary:
      "What the share dialog holds: this reply or the whole chat, a preview of what others will see, amounts and names hidden by default, who can open it, when it expires, and the link.",
    dos: [
      'Pass a preview that is already masked while masking is on: "Rp •••".',
      "Keep amounts and names hidden by default.",
    ],
    donts: ["Show real amounts or names in the preview while masking is on."],
    accessibility: [
      "Each setting is a named switch, and the scope is a named segmented control.",
    ],
    related: ["chat/ShareDialog.svelte", "build/BuildSharePanel.svelte"],
  },
  {
    file: "chat/Sources.svelte",
    summary:
      "What an answer leaned on, numbered like its citations, each with where it came from: an attachment and who read it, or a site's domain.",
    use: ["Under an answer that cites its sources."],
    dos: [
      'Say where each came from: "attachment · read by Maya", or the site\'s domain.',
      "Give it an id, so citations can link to each item (`<id>-<n>`).",
    ],
    accessibility: [
      'An ordered list under a "Sources" label; web sources open in a new tab.',
    ],
    related: ["chat/Citation.svelte", "chat/LinkPreview.svelte"],
  },
  {
    file: "chat/StopHint.svelte",
    summary: "While a reply streams, stop it here or with Escape.",
    use: ["While a reply is streaming."],
    accessibility: [
      "It declares Escape as its shortcut (aria-keyshortcuts); the page listens for the key.",
    ],
    related: ["chat/ReplyText.svelte", "chat/CrewWorking.svelte"],
  },
  {
    file: "chat/SupervisorStopped.svelte",
    summary:
      "What the supervisor stopped, task by task: done before the stop, stopped part-way (and how far it got), or never started. What already left Nomi isn't undone.",
    use: ["The supervisor, or the person, stops the crew's work."],
    dos: [
      'Say how far a stopped task got: "stopped at 2 of 3 found".',
      'Say plainly what didn\'t happen: "not started · nothing sent".',
      'Say when and why in the meta: "14:31 · you asked".',
    ],
    related: ["chat/CrewWorking.svelte", "chat/PlanCard.svelte"],
  },
  {
    file: "chat/Thought.svelte",
    summary: "One decision the crew made on the way, said quietly in a line.",
    use: ["A choice that shapes the answer and is worth a sentence."],
    avoid: [
      "Several steps with tools and timings: use ThoughtSteps.",
      "The model's full reasoning: use Reasoning.",
    ],
    dos: [
      "Say what was decided and why: \"You asked about this month only, so I'll leave last year's data out.\"",
    ],
    related: ["chat/ThoughtSteps.svelte", "chat/Reasoning.svelte"],
  },
  {
    file: "chat/ThoughtSteps.svelte",
    summary:
      'The real steps behind an answer, each with its tool, size and time in mono ("dana.records · 142 rows · 1.1s").',
    use: ["An answer took several tool calls worth showing."],
    avoid: ["A plan the person approved and can stop: use PlanCard."],
    dos: [
      "Write each step's meta from the real call: tool, size, time.",
      'Sum up in the label: "Worked through 4 steps", with "· 12s".',
    ],
    donts: ["Add steps that didn't happen: never theatrical."],
    accessibility: [
      "A button that says whether it is expanded opens and folds the list.",
      "The running step's spinner stops when reduced motion is on.",
    ],
    related: [
      "chat/Thought.svelte",
      "chat/Reasoning.svelte",
      "chat/PlanCard.svelte",
    ],
  },
  {
    file: "chat/VersionSwitcher.svelte",
    summary:
      'Steps between the versions of a message ("‹ 2 / 2 ›"). Edits branch instead of overwriting, so a message can have versions.',
    use: ["A message has been edited, so it has more than one version."],
    accessibility: [
      "Previous and next are named, and turn off at either end.",
      'The count is read as "Version 2 of 2".',
      "44px targets on touch screens.",
    ],
    related: ["chat/MessageEditor.svelte", "conversation/MessageUser.svelte"],
  },
  {
    file: "chat/VoiceRecorder.svelte",
    summary:
      "Takes the composer's place while recording: the real input level as bars (the latest fading in), the time so far and the language it transcribes in. Discard it, or use it as text you can edit before sending.",
    use: ["The person records a voice message."],
    dos: [
      "Feed it the real input levels, 0 to 1, oldest first.",
      'Show the language it transcribes in: "Bahasa Indonesia".',
    ],
    donts: ["Animate made-up levels: the bars are the real input."],
    accessibility: [
      'A group named "Recording"; the bars and the red dot are hidden from screen readers.',
      'The buttons are named "Discard recording" and "Use recording".',
      "The dot stops pulsing when reduced motion is on.",
    ],
    related: ["chat/VoiceTranscribed.svelte", "inputs/Composer.svelte"],
  },
  {
    file: "chat/VoiceTranscribed.svelte",
    summary:
      "Under the composer, a note that its text came from audio and can be edited before sending.",
    use: ["After a recording is used."],
    related: ["chat/VoiceRecorder.svelte"],
  },
] satisfies ComponentDoc[];
