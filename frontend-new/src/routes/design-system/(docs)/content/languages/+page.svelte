<script lang="ts">
	import CodeBlock from '$lib/ds/chat/CodeBlock.svelte';
	import DocPage from '$lib/docs/ui/DocPage.svelte';
	import DocSection from '$lib/docs/ui/DocSection.svelte';
	import DoDont from '$lib/docs/ui/DoDont.svelte';
	import Note from '$lib/docs/ui/Note.svelte';
	import { highlight } from '$lib/docs/highlight';
	import { docHref } from '$lib/docs/href';
	import enFile from '../../../../../../messages/en.json';
	import idFile from '../../../../../../messages/id.json';

	// Content → Languages. English and Bahasa Indonesia, read straight from messages/en.json and
	// messages/id.json so the examples are the shipped strings; AGENTS.md §4 for the rules.
	const en = enFile as Record<string, string>;
	const id = idFile as Record<string, string>;
	const count = Object.keys(en).filter((key) => key !== '$schema').length;

	const groups: { title: string; keys: string[] }[] = [
		{ title: 'Navigation', keys: ['nav_new_chat', 'nav_recents', 'nav_all_chats', 'nav_plan_usage', 'nav_settings', 'settings_back', 'settings_logout'] },
		{ title: 'Actions', keys: ['action_cancel', 'action_retry', 'approval_approve', 'approval_reject', 'diff_apply', 'draft_run', 'build_run', 'ws_publish', 'share_copy', 'export_zip'] },
		{ title: 'The crew at work', keys: ['build_working', 'build_step', 'crew_working', 'handoff', 'plan_note', 'background_note', 'queue_hint'] },
		{ title: 'States and errors', keys: ['msg_sending', 'msg_queued', 'msg_not_sent', 'reply_stopped', 'ws_publish_failed', 'session_title', 'session_body', 'limit_too_long'] },
		{ title: 'Privacy and money', keys: ['share_mask', 'share_mask_help', 'move_shared', 'dislike_include_chat_help', 'export_sub'] },
	];

	const plurals: [one: string, many: string][] = [
		['crew_working_one', 'crew_working'],
		['jump_new_one', 'jump_new'],
		['diff_request_one', 'diff_request'],
		['clarify_answered_one', 'clarify_answered'],
	];

	const kept = ['chat', 'file', 'live', 'deploy', 'Inbox', 'Supervisor', 'sidebar', 'repo', 'commit', 'checkout'];
	const translated: [en: string, id: string][] = [
		['Preview', 'Pratinjau'],
		['Settings', 'Pengaturan'],
		['Share', 'Bagikan'],
		['project', 'proyek'],
		['crew', 'kru'],
		['link', 'tautan'],
		['Plan & usage', 'Paket & pemakaian'],
		['plan (of steps)', 'rencana'],
	];

	const add = `// 1. messages/en.json — at the end
"build_working": "{name} is building"

// 2. messages/id.json — at the end, written at the same time
"build_working": "{name} sedang membangun"

// 3. In the component
import { m } from '$lib/paraglide/messages';
m.build_working({ name: 'Koda' });`;
</script>

