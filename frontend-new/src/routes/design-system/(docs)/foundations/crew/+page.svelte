<script lang="ts">
	import Mascot from '$lib/ds/agents/Mascot.svelte';
	import { CREW } from '$lib/ds/agents/crew';
	import AgentByline from '$lib/ds/identity/AgentByline.svelte';
	import CrewRow from '$lib/ds/identity/CrewRow.svelte';
	import Status from '$lib/ds/status/Status.svelte';
	import SuggestionRow from '$lib/ds/lists/SuggestionRow.svelte';
	import DocPage from '$lib/docs/ui/DocPage.svelte';
	import DocSection from '$lib/docs/ui/DocSection.svelte';
	import DoDont from '$lib/docs/ui/DoDont.svelte';
	import Example from '$lib/docs/ui/Example.svelte';
	import Guidance from '$lib/docs/ui/Guidance.svelte';
	import Note from '$lib/docs/ui/Note.svelte';
	import SheetEmbed from '$lib/docs/ui/SheetEmbed.svelte';
	import CrewColourSection from '$lib/gallery/CrewColourSection.svelte';
	import exported from '$lib/ds/tokens/tokens.json';
	import { docHref } from '$lib/docs/href';

	// Foundations → Crew & mascots. The crew from crew.ts, colours from tokens.json, where colour may
	// go from AGENTS.md §2 and the sheet's Crew colour section, sizes from Mascot.svelte and where the
	// components use them.
	const tokens = exported.tokens as Record<string, { value?: string | number; light?: string; dark?: string }>;
	const sizes = [
		{ size: 96, use: 'The whole mascot, as the crew board on the Nomi sheet draws it.' },
		{ size: 32, use: 'A member in a list (Crew Row), the first-run greeting, a phone chat row.' },
		{ size: 20, use: 'Bylines, a working status, cards a member sent, agent chips, suggestions, the sidebar mark.' },
		{ size: 16, use: "Who is in a chat, in the phone's nav bar." },
	] as const;
	const code = `<script lang="ts">
	import Mascot from '$lib/ds/agents/Mascot.svelte';
<\/script>

<!-- Next to the name: decorative. -->
<span class="flex items-center gap-2">
	<Mascot agent="koda" size={20} />
	<span class="text-ui font-medium text-text">Koda</span>
</span>

<!-- On its own: give it a label. -->
<Mascot agent="koda" size={32} label="Koda" />`;
</script>

<DocPage
	eyebrow="Foundations"
	title="Crew & mascots"
	lede="Nomi works with a crew of ten. Each member owns one colour and one mascot, and both appear only where that member speaks or works. App chrome stays neutral; mint stays the only action and live colour."
	wide
