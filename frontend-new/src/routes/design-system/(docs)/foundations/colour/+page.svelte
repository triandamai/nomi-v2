<script lang="ts">
	import Button from '$lib/ds/actions/Button.svelte';
	import DocPage from '$lib/docs/ui/DocPage.svelte';
	import DocSection from '$lib/docs/ui/DocSection.svelte';
	import DoDont from '$lib/docs/ui/DoDont.svelte';
	import Example from '$lib/docs/ui/Example.svelte';
	import Note from '$lib/docs/ui/Note.svelte';
	import SheetEmbed from '$lib/docs/ui/SheetEmbed.svelte';
	import ColourSection from '$lib/gallery/ColourSection.svelte';
	import exported from '$lib/ds/tokens/tokens.json';
	import { docHref } from '$lib/docs/href';

	// Foundations → Colour. Roles from AGENTS.md §2, values from tokens.json, the sheet's own section.
	const roles: [tokens: string[], role: string][] = [
		[['bg'], 'The page. Never pure black or white.'],
		[['surface', 'surface-2'], 'Raised areas: cards, headers, chips. surface-2 for fills inside them, hover and tracks.'],
		[['border', 'border-strong'], 'Dividers and quiet outlines; inputs, buttons and emphasis.'],
		[['text', 'text-2', 'text-3'], 'Primary text; secondary (body copy, icons); tertiary (meta, captions, placeholders).'],
		[['accent', 'accent-ink', 'accent-bg'], 'The one action and live state; text on accent; the quiet accent tint.'],
		[['danger', 'danger-bg', 'danger-border'], 'Errors and destructive actions.'],
		[['warning', 'warning-bg'], 'Needs attention, waiting, interrupted.'],
		[['focus'], 'The focus ring and the field glow.'],
	];
	const tokens = exported.tokens as Record<string, { light?: string; dark?: string }>;
</script>

<DocPage
	eyebrow="Foundations"
	title="Colour"
	lede="Neutral surfaces do the hierarchy; colour is rare and always means something. One mint accent marks the single thing to act on. Every colour is a token, themed for dark and light."
	wide
>
	<DocSection title="On the sheet" lede="The Colour section of the Nomi sheet, drawn from the same tokens the app uses.">
		<SheetEmbed sheet="nomi" id="colour" label="Colour"><ColourSection /></SheetEmbed>
	</DocSection>

	<DocSection title="Roles" lede="Pick a colour by what it does, not by how it looks. The class is the token's name without nm-.">
		<div class="overflow-x-auto rounded-md outline -outline-offset-1 outline-border">
			<table class="w-full min-w-170 border-collapse text-left">
				<thead>
					<tr class="border-b border-border bg-surface font-mono text-label tracking-label text-text-3 uppercase">
						<th scope="col" class="w-96 px-4 py-2.5 font-normal">Tokens</th>
						<th scope="col" class="px-4 py-2.5 font-normal">Role</th>
					</tr>
				</thead>
				<tbody>
					{#each roles as [names, role] (names[0])}
						<tr class="border-b border-border align-top last:border-b-0">
							<th scope="row" class="px-4 py-3 text-left font-normal">
								<span class="flex flex-col gap-2">
									{#each names as name (name)}
										<span class="flex items-center gap-2 whitespace-nowrap">
											<span data-theme="dark" class="size-4 shrink-0 rounded-xs outline -outline-offset-1 outline-border-strong" style:background="var(--nm-{name})"></span>
											<span data-theme="light" class="size-4 shrink-0 rounded-xs outline -outline-offset-1 outline-border-strong" style:background="var(--nm-{name})"></span>
											<code class="font-mono text-caption text-text">{name}</code>
											<span class="font-mono text-label text-text-3">{tokens[`nm-${name}`]?.dark} · {tokens[`nm-${name}`]?.light}</span>
										</span>
									{/each}
								</span>
							</th>
							<td class="px-4 py-3 text-ui leading-normal text-text-2">{role}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
		<p>Every value with its class is on the <a href={docHref('/design-system/tokens')}>Tokens</a> page.</p>
	</DocSection>

	<DocSection title="One accent, under 10%" lede="Mint is for the single thing to act on: the primary button, the live dot, the chosen option.">
		<p>
			Everything else is neutral. Inside a panel the main action is <strong>inverse</strong> (a text-coloured fill), so mint stays rare even
			on a busy screen. If two things on a screen are mint, one of them is wrong.
		</p>
		<Example title="The page's action, and a panel's" description="Primary once per view; inverse for the main action inside a panel.">
			<Button label="Publish" />
			<Button variant="inverse" label="Save changes" />
			<Button variant="secondary" label="Cancel" />
		</Example>
	</DocSection>

	<DocSection title="The crew's colours" lede="Each crew member owns a colour, used only on surfaces that belong to them.">
		<p>
			Their cards, tiles, who-it's-from markers and their usage take <code>crew-&lt;member&gt;</code>. Never buttons, links, focus, live state,
			errors or page backgrounds. See <a href={docHref('/design-system/foundations/crew')}>Crew &amp; mascots</a>.
		</p>
	</DocSection>

	<DocSection title="In code">
		<p>
			Use the classes: <code>bg-surface</code>, <code>text-text-2</code>, <code>border-border-strong</code>. Tailwind knows only the design's
			colours, so <code>text-red-500</code> doesn't exist, and a test fails on raw hex values, arbitrary values and the <code>dark:</code>
			variant. The theme is carried by the tokens themselves: a class is right in both themes.
		</p>
		<Note><p>A part of the page that must show one theme (a preview, a swatch) takes <code>data-theme="dark"</code> or <code>"light"</code>; everything inside follows it.</p></Note>
	</DocSection>

	<DocSection title="Best practices">
		<DoDont
			dos={[
				'Let neutrals do the hierarchy: bg, then surface, then surface-2.',
				'Keep mint for the one thing to act on, or what is live right now.',
				'Pair danger with words that say what happened; colour alone is never the message.',
				'Check every screen in dark and light.',
			]}
			donts={[
				'Gradients, glass, glows or a second accent hue.',
				'Pure #000 or #FFF surfaces.',
				'Crew colours on buttons, links, focus, errors or backgrounds.',
				'Raw hex values or the dark: variant in code.',
			]}
		/>
	</DocSection>
</DocPage>
