<script lang="ts">
	import Mascot from '$lib/ds/agents/Mascot.svelte';
	import { CREW, type CrewId } from '$lib/ds/agents/crew';
	import DocPage from '$lib/docs/ui/DocPage.svelte';
	import DocSection from '$lib/docs/ui/DocSection.svelte';
	import DoDont from '$lib/docs/ui/DoDont.svelte';
	import LinkCard from '$lib/docs/ui/LinkCard.svelte';
	import Note from '$lib/docs/ui/Note.svelte';
	import { guides } from '$lib/docs/nav';
	import { PRINCIPLES } from '$lib/docs/principles';
	import { docHref } from '$lib/docs/href';
	import exported from '$lib/ds/tokens/tokens.json';

	// Get started → For content & marketing. The character and principles (AGENTS.md §2, the Nomi
	// sheet's header), the crew as the sheets show them, the brand colour as the Colour sheet lists
	// it, and what the pen doesn't define yet.
	const content = guides[2].links;
	const tokens = exported.tokens as Record<string, { light?: string; dark?: string; value?: string | number }>;

	// What each member does, only where a sheet shows them doing it.
	const roles: Record<CrewId, string | undefined> = {
		nomi: 'The one you talk to first (“Ask Nomi, or hand something to the crew…”). Answers, drafts, and hands work to the member who owns it: “Nomi handed this to Dana”.',
		koda: 'Builds apps: shows the plan, writes the files, runs the tests and puts the app online. “Koda is building.”',
		dana: 'Money: transactions, budgets, subscriptions. Only records; never signs in to a bank.',
		kala: 'Reminders on time. “Kala pings you in Inbox and on your phone.”',
		rena: undefined,
		tara: 'Works through the tools you connect: sends an email from your Gmail once you approve it, shares from your Google Drive, checks your Google Calendar.',
		maya: 'Reads attachments: statements, receipts, PDFs.',
		raga: undefined,
		personality: undefined,
		supervisor: 'Stops the crew’s running tasks and says what finished, what stopped and what never started. “Supervisor stopped 3 tasks.”',
	};

	const swatches = ['nm-accent', 'nm-accent-ink', 'nm-bg', 'nm-surface', 'nm-text'] as const;

	const undefinedYet = [
		'A logo: its construction, clear space, minimum size, or versions for light and dark backgrounds.',
		'Colours for marketing beyond the app’s tokens, or a palette for campaigns.',
		'Type for campaigns: headline sizes, or anything beyond the app’s Geist and Geist Mono.',
		'Photography, illustration and motion. In the app, emoji and illustrations stay out of the chrome.',
		'How the mascots may be used outside the product: size, poses, backgrounds, merchandise.',
		'Templates for social, email campaigns, slides or the website.',
	];
</script>

<DocPage
	eyebrow="Get started"
	title="For content & marketing"
	lede="How Nomi looks and sounds, who the crew are, and the colour that marks Nomi. Everything here comes from the pen and the product’s own words; what isn’t defined yet is listed at the end."
	wide