>
	<DocSection title="On the sheet" lede="The Crew colour section of the Nomi sheet: each member's card in its own tint, with its mascot, its colour and its three tokens.">
		<SheetEmbed sheet="nomi" id="crew-colour" label="Crew colour"><CrewColourSection /></SheetEmbed>
	</DocSection>

	<DocSection title="The ten" lede="In the order the design names them. The colour is the same in both themes; the tint and the ink change with the theme.">
		<div class="overflow-x-auto rounded-md outline -outline-offset-1 outline-border">
			<table class="w-full min-w-200 border-collapse text-left">
				<thead>
					<tr class="border-b border-border bg-surface font-mono text-label tracking-label text-text-3 uppercase">
						<th scope="col" class="w-44 px-4 py-2.5 font-normal">Member</th>
						<th scope="col" class="px-4 py-2.5 font-normal">crew-&lt;id&gt;</th>
						<th scope="col" class="px-4 py-2.5 font-normal">-bg · dark, light</th>
						<th scope="col" class="px-4 py-2.5 font-normal">-ink · dark, light</th>
					</tr>
				</thead>
				<tbody>
					{#each CREW as member (member.id)}
						<tr class="border-b border-border align-middle last:border-b-0">
							<th scope="row" class="px-4 py-2.5 text-left font-normal">
								<span class="flex items-center gap-3">
									<Mascot agent={member.id} size={32} />
									<span class="flex flex-col gap-0.5">
										<span class="text-ui font-medium text-text">{member.name}</span>
										<code class="font-mono text-label text-text-3">{member.id}</code>
									</span>
								</span>
							</th>
							<td class="px-4 py-2.5">
								<span class="flex items-center gap-2 whitespace-nowrap">
									<span class="size-4 shrink-0 rounded-full" style:background="var(--crew-{member.id})"></span>
									<code class="font-mono text-label text-text-2">{tokens[`crew-${member.id}`]?.value}</code>
								</span>
							</td>
							{#each ['bg', 'ink'] as part (part)}
								<td class="px-4 py-2.5">
									<span class="flex items-center gap-2 whitespace-nowrap">
										<span data-theme="dark" class="size-4 shrink-0 rounded-xs outline -outline-offset-1 outline-border-strong" style:background="var(--crew-{member.id}-{part})"></span>
										<span data-theme="light" class="size-4 shrink-0 rounded-xs outline -outline-offset-1 outline-border-strong" style:background="var(--crew-{member.id}-{part})"></span>
										<code class="font-mono text-label text-text-2">{tokens[`crew-${member.id}-${part}`]?.dark} · {tokens[`crew-${member.id}-${part}`]?.light}</code>
									</span>
								</td>
							{/each}
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
		<p>
			Which part of the app each member speaks for is the app's business, not the design system's: the design only gives each one a name, a
			colour and a mascot.
		</p>
	</DocSection>

	<DocSection title="Three tokens each" lede="One colour, one tint to sit on, one ink to write with.">
		<ul>
			<li><code>crew-&lt;id&gt;</code>: the member's colour itself. It fills the mascot and the member's dot, and is the same in dark and light.</li>
			<li><code>crew-&lt;id&gt;-bg</code>: the tint of a surface that belongs to the member, such as their card on the sheet.</li>
			<li><code>crew-&lt;id&gt;-ink</code>: text and marks on that tint. Like the tint, it has a dark and a light value.</li>
		</ul>
		<Example title="A member's card" description="The tint, the colour and the ink together, as the Crew colour cards draw them." align="start">
			<div class="flex w-56 flex-col gap-2.5 rounded-lg bg-crew-tara-bg p-3.5 text-crew-tara-ink">
				<span class="flex items-center gap-2">
					<Mascot agent="tara" size={20} />
					<span class="size-3.5 shrink-0 rounded-full bg-crew-tara"></span>
				</span>
				<span class="text-phone-sm font-semibold">Tara</span>
				<span class="font-mono text-micro">crew-tara · -bg · -ink</span>
			</div>
			<div class="flex w-56 flex-col gap-2.5 rounded-lg bg-crew-kala-bg p-3.5 text-crew-kala-ink">
				<span class="flex items-center gap-2">
					<Mascot agent="kala" size={20} />
					<span class="size-3.5 shrink-0 rounded-full bg-crew-kala"></span>
				</span>
				<span class="text-phone-sm font-semibold">Kala</span>
				<span class="font-mono text-micro">crew-kala · -bg · -ink</span>
			</div>
		</Example>
	</DocSection>

	<DocSection title="Where crew colour goes" lede="Only on surfaces that belong to that member. Everything else stays neutral.">
		<Guidance
			use={[
				'Cards and tiles that are a crew member: the crew header, the selected member. Suggestions and template cards tint only on hover.',
				'Who an item is from: an inbox source tile, today panels, suggestions.',
				'Per-member data: usage bars, breakdowns.',
				'App identity tiles on projects.',
			]}
			avoid={[
				'Buttons, links, focus or live state: that is accent.',
				'Navigation, settings rows, forms and tables.',
				'Errors and warnings: keep danger and warning.',
				'Large page backgrounds inside the app.',
			]}
		/>
		<Example title="Crew colour in place" description="A byline names who spoke, a suggestion shows who it goes to, a crew row lists a member." align="start">
			<div class="flex w-full max-w-120 flex-col gap-3">
				<AgentByline agent="koda" name="Koda" meta="15:37 · 4m 12s" />
				<SuggestionRow agent="dana" label="Summarize this week's spending" route="→ dana" />
				<CrewRow agent="dana" name="Dana" role="Transactions, budgets, subscriptions" state="idle" />
			</div>
		</Example>
	</DocSection>

	<DocSection title="Mascots" lede="One colour and one silhouette per member, flat, with no head accessories. Drawn from the pen's own paths.">
		<Example title="The crew at 96" description="The whole set, at the size the crew board draws it.">
			<div class="grid grid-cols-2 gap-x-6 gap-y-5 sm:grid-cols-5">
				{#each CREW as member (member.id)}
					<span class="flex flex-col items-center gap-3">
						<Mascot agent={member.id} />
						<span class="text-ui text-text-2">{member.name}</span>
					</span>
				{/each}
			</div>
		</Example>

		<h3>Four sizes</h3>
		<p>The component takes only these: <code>96</code> (the default), <code>32</code>, <code>20</code> and <code>16</code>.</p>
		<div class="overflow-x-auto rounded-md outline -outline-offset-1 outline-border">
			<table class="w-full min-w-170 border-collapse text-left">
				<thead>
					<tr class="border-b border-border bg-surface font-mono text-label tracking-label text-text-3 uppercase">
						<th scope="col" class="w-24 px-4 py-2.5 font-normal">Size</th>
						<th scope="col" class="px-4 py-2.5 font-normal">Mascot</th>
						<th scope="col" class="px-4 py-2.5 font-normal">Where</th>
					</tr>
				</thead>
				<tbody>
					{#each sizes as row (row.size)}
						<tr class="border-b border-border align-middle last:border-b-0">
							<th scope="row" class="px-4 py-3 text-left font-mono text-label font-normal text-text">{row.size}px</th>
							<td class="px-4 py-3"><span class="flex items-end gap-3"><Mascot agent="nomi" size={row.size} /><Mascot agent="raga" size={row.size} /></span></td>
							<td class="px-4 py-3 text-ui leading-normal text-text-2">{row.use}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

		<h3>Only where they speak or work</h3>
		<p>
			A mascot stands for a member who is doing something here: answering, asking, working, or listed as part of the crew. A working status
			shows the member's mascot instead of a dot.
		</p>
		<Example title="Working" description="The mascot replaces the status dot while that member works." align="start">
			<Status kind="working" label="Koda is building" agent="koda" />
		</Example>
	</DocSection>

	<DocSection title="In code">
		<p>
			<code>Mascot</code> takes the member's id and a size. Next to the member's name it is decorative and hidden from screen readers; when it
			stands alone, give it a <code>label</code> so it is announced as an image.
		</p>
		<Example title="Decorative and labelled" description="The same mascot beside a name, and alone with a label." {code}>
			<span class="flex items-center gap-2">
				<Mascot agent="koda" size={20} />
				<span class="text-ui font-medium text-text">Koda</span>
			</span>
			<Mascot agent="koda" size={32} label="Koda" />
		</Example>
		<p>
			The colour classes are the tokens' names: <code>bg-crew-koda-bg</code>, <code>text-crew-koda-ink</code>, <code>bg-crew-koda</code>. Write
			each one out in full, as the Crew colour section does for all ten: Tailwind generates only classes it finds written in the markup, so
			<code>bg-crew-{'{id}'}</code> built at run time does nothing.
		</p>
		<Note>
			<p>See also <a href={docHref('/design-system/components/mascot')}>Mascot</a> and <a href={docHref('/design-system/foundations/colour')}>Colour</a>.</p>
		</Note>
	</DocSection>

	<DocSection title="Best practices">
		<DoDont
			dos={[
				"Use the member's mascot when that member speaks or works.",
				"Keep crew colour on what belongs to the member: their card, tile, marker or usage.",
				'Label a mascot that stands alone.',
				'Use the three tokens together: tint, colour, ink.',
			]}
			donts={[
				'Crew colours on buttons, links, focus, live state, errors or page backgrounds.',
				'A mascot as decoration, where no member speaks or works.',
				'Shading, gradients or head accessories on a mascot.',
				'Mascot sizes other than 96, 32, 20 and 16.',
			]}
		/>
	</DocSection>
</DocPage>