{#snippet pairs(keys: string[])}
	<div class="overflow-x-auto rounded-md outline -outline-offset-1 outline-border">
		<table class="w-full min-w-200 border-collapse text-left">
			<thead>
				<tr class="border-b border-border bg-surface font-mono text-label tracking-label text-text-3 uppercase">
					<th scope="col" class="w-52 px-4 py-2.5 font-normal">Key</th>
					<th scope="col" class="px-4 py-2.5 font-normal">English</th>
					<th scope="col" class="px-4 py-2.5 font-normal">Bahasa Indonesia</th>
				</tr>
			</thead>
			<tbody>
				{#each keys as key (key)}
					<tr class="border-b border-border align-top last:border-b-0">
						<th scope="row" class="px-4 py-3 text-left font-mono text-label leading-normal font-normal break-all text-text-3">{key}</th>
						<td lang="en" class="px-4 py-3 text-ui leading-normal text-text">{en[key]}</td>
						<td lang="id" class="px-4 py-3 text-ui leading-normal text-text">{id[key]}</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
{/snippet}

<DocPage
	eyebrow="Content"
	title="Languages"
	lede="Nomi ships in English and Bahasa Indonesia. Every word a person reads is a message in both, written at the same time, and the two files are checked against each other."
	wide
>
	<DocSection title="Two languages, one set of keys">
		<ul>
			<li>English is the base language; Indonesian (<code>id</code>) is the other. The strings live in <code>messages/en.json</code> and <code>messages/id.json</code>, {count} keys in each today.</li>
			<li>Paraglide compiles them into functions: <code>m.nav_new_chat()</code>, <code>m.build_working(&#123; name &#125;)</code>. Components never hold a visible string of their own.</li>
			<li>The language is the one the person saved (a cookie); before they sign in, the browser’s language decides; otherwise English.</li>
			<li>The design-system docs and sheets are documentation and stay in English.</li>
		</ul>
	</DocSection>

	<DocSection title="Side by side" lede="Read from the message files as they ship.">
		{#each groups as group (group.title)}
			<h3>{group.title}</h3>
			{@render pairs(group.keys)}
		{/each}
	</DocSection>

	<DocSection title="How the Indonesian reads">
		<ul>
			<li>It speaks to the person as <strong>kamu</strong> (and <strong>-mu</strong>: “Jawabanmu”, “Namamu”), never “Anda”.</li>
			<li>Crew names, product names and providers never change: Koda, Dana, Google Drive, GitHub, DeepSeek.</li>
			<li>Placeholders move to wherever Indonesian puts them: “{en.build_plan_label}” becomes “{id.build_plan_label}”.</li>
		</ul>
		<div class="grid gap-4 md:grid-cols-2">
			<div class="flex flex-col gap-2.5">
				<h3 class="text-ui font-medium text-text">Kept in English</h3>
				<p class="text-ui leading-normal text-text-2">{kept.join(' · ')}</p>
			</div>
			<div class="flex flex-col gap-2.5">
				<h3 class="text-ui font-medium text-text">Translated</h3>
				<ul class="flex flex-col gap-1">
					{#each translated as [english, indonesian] (english)}
						<li class="text-ui leading-normal text-text-2">{english} → <span class="text-text">{indonesian}</span></li>
					{/each}
				</ul>
			</div>
		</div>
		<Note tone="warning">
			<p>
				Not settled yet: the Publish button is “{id.ws_publish}”, but <code>build_after_publish</code> refers to it as “Publish” (“{id.build_after_publish}”).
				The sheets draw the interface in English only, so terms like this are decided in the message files. Pick one and use it in both places.
			</p>
		</Note>
	</DocSection>

	<DocSection title="One and many" lede="Two keys, the singular ending in _one. The code picks the key.">
		{@render pairs(plurals.flat())}
	</DocSection>

	<DocSection title="Adding a message">
		<ol>
			<li>Add the key at the end of <code>messages/en.json</code> and of <code>messages/id.json</code>, with the Indonesian written at the same time. Keys are snake_case, starting with the area: <code>composer_</code>, <code>nav_</code>, <code>build_</code>, <code>ws_</code>.</li>
			<li>Keep every placeholder in both: <code>&#123;name&#125;</code>, <code>&#123;count&#125;</code>. Values come in as ready-written strings.</li>
			<li>Use it through <code>m</code>. The check and test scripts compile the messages first (<code>npm run i18n</code>).</li>
			<li>Run the unit tests: <code>src/lib/messages.test.ts</code> fails on a key missing from one file, an empty string or a placeholder dropped from the translation.</li>
		</ol>
		<CodeBlock code={add} language="ts" tokens={highlight(add, 'ts')} />
		<p>
			Write the English first by the rules on <a href={docHref('/design-system/content/writing')}>Writing style</a>, with the words on
			<a href={docHref('/design-system/content/vocabulary')}>Vocabulary</a>. Apostrophes are typographic (’) in both files.
		</p>
	</DocSection>

	<DocSection title="Best practices">
		<DoDont
			dos={[
				'Write both languages in the same change.',
				'Keep every placeholder, in the place the language needs it.',
				'Keep crew, product and provider names as they are.',
				'Use two keys for one and many.',
			]}
			donts={[
				'A visible string written straight into a component.',
				'An Indonesian string copied from the English to fill a gap.',
				'A placeholder left out of the translation.',
				'Translating a provider’s quoted reason.',
			]}
		/>
	</DocSection>
</DocPage>
