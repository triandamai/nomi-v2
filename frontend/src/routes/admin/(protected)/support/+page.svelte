<script lang="ts">
	// Admin → Support: Nomi answering customers of other products from the knowledge base.
	// Questions it couldn't answer arrive as drafts to turn into articles; keys let products call it.
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { page } from '$app/state';
	import Button from '$lib/components/m3/Button.svelte';
	import Dialog from '$lib/components/m3/Dialog.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import IconPlus from '$lib/components/icons/IconPlus.svelte';
	import IconCopy from '$lib/components/icons/IconCopy.svelte';
	import { adminApi } from '$lib/adminApi';

	type Tab = 'drafts' | 'articles' | 'keys' | 'questions';
	interface Article { id: string; title: string; body: string; tags: string[]; published: boolean; updated_at: string }
	interface Draft { id: string; question: string; title: string; body: string; times_asked: number; last_asked_at: string }
	interface Key { id: string; name: string; prefix: string; product_name: string; created_at: string; last_used_at: string | null; revoked_at: string | null }
	interface Question { id: string; question: string; answer: string; answered: boolean; customer: string | null; key_name: string | null; created_at: string }

	let tab = $state<Tab>('drafts');
	let drafts = $state<Draft[] | null>(null);
	let articles = $state<Article[] | null>(null);
	let keys = $state<Key[] | null>(null);
	let questions = $state<Question[] | null>(null);
	let error = $state<string | null>(null);
	let search = $state('');

	const when = (at: string) => new Intl.DateTimeFormat(getLocale(), { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(at));

	async function load(which: Tab) {
		error = null;
		try {
			if (which === 'drafts') drafts = await adminApi<Draft[]>('support/drafts');
			if (which === 'articles') articles = await adminApi<Article[]>(`support/articles${search.trim() ? `?q=${encodeURIComponent(search.trim())}` : ''}`);
			if (which === 'keys') keys = await adminApi<Key[]>('support/keys');
			if (which === 'questions') questions = await adminApi<Question[]>('support/questions');
		} catch (e) {
			error = (e as Error).message;
		}
	}
	$effect(() => {
		void load(tab);
	});

	// The article editor: a new article, an existing one, or a draft being published.
	let editorOpen = $state(false);
	let editing = $state<{ kind: 'new' } | { kind: 'article'; id: string } | { kind: 'draft'; id: string; question: string }>({ kind: 'new' });
	let title = $state('');
	let body = $state('');
	let tags = $state('');
	let published = $state(true);
	let saving = $state(false);

	function openEditor(target: typeof editing, values: { title?: string; body?: string; tags?: string[]; published?: boolean } = {}) {
		editing = target;
		title = values.title ?? '';
		body = values.body ?? '';
		tags = (values.tags ?? []).join(', ');
		published = values.published ?? true;
		editorOpen = true;
	}

	async function save(event: SubmitEvent) {
		event.preventDefault();
		saving = true;
		error = null;
		const input = { title: title.trim(), body, tags: tags.split(',').map((t) => t.trim()).filter(Boolean), published };
		try {
			if (editing.kind === 'new') await adminApi('support/articles', { method: 'POST', body: input });
			if (editing.kind === 'article') await adminApi(`support/articles/${editing.id}`, { method: 'PUT', body: input });
			if (editing.kind === 'draft') await adminApi(`support/drafts/${editing.id}/publish`, { method: 'POST', body: input });
			editorOpen = false;
			await load(tab);
		} catch (e) {
			error = (e as Error).message;
		} finally {
			saving = false;
		}
	}

	async function dismiss(draft: Draft) {
		await adminApi(`support/drafts/${draft.id}/dismiss`, { method: 'POST' }).catch((e) => (error = (e as Error).message));
		await load('drafts');
	}

	async function removeArticle(article: Article) {
		if (!confirm(m.support_delete_confirm({ title: article.title }))) return;
		await adminApi(`support/articles/${article.id}`, { method: 'DELETE' }).catch((e) => (error = (e as Error).message));
		await load('articles');
	}

	// Keys: the secret is shown once, right after it's made.
	let keyOpen = $state(false);
	let keyName = $state('');
	let productName = $state('');
	let newSecret = $state<string | null>(null);
	let copied = $state(false);

	async function createKey(event: SubmitEvent) {
		event.preventDefault();
		try {
			const created = await adminApi<{ secret: string }>('support/keys', { method: 'POST', body: { name: keyName, product_name: productName } });
			newSecret = created.secret;
			keyName = '';
			productName = '';
			await load('keys');
		} catch (e) {
			error = (e as Error).message;
		}
	}

	async function revoke(key: Key) {
		if (!confirm(m.support_revoke_confirm({ name: key.name }))) return;
		await adminApi(`support/keys/${key.id}`, { method: 'DELETE' }).catch((e) => (error = (e as Error).message));
		await load('keys');
	}

	async function copy(text: string) {
		await navigator.clipboard.writeText(text).catch(() => {});
		copied = true;
		setTimeout(() => (copied = false), 1500);
	}

	const endpoint = $derived(`${page.url.origin.replace(/\/$/, '')}/api/support/v1/ask`);
	const example = $derived(
		`curl -X POST ${endpoint} \\\n  -H "Authorization: Bearer nsk_..." \\\n  -H "Content-Type: application/json" \\\n  -d '{"question": "How do I get a refund?", "customer": "user-42"}'`,
	);

	const TABS: { id: Tab; label: () => string }[] = [
		{ id: 'drafts', label: m.support_tab_drafts },
		{ id: 'articles', label: m.support_tab_articles },
		{ id: 'keys', label: m.support_tab_keys },
		{ id: 'questions', label: m.support_tab_questions },
	];
</script>

<PageHeader title={m.admin_support()} lede={m.support_lede()} agent="nomi" />

<div class="tabs" role="tablist" aria-label={m.admin_support()}>
	{#each TABS as t (t.id)}
		<button type="button" role="tab" class="tab" aria-selected={tab === t.id} onclick={() => (tab = t.id)}>
			{t.label()}
			{#if t.id === 'drafts' && drafts && drafts.length > 0}<span class="tab__count">{drafts.length}</span>{/if}
		</button>
	{/each}
</div>

{#if error}<p class="error" role="alert">{error}</p>{/if}

{#if tab === 'drafts'}
	<p class="hint">{m.support_drafts_hint()}</p>
	{#if drafts && drafts.length === 0}
		<p class="empty">{m.support_drafts_empty()}</p>
	{:else if drafts}
		<ul class="list">
			{#each drafts as draft (draft.id)}
				<li class="row">
					<div class="row__text">
						<span class="row__title">{draft.question}</span>
						<span class="row__meta">{m.support_asked({ count: draft.times_asked, when: when(draft.last_asked_at) })}</span>
						{#if draft.title}<span class="row__sub">{m.support_suggested({ title: draft.title })}</span>{/if}
					</div>
					<div class="row__actions">
						<Button variant="tonal" size="s" onclick={() => openEditor({ kind: 'draft', id: draft.id, question: draft.question }, { title: draft.title, body: draft.body })}>
							{m.support_write_article()}
						</Button>
						<Button variant="text" size="s" onclick={() => dismiss(draft)}>{m.support_dismiss()}</Button>
					</div>
				</li>
			{/each}
		</ul>
	{/if}
{:else if tab === 'articles'}
	<div class="toolbar">
		<form class="search" onsubmit={(e) => { e.preventDefault(); void load('articles'); }}>
			<TextField name="q" label={m.support_search()} bind:value={search} />
		</form>
		<Button variant="filled" size="s" onclick={() => openEditor({ kind: 'new' })}><IconPlus size={18} />{m.support_new_article()}</Button>
	</div>
	{#if articles && articles.length === 0}
		<p class="empty">{m.support_articles_empty()}</p>
	{:else if articles}
		<ul class="list">
			{#each articles as article (article.id)}
				<li class="row">
					<div class="row__text">
						<span class="row__title">{article.title}{#if !article.published}<span class="tag">{m.support_unpublished()}</span>{/if}</span>
						<span class="row__meta">{article.tags.join(', ')}{article.tags.length ? ' · ' : ''}{when(article.updated_at)}</span>
					</div>
					<div class="row__actions">
						<Button variant="text" size="s" onclick={() => openEditor({ kind: 'article', id: article.id }, article)}>{m.support_edit()}</Button>
						<Button variant="text" size="s" class="danger" onclick={() => removeArticle(article)}>{m.support_delete()}</Button>
					</div>
				</li>
			{/each}
		</ul>
	{/if}
{:else if tab === 'keys'}
	<section class="panel">
		<h2 class="panel__title">{m.support_connect_title()}</h2>
		<p class="hint">{m.support_connect_hint()}</p>
		<pre class="code">{example}</pre>
		<form class="key-form" onsubmit={createKey}>
			<TextField name="name" label={m.support_key_name()} bind:value={keyName} required maxlength={80} />
			<TextField name="product" label={m.support_product_name()} bind:value={productName} maxlength={80} />
			<Button type="submit" variant="filled" size="s" disabled={!keyName.trim()}>{m.support_create_key()}</Button>
		</form>
		{#if newSecret}
			<div class="secret" role="status">
				<p>{m.support_key_once()}</p>
				<code>{newSecret}</code>
				<Button variant="tonal" size="s" onclick={() => newSecret && copy(newSecret)}><IconCopy size={16} />{copied ? m.pj_copied() : m.pj_copy()}</Button>
			</div>
		{/if}
	</section>
	{#if keys && keys.length > 0}
		<ul class="list">
			{#each keys as key (key.id)}
				<li class="row" class:row--off={key.revoked_at}>
					<div class="row__text">
						<span class="row__title">{key.name}{#if key.revoked_at}<span class="tag">{m.support_revoked()}</span>{/if}</span>
						<span class="row__meta">{key.prefix}… · {key.product_name || '—'} · {key.last_used_at ? m.support_last_used({ when: when(key.last_used_at) }) : m.support_never_used()}</span>
					</div>
					{#if !key.revoked_at}
						<div class="row__actions"><Button variant="text" size="s" class="danger" onclick={() => revoke(key)}>{m.support_revoke()}</Button></div>
					{/if}
				</li>
			{/each}
		</ul>
	{/if}
{:else if questions}
	{#if questions.length === 0}
		<p class="empty">{m.support_questions_empty()}</p>
	{:else}
		<ul class="list">
			{#each questions as q (q.id)}
				<li class="row row--stack">
					<span class="row__title">{q.question}<span class="tag" class:tag--miss={!q.answered}>{q.answered ? m.support_answered() : m.support_unanswered()}</span></span>
					<span class="row__answer">{q.answer}</span>
					<span class="row__meta">{q.customer ?? '—'} · {q.key_name ?? '—'} · {when(q.created_at)}</span>
				</li>
			{/each}
		</ul>
	{/if}
{/if}

<Dialog bind:open={editorOpen} headline={editing.kind === 'new' ? m.support_new_article() : editing.kind === 'draft' ? m.support_write_article() : m.support_edit()}>
	<form id="article-form" class="editor" onsubmit={save}>
		{#if editing.kind === 'draft'}<p class="hint">{m.support_from_question({ question: editing.question })}</p>{/if}
		<TextField name="title" label={m.support_article_title()} bind:value={title} required maxlength={200} />
		<label class="area">
			<span class="area__label">{m.support_article_body()}</span>
			<textarea rows="10" bind:value={body}></textarea>
		</label>
		<TextField name="tags" label={m.support_article_tags()} bind:value={tags} />
		<label class="check"><input type="checkbox" bind:checked={published} />{m.support_published()}</label>
	</form>
	{#snippet actions()}
		<Button variant="text" type="button" onclick={() => (editorOpen = false)}>{m.issues_cancel()}</Button>
		<Button variant="filled" type="submit" form="article-form" disabled={saving || !title.trim()}>{m.support_save()}</Button>
	{/snippet}
</Dialog>

<style>
	.tabs {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
		margin-bottom: 14px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.tab {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		min-height: 44px;
		padding: 0 14px;
		border: none;
		border-bottom: 2px solid transparent;
		background: none;
		color: var(--md-sys-color-on-surface-variant);
		font: inherit;
		font-weight: 600;
		cursor: pointer;
	}
	.tab[aria-selected='true'] {
		border-bottom-color: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-surface);
	}
	.tab:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: -2px;
	}
	.tab__count {
		min-width: 18px;
		padding: 0 5px;
		border-radius: 4px;
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
		font-size: 0.75rem;
		text-align: center;
	}
	.hint,
	.empty {
		margin: 0 0 12px;
		color: var(--md-sys-color-on-surface-variant);
	}
	.error {
		margin: 0 0 12px;
		color: var(--md-sys-color-error);
	}
	.toolbar {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		margin-bottom: 12px;
	}
	.search {
		flex: 1;
		max-width: 420px;
	}
	.list {
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.row {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px 16px;
		padding: 12px 4px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.row--stack {
		flex-direction: column;
		align-items: flex-start;
	}
	.row--off {
		opacity: 0.6;
	}
	.row__text {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.row__title {
		font-weight: 600;
		overflow-wrap: anywhere;
	}
	.row__meta,
	.row__sub {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.row__answer {
		max-width: 75ch;
		color: var(--md-sys-color-on-surface);
		font-size: 0.875rem;
		white-space: pre-wrap;
	}
	.row__actions {
		display: flex;
		gap: 4px;
	}
	.row__actions :global(.danger) {
		color: var(--md-sys-color-error);
	}
	.tag {
		margin-left: 8px;
		padding: 1px 6px;
		border-radius: 4px;
		background: var(--md-sys-color-surface-container-highest);
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.6875rem;
		font-weight: 600;
		vertical-align: 2px;
	}
	.tag--miss {
		background: var(--md-sys-color-tertiary-container);
		color: var(--md-sys-color-on-tertiary-container);
	}
	.panel {
		display: flex;
		flex-direction: column;
		gap: 10px;
		margin-bottom: 16px;
		padding: 16px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-low);
	}
	.panel__title {
		margin: 0;
		font-size: 1rem;
		font-weight: 700;
	}
	.code {
		margin: 0;
		padding: 12px;
		overflow-x: auto;
		border-radius: 8px;
		background: var(--md-sys-color-surface-container-highest);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
	}
	.key-form {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
		align-items: center;
		gap: 10px;
	}
	.secret {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px 12px;
		padding: 12px;
		border-radius: 8px;
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.secret p {
		flex-basis: 100%;
		margin: 0;
		font-size: 0.875rem;
	}
	.secret code {
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.8125rem;
		overflow-wrap: anywhere;
	}
	.editor {
		display: flex;
		flex-direction: column;
		gap: 12px;
		min-width: min(560px, 80vw);
	}
	.area {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.area__label {
		color: var(--md-sys-color-on-surface-variant);
		font-size: var(--md-sys-typescale-body-small-size);
	}
	.area textarea {
		box-sizing: border-box;
		width: 100%;
		padding: 12px;
		border: 1.5px solid var(--md-sys-color-outline);
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface);
		color: var(--md-sys-color-on-surface);
		font: inherit;
		resize: vertical;
	}
	.area textarea:focus {
		outline: none;
		border-color: var(--md-sys-color-primary);
	}
	.check {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		min-height: 44px;
	}
	.check input {
		accent-color: var(--md-sys-color-primary);
	}
</style>
