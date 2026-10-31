<script lang="ts">
	import { ArrowRight, GitBranch } from '@lucide/svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import Status from '$lib/ds/status/Status.svelte';
	import SectionLabel from '$lib/ds/status/SectionLabel.svelte';
	import AgentByline from '$lib/ds/identity/AgentByline.svelte';
	import KVRow from '$lib/ds/conversation/KVRow.svelte';
	import Chip from '$lib/ds/inputs/Chip.svelte';
	import CodeBlock from '$lib/ds/chat/CodeBlock.svelte';
	import DocPage from '$lib/docs/ui/DocPage.svelte';
	import DocSection from '$lib/docs/ui/DocSection.svelte';
	import DoDont from '$lib/docs/ui/DoDont.svelte';
	import Note from '$lib/docs/ui/Note.svelte';
	import css from '$lib/ds/tokens/tokens.css?raw';
	import { docHref } from '$lib/docs/href';

	// Foundations → Dark & light. How tokens.css carries both themes (quoted from the file itself), how
	// the theme is chosen (src/lib/theme.ts, app.html), and what [data-theme] does (tokens.css, app.css).
	const root = css.slice(css.indexOf(':root'), css.indexOf('}'));
	const picked = root.split('\n').filter((line) => /^\s*(color-scheme|--nm-(bg|surface|text|accent)|--crew-koda(-bg)?):/.test(line));
	const themes = [...css.matchAll(/\[data-theme='(light|dark)'\]\s*\{[^}]*\}/g)].map((match) => match[0].replace(/\s+/g, ' '));
	const excerpt = [':root {', ...picked, '\t/* … */', '}', '', ...themes].join('\n');

	const subtree = `<!-- A light preview inside the dark workspace: everything inside follows. -->
<div data-theme="light" class="bg-bg text-text">
	<Button label="Publish" />
</div>`;
</script>

{#snippet sample(theme: 'dark' | 'light')}
	<div data-theme={theme} class="flex min-w-0 flex-1 basis-72 flex-col gap-4 rounded-md bg-bg p-6 outline -outline-offset-1 outline-border">
		<SectionLabel label={theme === 'dark' ? 'Dark' : 'Light'} />
		<AgentByline agent="koda" name="Koda" meta="15:37 · 4m 12s" />
		<KVRow icon={GitBranch} label="Code" value="23 files · main@a3f9c21" />
		<span class="flex flex-wrap items-center gap-3">
			<Status kind="live" label="Live" />
			<Status kind="failed" label="Deploy failed" />
			<Status kind="attention" label="Needs attention" />
		</span>
		<span class="flex flex-wrap items-center gap-2">
			<Button label="Start building" iconEnd={ArrowRight} />
			<Button variant="secondary" label="Add credits" />
			<Chip kind="agent" label="Nomi" agent="nomi" />
		</span>
	</div>
{/snippet}

<DocPage
	eyebrow="Foundations"
	title="Dark & light"
	lede="Nomi has two themes, and neither is an afterthought. Every colour token carries its dark and its light value, so the same class is right in either, and any part of a page can show the other theme."
	wide
>
	<DocSection title="Both, side by side" lede="The same components in a dark and a light subtree. Nothing in them knows which theme it is in.">
		<div class="flex flex-wrap gap-4">
			{@render sample('dark')}
			{@render sample('light')}
		</div>
	</DocSection>

	<DocSection title="Every token has both" lede="Written once, with light-dark(), in the tokens exported from the pen.">
		<p>
			<code>tokens.css</code> gives each colour its light and dark value in one declaration, <code>light-dark(light, dark)</code>. The browser
			picks one by the element's <code>color-scheme</code>: the page says <code>light dark</code> (follow the device), and a
			<code>data-theme</code> attribute narrows it to one. This is the file itself, trimmed:
		</p>
		<CodeBlock code={excerpt} language="css" file="src/lib/ds/tokens/tokens.css" />
		<p>
			A crew member's own colour (<code>crew-koda</code>) is a single value, the same in both themes; its tint and ink change. Every value with
			both themes is on the <a href={docHref('/design-system/tokens')}>Tokens</a> page.
		</p>
	</DocSection>

	<DocSection title="How the theme is chosen" lede="The person's choice: light, dark or system. System is the default.">
		<ul>
			<li><strong>System</strong> follows the device. Nothing is set on the page; the tokens follow the device by themselves.</li>
			<li>
				<strong>Light</strong> or <strong>Dark</strong> is saved with the person's preferences and mirrored in the <code>nm_theme</code> cookie,
				so the server writes <code>data-theme</code> on <code>&lt;html&gt;</code> before the first frame. The page never flashes the other theme
				while it loads.
			</li>
		</ul>
		<p>
			These docs have their own switch in the header (System, Dark, Light), remembered on this device, and the sheets take
			<code>?theme=dark</code> or <code>?theme=light</code>.
		</p>
	</DocSection>

	<DocSection title="A part of the page in the other theme" lede="Any element with data-theme re-themes everything inside it.">
		<p>
			Put <code>data-theme="light"</code> or <code>"dark"</code> on an element and every token inside it resolves for that theme. The workspace
			does this for a light phone preview inside the dark app; the docs do it for the examples' theme switch and the colour swatches.
		</p>
		<CodeBlock code={subtree} language="svelte" />
		<Note>
			<p>
				A <code>data-theme</code> subtree takes the theme's text colour by itself, but not its background. Give it <code>bg-bg</code> (or
				<code>bg-surface</code>) so it doesn't show the page's colour behind the other theme's text.
			</p>
		</Note>
	</DocSection>

	<DocSection title="In code: never dark:" lede="The tokens carry the theme. A class is right in both themes, or it is the wrong class.">
		<p>
			Write <code>bg-surface</code> and <code>text-text-2</code>, and nothing per theme. Tailwind's <code>dark:</code> variant follows only the
			device's setting, so it would ignore the person's choice and every <code>data-theme</code> subtree; a test fails on it. If a colour looks
			wrong in one theme, the token or the role is wrong: fix that, in the pen first.
		</p>
	</DocSection>

	<DocSection title="Check both" lede="Every screen, at 1512 and at 390, in dark and in light.">
		<p>
			Each of the pen's flows has a light twin, and every screen is checked in both before it ships. In these docs, switch the theme in the
			header, or use an example's Page / Dark / Light control to see one part in both.
		</p>
		<DoDont
			dos={[
				'Design and review every screen in dark and light.',
				'Use the token classes; they follow the theme by themselves.',
				'data-theme on a subtree that must show one theme, with its own background.',
			]}
			donts={[
				'The dark: variant.',
				'Raw hex values, or a colour that only works in one theme.',
				'Pure #000 or #FFF surfaces.',
			]}
		/>
	</DocSection>
</DocPage>