>
	<DocSection title="The character" lede="“An instrument, not a toy.”">
		<p>
			Calm, dense, precise. Nomi should feel like a well-made tool for people’s money, plans and projects, not a playful chat toy. Neutral
			surfaces do the hierarchy; colour is rare and always means something. The Nomi sheet opens with one line: <strong>neutral surfaces,
			one accent, mono for anything a machine produced.</strong>
		</p>
	</DocSection>

	<DocSection title="The principles" lede="Five rules every screen follows, word for word from the top of the Nomi sheet.">
		<ol class="flex flex-col overflow-hidden rounded-md border border-border">
			{#each PRINCIPLES as [title, body], index (title)}
				<li class="flex gap-4 border-b border-border px-4 py-3.5 last:border-b-0">
					<span class="font-mono text-label leading-firm text-text">0{index + 1}</span>
					<span class="flex min-w-0 flex-1 flex-col gap-0.5">
						<span class="text-ui font-medium text-text">{title}</span>
						<span class="text-caption text-text-3">{body}</span>
					</span>
				</li>
			{/each}
		</ol>
	</DocSection>

	<DocSection title="How Nomi sounds">
		<p>
			Plain, specific, sentence case. Nomi names the real thing (“Koda is building”, “deploy #2”), says what happened when something fails,
			and never calls itself “magic”, “effortless” or “AI-powered”. Every word ships in English and Bahasa Indonesia.
		</p>
		<div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
			{#each content as page (page.path)}
				<LinkCard href={docHref(page.path)} title={page.title} blurb={page.blurb} />
			{/each}
		</div>
	</DocSection>

	<DocSection title="The crew" lede="Nomi and nine others. Each has a name, a colour and a mascot.">
		<div class="overflow-x-auto rounded-md outline -outline-offset-1 outline-border">
			<table class="w-full min-w-170 border-collapse text-left">
				<thead>
					<tr class="border-b border-border bg-surface font-mono text-label tracking-label text-text-3 uppercase">
						<th scope="col" class="w-56 px-4 py-2.5 font-normal">Member</th>
						<th scope="col" class="px-4 py-2.5 font-normal">What the sheets show them doing</th>
					</tr>
				</thead>
				<tbody>
					{#each CREW as member (member.id)}
						<tr class="border-b border-border align-top last:border-b-0">
							<th scope="row" class="px-4 py-3 text-left font-normal">
								<span class="flex items-center gap-2.5">
									<Mascot agent={member.id} size={32} />
									<span class="flex flex-col">
										<span class="text-ui font-medium text-text">{member.name}</span>
										<code class="font-mono text-label text-text-3">crew-{member.id}</code>
									</span>
								</span>
							</th>
							<td class="px-4 py-3 text-ui leading-normal text-text-2">
								{#if roles[member.id]}{roles[member.id]}{:else}<span class="text-text-3">Named, with a colour and a mascot. What {member.name} does isn’t drawn in the sheets yet.</span>{/if}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
		<h3>Mascots are identity, not decoration</h3>
		<ul>
			<li>One per crew member: one colour and one silhouette, flat, no head accessories.</li>
			<li>Used only where that member speaks or works: their byline, their card, their running task.</li>
			<li>Drawn at 96, 32, 20 and 16, from the pen’s own shapes.</li>
			<li>A member’s colour (<code>crew-&lt;member&gt;</code>) belongs only on surfaces that are theirs: their cards, tiles, who-it’s-from markers, their usage. Never on buttons, links, focus, live state, errors or page backgrounds.</li>
		</ul>
		<p>Every member’s colours and sizes are on <a href={docHref('/design-system/foundations/crew')}>Crew &amp; mascots</a>.</p>
	</DocSection>

	<DocSection title="Nomi’s colour" lede="One accent: mint, nm-accent.">
		<p>
			The Colour sheet lists the accent’s uses as <strong>send, live dot, logo</strong>. In the app it marks the single thing to act on and
			what is live right now, and stays under 10% of a view. Everything else is neutral, and the page is never pure black or pure white.
		</p>
		<div class="grid gap-3 sm:grid-cols-2">
			{#each ['dark', 'light'] as theme (theme)}
				<div data-theme={theme} class="flex flex-col gap-3 rounded-md bg-bg p-4 outline -outline-offset-1 outline-border">
					<span class="font-mono text-label tracking-label text-text-3 uppercase">{theme}</span>
					<div class="flex flex-col gap-2">
						{#each swatches as name (name)}
							<span class="flex items-center gap-3">
								<span class="size-8 shrink-0 rounded-sm outline -outline-offset-1 outline-border-strong" style:background="var(--{name})"></span>
								<code class="font-mono text-caption text-text">{name}</code>
								<span class="ml-auto font-mono text-label text-text-3">{tokens[name]?.[theme as 'dark' | 'light']}</span>
							</span>
						{/each}
					</div>
				</div>
			{/each}
		</div>
		<p>
			The only lockup the pen draws is the app sidebar’s wordmark: the Nomi mascot at 20px beside <strong>nomi</strong>, lowercase, in Geist
			at weight 600. The mascot there is Nomi’s crew colour (<code>crew-nomi</code>, {tokens['crew-nomi']?.value}), not the accent. Values
			and every other colour are on <a href={docHref('/design-system/foundations/colour')}>Colour</a>.
		</p>
	</DocSection>

	<DocSection title="Type">
		<p>
			Geist for everything people read, Geist Mono for anything a machine produced: paths, IDs, costs, timestamps, model names. Weights 400,
			500 and 600 only; sentence case everywhere, with capitals only in the small mono section labels. See
			<a href={docHref('/design-system/foundations/typography')}>Typography</a>.
		</p>
	</DocSection>

	<DocSection title="Example content" lede="The sheets tell one consistent story. Reuse its names rather than inventing new ones.">
		<p>
			The person is <strong>Mario Prasetyo</strong>; his teammate is Sari Wulandari. Koda builds <code>warung-kopi-menu</code>, a menu and
			pre-order page for Warung Kopi Bu Siti, started at 08:13 and live as deploy #1 at 08:18. In chat, Dana tracks October’s food spending
			(Rp 2,1 jt, up 20% from September), Kala reminds Mario to pay kos rent to Bu Sri, and the crew plans a trip to Bandung with Sari. The
			Build sheet’s <a href={docHref('/design-system/patterns/build/story-sheet')}>Story sheet</a> lists every value the build flows use.
		</p>
	</DocSection>

	<DocSection title="Not defined yet" lede="The pen is a product design. It doesn’t yet define anything for use outside the app.">
		<ul>
			{#each undefinedYet as item (item)}<li>{item}</li>{/each}
		</ul>
		<Note tone="warning">
			<p>Don’t invent these from the app’s parts. Anything new is drawn in Nomi.pen first, like every other part of the design system.</p>
		</Note>
	</DocSection>

	<DocSection title="Best practices">
		<DoDont
			dos={[
				'Say what Nomi does in plain words: it builds real web apps from a description; you get a live link, and the code is yours to export.',
				'Name the crew member who does the work.',
				'Use mint for the one thing to look at or act on.',
				'Use the story’s names and numbers for examples.',
			]}
			donts={[
				'“Magic”, “effortless” or “AI-powered”.',
				'A mascot as decoration, or a mascot for a member who isn’t speaking or working.',
				'Gradients, glass, glows or a second accent hue.',
				'Title Case or ALL CAPS headlines.',
			]}
		/>
	</DocSection>
</DocPage>
