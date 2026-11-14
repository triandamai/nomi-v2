<script lang="ts">
	import { Plus } from '@lucide/svelte';
	import { enhance } from '$app/forms';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { can } from '$lib/admin/access';
	import type { Article, SupportKey } from '$lib/admin/api';
	import { dateTime, full } from '$lib/admin/format';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import FormSection from '$lib/ds/admin/FormSection.svelte';
	import FilterBar from '$lib/ds/admin/FilterBar.svelte';
	import AdminTable, { type TableColumn } from '$lib/ds/admin/AdminTable.svelte';
	import TextArea from '$lib/ds/admin/TextArea.svelte';
	import ConfirmDialog from '$lib/ds/admin/ConfirmDialog.svelte';
	import SecretReveal from '$lib/ds/admin/SecretReveal.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import Tag from '$lib/ds/status/Tag.svelte';
	import SectionLabel from '$lib/ds/status/SectionLabel.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';
	import Outcome from '$lib/admin/ui/Outcome.svelte';

	// Support (support: view): the drafts Koda wrote from questions the articles couldn't answer
	// (write: publish with edits, or dismiss), the help articles, searched in the address (write
	// adds and changes them), the questions asked, and the keys products ask with (manage: make
	// one, its secret shown once, or revoke).
	let { data, form } = $props();

	const language = getLocale();
	const canWrite = $derived(can(data.access, 'support', 'write'));
	const canManage = $derived(can(data.access, 'support', 'manage'));

	const articleColumns: TableColumn<Article>[] = [
		{ key: 'title', label: m.admin_col_title(), sort: (row) => row.title },
		{ key: 'tags', label: m.admin_col_tags(), phone: false },
		{ key: 'status', label: m.admin_col_status() },
		{ key: 'updated', label: m.admin_col_updated(), sort: (row) => row.updated_at, phone: false },
	];

	const made = $derived(form?.action === 'createKey' && form.ok ? (form.data as SupportKey & { secret: string }) : null);
	let revealed = $state<string | null>(null);
	$effect(() => {
		if (made) revealed = made.secret;
	});

	let revoking = $state<SupportKey | null>(null);
	let confirmOpen = $state(false);
	let revokeForm: HTMLFormElement | undefined = $state();
</script>

<PageHeader title={m.admin_page_support()} intro={m.admin_support_intro()} />

