import type { PatternDoc } from '../types';

// The Build sheet, section by section: what each pattern does for the person and the rules for it,
// drawn from the sheet's own notes (pen/export/build--*.html) and the components that build it.
// Quoted copy is the pen's; every number comes from the sheet's one story, warung-kopi-menu.

export default [
	{
		sheet: 'build',
		id: 'build-estimate',
		summary:
			'Koda’s plan, shown before anything runs: the steps, what you’ll have at the end, and one clear choice. Cost is a quiet line, never the headline. Once the build starts it collapses into the thread, and when it’s live it shows what the build actually took.',
		use: [
			'Someone has described an app and Koda is ready to build it.',
			'Someone wants it built but held back until they press Publish.',
			'The plan needs changing before anything runs (Keep planning).',
			'A build is running, or has just gone live.',
		],
		dos: [
			'Name the project and what it is: “warung-kopi-menu”, “Menu and pre-order page for Warung Kopi Bu Siti”.',
			'List the steps Koda will take in plain words, then what you’ll have when it’s done.',
			'Offer the ways to run it as numbered choices, the recommendation picked first, with a last one to keep planning.',
			'Put time and cost on one quiet line under the choice: “About 5 min · included in your free plan”.',
			'While it builds, show the real step, file and clock with Stop; once it’s live, show the link with Open and Share, and what the build took.',
		],
		donts: [
			'Don’t lead with the cost.',
			'Don’t start anything before the person chooses.',
			'Don’t show a percentage or a progress bar while it builds; count the plan’s steps (“Step 2 of 6”).',
			'Don’t leave the full estimate open once the build starts.',
		],
		content: [
			'Head the steps with who does them: “Here’s what Koda will build”.',
			'The outcome names exactly what will exist: “When it’s done: a live link customers can open today. The code is yours to export.”',
			'Choices say what will happen: “Build it”, “Build it, ask before it goes live”, “Keep planning”.',
			'Machine facts are mono: “writing src/routes/menu/+page.svelte”, “01:12”, “deploy #1 · 08:18”. Rough estimates carry a ~: “~3 min left”.',
			'Once it’s live, say what changes next: “Changes after this need Publish”.',
		],
	},
	{
		sheet: 'build',
		id: 'building',
		summary:
			'What you see while Koda builds: the real step, the real file, the real clock. No percentage, no bar. The preview stays empty until there is something real to show.',
		use: [
			'Koda is writing a project’s files.',
			'The preview has nothing to show before the first deploy.',
			'Someone wants to see what’s done, what’s being written and what comes next.',
		],
		dos: [
			'Count the files written out of all of them: “5 of 9 written”.',
			'Show each finished file with the lines it added (“+38”), mark the one being written now, and fade the ones still to come.',
			'In the empty preview, say where the app will appear and list the plan’s real steps.',
		],
		donts: [
			'Don’t show a percentage or a progress bar.',
			'Don’t fill the preview with a mock-up before there’s a real deploy.',
			'Don’t guess how long is left when nobody knows; show the elapsed time and the current step.',
		],
		content: [
			'File paths are mono and exact: “src/routes/menu/+page.svelte”.',
			'The empty preview says when it opens: “The preview opens here after the first deploy”.',
			'Steps reuse the plan’s words: “Menu page”, “Pre-order form”, “Deploy to the live link”.',
		],
	},
	{
		sheet: 'build',
		id: 'share-export',
		summary:
			'Two dialogs reached from the live card: Share gets the link out, Export hands over everything Koda built. Export is never behind a plan: it takes the same number of clicks as upgrading, and works on Free.',
		use: [
			'A build is live and someone wants to send the link.',
			'Someone decides whether the app can appear in search.',
			'Someone wants to invite people to build together.',
			'Someone wants the code and the data, to take elsewhere.',
		],
		dos: [
			'Lead with the link and Copy link, then show how it looks when shared, with a way to edit that.',
			'Explain Public and Unlisted in one line: “Public can appear in search. Unlisted only opens with the link.”',
			'Keep the QR code and embed under More ways to share.',
			'List what Export includes: the repo with its git history, a database dump, and the env var names.',
			'Give the two ways out equal weight: Push to GitHub and Download zip.',
		],
		donts: [
			'Don’t put Export behind a paid plan, or make it take more clicks than upgrading.',
			'Don’t export env var values. Say they’re left out, and that they need setting again wherever the app is deployed.',
			'Don’t list a database dump for an app that stores no data.',
		],
		content: [
			'Export opens with ownership: “Everything Koda built is yours. Included on Free, no fee.”',
			'Name what’s included exactly, in mono where a machine made it: “9 files · git history · main@4be1c07”, “DATABASE_URL · PUBLIC_SITE_URL · ADMIN_PIN”.',
			'Say what’s left out plainly: “Env var values are not exported. Set them again wherever you deploy.”',
		],
	},
	{
		sheet: 'build',
		id: 'first-run',
		summary: 'One screen after the sign-in code: a name and one line on what Nomi does. No crew tour; the composer comes next.',
		use: [
			'Someone has just entered their sign-in code for the first time.',
			'Nomi needs the name it will show on the person’s projects.',
			'Nomi already knows the name and only needs it confirmed: it arrives prefilled.',
		],
		dos: [
			'Ask for one input at most: the name, prefilled when it’s known.',
			'Say in one line what Nomi does and what the person gets.',
			'Say where the name shows and where it can be changed later.',
			'Go straight to the composer after Continue.',
		],
		donts: [
			'Don’t add a crew tour.',
			'Don’t put more onboarding screens between this one and the composer.',
			'Don’t make the one line a pitch: say plainly what Nomi does.',
		],
		content: [
			'The title is a plain question: “What should we call you?”',
			'The line says what Nomi does and what you keep: “Nomi builds real web apps from a description. You get a live link, and the code is yours to export.”',
			'Helper text says where the value shows and where to change it: “Shown on your projects. Change it in Settings › Profile.”',
		],
	},
	{
		sheet: 'build',
		id: 'workspace',
		summary:
			'Chat and preview share one screen. The split is draggable, and every project tool opens as a sheet over the preview, so the app you are looking at never closes. Publish in the top bar decides what goes live.',
		use: [
			'Someone works on a project with Koda while watching the preview.',
			'The chat or the preview needs more room.',
			'Someone opens a project tool, such as the domain, database or settings.',
			'Changes are waiting in the preview to go live.',
		],
		dos: [
			'Keep the chat between 360 and 720 px and the preview at 480 or wider. A double-click on the handle resets the chat to 520.',
			'Let the handle move with the arrow keys too, with Shift for bigger steps.',
			'Open every project tool as a sheet over the preview; Escape closes it back to the same preview.',
			'Show the count on Publish as what will go live, and keep the last live deploy running while publishing and after a failure.',
		],
		donts: [
			'Don’t close or reload the preview to show a project tool.',
			'Don’t put changes live before Publish is pressed.',
			'Don’t take the live site down while publishing, or when a publish fails.',
		],
		content: [
			'Publish states are short and exact: “Up to date”, “Publish 2”, “Publishing… 0:12”, “Publish failed · Retry”.',
			'Each state means one thing for the live site: “Publishing · the live site keeps running”, “Failed · the last live deploy stays up”.',
			'Sheet hints say what stays: “esc closes · preview keeps running”.',
		],
	},
	{
		sheet: 'build',
		id: 'code-terminal-phone',
		summary:
			'The preview pane’s other two tabs and the phone switch. Code and Terminal are read-only: what Koda wrote and what Koda ran. Changes always go through the chat.',
		use: [
			'Someone wants to read the code Koda wrote, or ask about particular lines.',
			'Someone wants to see what Koda ran for a build, test or deploy, and why it failed.',
			'Someone checks the app at real phone size, or opens it on their own phone.',
		],
		dos: [
			'Show the file tree with its commit (“FILES · main@9d2e41a”), and when Koda last edited the open file.',
			'Let people pick line numbers (Shift for a range) and ask Koda about them; the lines are quoted in the composer.',
			'List the commands in order with the time of each, and sum up a failure in one line at the foot: what failed, what’s live, and a way to ask for the fix.',
			'Show the phone preview at real phone size, light inside the dark workspace, with the device named: “iPhone 15 · 393 × 852”.',
		],
		donts: [
			'Don’t let people edit code or run commands here; changes go through the chat.',
			'Don’t leave people guessing what’s live after a failure: “live stays on deploy #2”.',
			'Don’t re-theme the app to match the dark workspace: the phone preview shows it light.',
		],
		content: [
			'Read-only notes point to the chat: “Read-only. Ask Koda in the chat to change anything; every change is a commit you can roll back.”',
			'Failure summaries say what failed and what customers see: “1 test failed: no weekend pickup slots after 11:00”, “Nothing went live. Customers still see deploy #2.”',
			'Ask buttons name what they ask about: “Ask Koda about this file”, “Ask Koda about lines 14–15”, “Ask Koda to fix”.',
		],
	},
	{
		sheet: 'build',
		id: 'build-together',
		summary:
			'Invite people into a project’s chat. Everyone prompts the same Koda; asks queue so changes never collide; roles decide who can publish.',
		use: [
			'Someone invites a teammate into a project.',
			'Two people ask Koda for changes at the same time.',
			'A teammate who can prompt but not publish has a change ready.',
			'Someone wants to see where a teammate is in the project, and follow them.',
		],
		dos: [
			'Show each teammate in their own peer colour, with initials, when they wrote and that they asked Koda. You stay neutral.',
			'Queue asks one change at a time, with who asked and where each one is; the next can be moved up (“Do Sari’s first”).',
			'Pick a role when inviting, changeable later in People: Can build, Can prompt or Can view.',
			'Ask the publisher when a teammate’s change is built and tested in the preview, with Publish and Review in preview.',
			'Show who’s in the project and what each is looking at, with Follow and Stop following.',
		],
		donts: [
			'Don’t run two people’s changes at once.',
			'Don’t let someone who can only prompt publish; the builder does.',
			'Don’t give teammates crew colours. Crew colours belong to the crew; teammates have peer colours.',
		],
		content: [
			'Joined notes say what the person can do and who publishes: “Sari joined · can prompt · Mario publishes”.',
			'Each role is one line: “Prompt Koda, edit and publish”, “Ask Koda for changes; you publish”, “Open the preview and code, no changes”.',
			'The invite email says who invited you to what, with one button: “Mario Prasetyo invited you to build warung-kopi-menu”, “Join and build”.',
			'The queue explains itself: “Koda finishes one change before starting the next, so edits never collide.”',
		],
	},
	{
		sheet: 'build',
		id: 'build-on-mobile',
		summary:
			'Building on a phone, in one column at 390 pt. Chat and Preview switch with a segmented control; every project tool is a bottom sheet; the live app gets the full screen with Koda one tap away.',
		use: [
			'Someone builds or changes a project from their phone.',
			'Someone is looking at their live app on a phone and wants to ask Koda about a screen.',
			'A build runs while the app is closed.',
			'Someone sets up a domain, or publishes a teammate’s change, on a phone.',
		],
		dos: [
			'Switch Chat and Preview with a segmented control; Preview counts the changes waiting to be seen.',
			'Float the ask bar over the live app, so Koda is one tap away.',
			'Show the plan with one big Build it, what it costs under it, and the two other choices side by side.',
			'While building, show one card with the real step and file, and say the app can be closed: a notification says when it’s live.',
			'Show DNS records one per card, the value copied in one tap, with whether the domain’s provider has it yet.',
		],
		donts: [
			'Don’t use popovers; every project tool is a bottom sheet.',
			'Don’t show a percentage while it builds. The bars are the plan’s steps.',
			'Don’t shrink the queue and the publish request below thumb size.',
		],
		content: [
			'Shorten the desktop copy where the phone needs it: “Let Sari go first”, “Sari’s change is ready”, “One change at a time, so edits never collide.”',
			'The ask bar names its context: “Ask Koda about this screen”.',
			'Say what happens if they leave: “You can close the app. We’ll notify you when it’s live.”',
			'Status is a few words: “live · deploy #2”, “Up to date”, “waiting”.',
		],
	},
	{
		sheet: 'build',
		id: 'story-sheet',
		summary:
			'The one story every Build frame is drawn from: one person, one project, one timeline, the same numbers everywhere. Values in brackets aren’t decided yet and stay visible in the design.',
		use: [
			'Someone designs or builds a frame in Flow / Build and needs its names, times and numbers.',
			'Someone writes copy or a test that shows a build’s estimate, actuals, URL or deploy.',
			'A value isn’t decided yet, such as the Pro price or the hosting domain.',
		],
		dos: [
			'Take every name, time and number from the sheet: Mario Prasetyo, warung-kopi-menu, build started 08:13, deploy #1 at 08:18.',
			'Keep the timeline the same across frames: code verified 08:11, build started 08:13, live at 08:18 after 05:40.',
			'Leave undecided values in brackets, where people can see them: “[cost per 1K tokens]”, “[hosting domain]”.',
		],
		donts: [
			'Don’t replace an open value with a made-up figure; keep the bracket.',
			'Don’t let numbers drift between frames, such as a different file count or deploy number.',
			'Don’t treat the URL’s domain as final; it stands in for the [hosting domain].',
		],
		content: [
			'Estimates are ranges and actuals are exact: “4–7 min · 38K–62K tokens” against “05:40 · 51.2K tokens”.',
			'Open values keep their bracketed names: [Pro price IDR], [Free live-project cap], [Free builds/month], [Pro allowance], [hosting domain], [e-wallet list], [payment provider], [cost per 1K tokens].',
		],
	},
	{
		sheet: 'build',
		id: 'build-types',
		summary:
			'The same workspace for every build. Only what the app actually has shows up: no database tab for a landing page, no QR unless it’s for a physical place.',
		use: [
			'Koda sets up a landing page, a web app with data, a catalog or shop, an internal tool, or a portfolio or blog.',
			'Deciding which workspace tabs a project gets.',
			'Deciding what Share and Monitor lead with for each type.',
		],
		dos: [
			'Show the Database tab and the database dump only when the app stores data.',
			'Lead Share with what fits the type: a link and preview card for a landing page, a QR for a counter, an unlisted link for a team’s internal tool.',
			'Keep QR, embed and print under More ways to share, for every type.',
			'Make the estimate name exactly what will exist.',
		],
		donts: [
			'Don’t change the workspace from type to type; only its tabs differ.',
			'Don’t show tabs or numbers for things the app doesn’t have.',
			'Don’t lead with a QR code unless the app is for a physical place.',
		],
		content: [
			'Name the types plainly: “Landing page”, “Web app with data”, “Catalog / shop”, “Internal tool”, “Portfolio / blog”.',
			'Monitor leads with what matters for the type: “Visits · sources · form sends” for a landing page, “Orders · top items” for a shop.',
		],
	},
] satisfies PatternDoc[];