<section aria-label={m.admin_support_drafts()} class="flex flex-col gap-3">
	<SectionLabel label={m.admin_support_drafts()} count={data.drafts.ok ? data.drafts.data.length : undefined} />
	<Outcome {form} action={['publishDraft', 'dismissDraft']} done={form?.action === 'publishDraft' ? m.admin_draft_published() : m.admin_draft_dismissed()} failed={m.admin_draft_failed()} />
	{#if data.drafts.ok}
		{#if data.drafts.data.length === 0}
			<p class="text-ui text-text-3">{m.admin_support_drafts_empty()}</p>
		{/if}
		{#each data.drafts.data as draft (draft.id)}
			<FormSection title={draft.question} description={m.admin_draft_asked({ count: full(draft.times_asked, language), when: dateTime(draft.last_asked_at, language) })}>
				{#if canWrite}
					<form method="POST" action="?/publishDraft" use:enhance class="flex flex-col gap-4">
						<input type="hidden" name="id" value={draft.id} />
						<TextField label={m.admin_draft_title()} name="title" value={draft.title} required autocomplete="off" />
						<TextArea label={m.admin_draft_body()} name="body" value={draft.body} />
						<TextField label={m.admin_draft_tags()} name="tags" helper={m.admin_draft_tags_helper()} autocomplete="off" />
						<div class="flex flex-wrap justify-end gap-2">
							<Button label={m.admin_draft_publish()} variant="inverse" type="submit" />
						</div>
					</form>
					<form method="POST" action="?/dismissDraft" use:enhance class="flex justify-end">
						<input type="hidden" name="id" value={draft.id} />
						<Button label={m.admin_draft_dismiss()} variant="ghost" type="submit" />
					</form>
				{:else}
					<p class="text-ui font-medium text-text">{draft.title}</p>
					<p class="text-ui leading-normal whitespace-pre-wrap text-text-2">{draft.body}</p>
				{/if}
			</FormSection>
		{/each}
	{:else}
		<LoadFailed title={m.admin_support_drafts_failed()} problem={data.drafts.problem} />
	{/if}
</section>

<section aria-label={m.admin_support_articles()} class="flex flex-col gap-3">
	<SectionLabel label={m.admin_support_articles()} />
	<FilterBar label={m.admin_support_articles_search()} placeholder={m.admin_support_articles_search()} query={data.query} name="q" submitLabel={m.admin_search_submit()}>
		{#snippet actions()}
			{#if canWrite}<Button label={m.admin_support_articles_add()} variant="secondary" iconStart={Plus} href="/admin/support/articles/new" />{/if}
		{/snippet}
	</FilterBar>
	{#if data.articles.ok}
		{#if data.articles.data.length === 0}
			<p class="text-ui text-text-3">{data.query ? m.admin_support_articles_none({ query: data.query }) : m.admin_support_articles_empty()}</p>
		{:else}
			<AdminTable label={m.admin_support_articles()} columns={articleColumns} rows={data.articles.data} rowKey={(row) => row.id} muted={(row) => !row.published}>
				{#snippet cell(row, column)}
					{#if column.key === 'title'}
						<a href="/admin/support/articles/{row.id}" class="rounded-xs font-medium text-text outline-offset-2 hover:underline">{row.title}</a>
					{:else if column.key === 'tags'}
						<span class="font-mono text-label text-text-2">{row.tags.join(', ') || '—'}</span>
					{:else if column.key === 'status'}
						<Tag kind="neutral" label={row.published ? m.admin_article_published() : m.admin_article_unpublished()} />
					{:else if column.key === 'updated'}
						<span class="font-mono text-text-2">{dateTime(row.updated_at, language)}</span>
					{/if}
				{/snippet}
			</AdminTable>
		{/if}
	{:else}
		<LoadFailed title={m.admin_support_articles_failed()} problem={data.articles.problem} />
	{/if}
</section>

<section aria-label={m.admin_support_questions()} class="flex flex-col gap-3">
	<SectionLabel label={m.admin_support_questions()} />
	{#if data.questions.ok}
		{#if data.questions.data.length === 0}
			<p class="text-ui text-text-3">{m.admin_support_questions_empty()}</p>
		{:else}
			<ol class="flex flex-col rounded-md outline -outline-offset-1 outline-border">
				{#each data.questions.data as question (question.id)}
					<li class="flex flex-col gap-1 border-b border-border px-4 py-3 last:border-b-0">
						<span class="flex flex-wrap items-baseline gap-x-2">
							<span class="text-ui font-medium text-text">{question.question}</span>
							{#if !question.answered}<Tag kind="neutral" label={m.admin_question_unanswered()} />{/if}
						</span>
						{#if question.answer}<span class="text-caption leading-normal text-text-2">{question.answer}</span>{/if}
						<span class="font-mono text-label text-text-3">
							{dateTime(question.created_at, language)}{#if question.customer || question.key_name}
								· {m.admin_question_from({ customer: question.customer ?? '—', key: question.key_name ?? '—' })}{/if}
						</span>
					</li>
				{/each}
			</ol>
		{/if}
	{:else}
		<LoadFailed title={m.admin_support_questions_failed()} problem={data.questions.problem} />
	{/if}
</section>

<section aria-label={m.admin_support_keys()} class="flex flex-col gap-3">
	<SectionLabel label={m.admin_support_keys()} />
	<p class="text-ui text-text-2">{m.admin_support_keys_desc()}</p>
	{#if revealed && made}
		<SecretReveal title={m.admin_key_secret({ name: made.name })} secret={revealed} copyLabel={m.admin_copy()} copiedLabel={m.admin_copied()} doneLabel={m.admin_done()} ondone={() => (revealed = null)} />
		<p class="text-caption text-text-2">{m.admin_key_made()}</p>
	{/if}
	<Outcome {form} action="createKey" failed={m.admin_key_create_failed()} />
	<Outcome {form} action="revokeKey" done={m.admin_key_revoked()} failed={m.admin_key_revoke_failed()} />
	{#if data.keys.ok}
		{#if data.keys.data.length === 0}
			<p class="text-ui text-text-3">{m.admin_support_keys_empty()}</p>
		{:else}
			<ul class="flex flex-col rounded-md outline -outline-offset-1 outline-border">
				{#each data.keys.data as key (key.id)}
					<li class={['flex flex-wrap items-center gap-x-4 gap-y-1 border-b border-border px-4 py-3 last:border-b-0', key.revoked_at && 'opacity-60']}>
						<span class="flex min-w-0 flex-1 basis-60 flex-col gap-0.5">
							<span class="text-ui font-medium text-text">{key.name}{key.product_name ? ` · ${key.product_name}` : ''}</span>
							<span class="font-mono text-label text-text-3">
								{key.prefix}… · {key.revoked_at ? m.admin_key_revoked_on({ when: dateTime(key.revoked_at, language) }) : key.last_used_at ? m.admin_key_used({ when: dateTime(key.last_used_at, language) }) : m.admin_key_unused()}
							</span>
						</span>
						{#if canManage && !key.revoked_at}
							<Button label={m.admin_key_revoke()} variant="ghost-danger" aria-label={m.admin_key_revoke_named({ name: key.name })} onclick={() => ((revoking = key), (confirmOpen = true))} />
						{/if}
					</li>
				{/each}
			</ul>
		{/if}
	{:else}
		<LoadFailed title={m.admin_support_keys_failed()} problem={data.keys.problem} />
	{/if}
	{#if canManage}
		<form method="POST" action="?/createKey" use:enhance class="flex flex-wrap items-end gap-3">
			<div class="min-w-48 flex-1"><TextField label={m.admin_key_name()} name="name" required autocomplete="off" /></div>
			<div class="min-w-48 flex-1"><TextField label={m.admin_key_product()} name="product_name" autocomplete="off" /></div>
			<Button label={m.admin_key_create()} variant="secondary" type="submit" />
		</form>
	{/if}
</section>

<form bind:this={revokeForm} method="POST" action="?/revokeKey" use:enhance hidden>
	<input type="hidden" name="id" value={revoking?.id ?? ''} />
</form>
<ConfirmDialog
	bind:open={confirmOpen}
	title={m.admin_key_revoke_title({ name: revoking?.name ?? '' })}
	body={m.admin_key_revoke_body()}
	confirmLabel={m.admin_key_revoke()}
	cancelLabel={m.admin_cancel()}
	onconfirm={() => {
		confirmOpen = false;
		revokeForm?.requestSubmit();
	}}
/>
