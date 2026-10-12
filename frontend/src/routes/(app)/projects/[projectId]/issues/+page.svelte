<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { enhance } from '$app/forms';
	import { goto } from '$app/navigation';
	import Button from '$lib/components/m3/Button.svelte';
	import Chip from '$lib/components/m3/Chip.svelte';
	import Dialog from '$lib/components/m3/Dialog.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import IconArrowBack from '$lib/components/icons/IconArrowBack.svelte';
	import IconPlus from '$lib/components/icons/IconPlus.svelte';
	import IconChevronDown from '$lib/components/icons/IconChevronDown.svelte';
	import { ISSUE_KINDS, ISSUE_SEVERITIES, issuesHref } from '$lib/issues';
	import type { IssueKind, IssueSeverity, ProjectIssue } from '$lib/types';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	const KIND: Record<IssueKind, () => string> = {
		feature: m.issues_kind_feature,
		enhancement: m.issues_kind_enhancement,
		bug: m.issues_kind_bug,
	};
	const SEVERITY: Record<IssueSeverity, () => string> = {
		low: m.issues_severity_low,
		medium: m.issues_severity_medium,
		high: m.issues_severity_high,
		critical: m.issues_severity_critical,
	};

	// The editor: a new issue, or an existing one being edited.
	let editorOpen = $state(false);
	let editing = $state<number | null>(null);
	let title = $state('');
	let body = $state('');
	let kind = $state<IssueKind>('bug');
	let severity = $state<IssueSeverity>('medium');
	let saving = $state(false);

	let expanded = $state<number | null>(null);
	let deleting = $state<ProjectIssue | null>(null);
	let deleteOpen = $state(false);

	const error = $derived(form && 'error' in form ? (form.error as string) : null);
	const filtered = $derived(data.filters.kind !== null || data.filters.severity !== null);

	function openNew() {
		editing = null;
		title = '';
		body = '';
		kind = 'bug';
		severity = 'medium';
		editorOpen = true;
	}

	function openEdit(issue: ProjectIssue) {
		editing = issue.number;
		title = issue.title;
		body = issue.body;
		kind = issue.kind;
		severity = issue.severity;
		editorOpen = true;
	}

	function when(at: string): string {
		const date = new Date(at);
		const days = Math.round((Date.now() - date.getTime()) / 86_400_000);
		if (days < 7) {
			const rtf = new Intl.RelativeTimeFormat(getLocale(), { numeric: 'auto' });
			const hours = Math.round((Date.now() - date.getTime()) / 3_600_000);
			return hours < 24 ? rtf.format(-hours, 'hour') : rtf.format(-days, 'day');
		}
		return new Intl.DateTimeFormat(getLocale(), { dateStyle: 'medium' }).format(date);
	}

	function filter(change: Parameters<typeof issuesHref>[1]) {
		expanded = null;
		void goto(issuesHref(data.filters, change), { keepFocus: true, noScroll: true });
	}

	const emptyText = $derived(
		filtered ? m.issues_empty_filtered() : data.filters.status === 'open' ? m.issues_empty_open() : m.issues_empty_closed(),
	);
</script>

<div class="issues">
	<div class="issues__inner">
		<a class="back" href="/projects/session/{data.project.sessionId}" data-sveltekit-reload><IconArrowBack size={18} />{m.issues_back()}</a>

		<header class="head">
			<div class="head__text">
				<h1 class="md-display-small head__title">{m.issues_title()}</h1>
				<p class="head__lede">{m.issues_lede({ project: data.project.name })}</p>
			</div>
			<Button variant="filled" size="m" onclick={openNew}><IconPlus size={20} />{m.issues_new()}</Button>
		</header>

		<div class="bar">
			<div class="tabs" role="tablist" aria-label={m.issues_filters()}>
				<button type="button" role="tab" class="tab" aria-selected={data.filters.status === 'open'} onclick={() => filter({ status: 'open' })}>
					{m.issues_open()}<span class="tab__count">{data.counts.open}</span>
				</button>
				<button type="button" role="tab" class="tab" aria-selected={data.filters.status === 'closed'} onclick={() => filter({ status: 'closed' })}>
					{m.issues_closed()}<span class="tab__count">{data.counts.closed}</span>
				</button>
			</div>
			<div class="chips" role="group" aria-label={m.issues_type()}>
				{#each ISSUE_KINDS as k (k)}
					<Chip variant="filter" selected={data.filters.kind === k} onclick={() => filter({ kind: data.filters.kind === k ? null : k })}>{KIND[k]()}</Chip>
				{/each}
			</div>
			<label class="severity-select">
				<span class="sr-only">{m.issues_severity()}</span>
				<select
					value={data.filters.severity ?? ''}
					onchange={(event) => filter({ severity: ((event.currentTarget as HTMLSelectElement).value || null) as IssueSeverity | null })}
				>
					<option value="">{m.issues_all_severities()}</option>
					{#each ISSUE_SEVERITIES as s (s)}
						<option value={s}>{SEVERITY[s]()}</option>
					{/each}
				</select>
				<IconChevronDown size={16} />
			</label>
		</div>

		{#if error && !editorOpen}<p class="error" role="alert">{error}</p>{/if}

		{#if data.issues.length === 0}
			<p class="empty">{emptyText}</p>
		{:else}
			<ul class="list">
				{#each data.issues as issue (issue.number)}
					{@const open = expanded === issue.number}
					<li class="issue" class:issue--open={open}>
						<button type="button" class="issue__row" aria-expanded={open} onclick={() => (expanded = open ? null : issue.number)}>
							<span class="kind kind--{issue.kind}">{KIND[issue.kind]()}</span>
							<span class="issue__text">
								<span class="issue__title">{issue.title}</span>
								<span class="issue__meta">
									{m.issues_opened_by({ number: issue.number, name: issue.created_by ?? '—', when: when(issue.created_at) })}
									{#if issue.closed_at}· {m.issues_closed_on({ when: when(issue.closed_at) })}{/if}
								</span>
							</span>
							<span class="severity severity--{issue.severity}">{SEVERITY[issue.severity]()}</span>
						</button>
						{#if open}
							<div class="issue__detail">
								{#if issue.body}<p class="issue__body">{issue.body}</p>{/if}
								<div class="issue__actions">
									{#if issue.status === 'open'}
										<form method="POST" action="?/askKoda" use:enhance>
											<input type="hidden" name="number" value={issue.number} />
											<Button type="submit" variant="tonal" size="s">{m.issues_send_to_koda()}</Button>
										</form>
									{/if}
									<Button type="button" variant="text" size="s" onclick={() => openEdit(issue)}>{m.issues_edit()}</Button>
									<form method="POST" action="?/status" use:enhance={() => async ({ update }) => { expanded = null; await update(); }}>
										<input type="hidden" name="number" value={issue.number} />
										<input type="hidden" name="status" value={issue.status === 'open' ? 'closed' : 'open'} />
										<Button type="submit" variant="text" size="s">{issue.status === 'open' ? m.issues_close() : m.issues_reopen()}</Button>
									</form>
									<Button type="button" variant="text" size="s" class="danger" onclick={() => { deleting = issue; deleteOpen = true; }}>{m.issues_delete()}</Button>
								</div>
							</div>
						{/if}
					</li>
				{/each}
			</ul>
		{/if}
	</div>
</div>

<Dialog bind:open={editorOpen} headline={editing === null ? m.issues_new() : `#${editing}`} class="issue-dialog">
	<form
		id="issue-form"
		method="POST"
		action={editing === null ? '?/create' : '?/update'}
		class="editor"
		use:enhance={() => {
			saving = true;
			return async ({ update, result }) => {
				await update({ reset: false });
				saving = false;
				if (result.type === 'success') editorOpen = false;
			};
		}}
	>
		{#if editing !== null}<input type="hidden" name="number" value={editing} />{/if}
		<TextField name="title" label={m.issues_field_title()} bind:value={title} maxlength={200} required />
		<fieldset class="choice">
			<legend class="choice__label">{m.issues_type()}</legend>
			<div class="choice__options">
				{#each ISSUE_KINDS as k (k)}
					<label class="choice__option" class:choice__option--on={kind === k}>
						<input type="radio" name="kind" value={k} bind:group={kind} />{KIND[k]()}
					</label>
				{/each}
			</div>
		</fieldset>
		<fieldset class="choice">
			<legend class="choice__label">{m.issues_severity()}</legend>
			<div class="choice__options">
				{#each ['low', 'medium', 'high', 'critical'] as const as s (s)}
					<label class="choice__option choice__option--{s}" class:choice__option--on={severity === s}>
						<input type="radio" name="severity" value={s} bind:group={severity} />{SEVERITY[s]()}
					</label>
				{/each}
			</div>
		</fieldset>
		<label class="area">
			<span class="area__label">{m.issues_field_body()}</span>
			<textarea name="body" rows="6" maxlength="20000" bind:value={body} placeholder={m.issues_field_body_hint()}></textarea>
		</label>
		{#if error && editorOpen}<p class="error" role="alert">{error}</p>{/if}
	</form>
	{#snippet actions()}
		<Button variant="text" type="button" onclick={() => (editorOpen = false)}>{m.issues_cancel()}</Button>
		<Button variant="filled" type="submit" form="issue-form" disabled={saving || !title.trim()}>
			{editing === null ? m.issues_create() : m.issues_save()}
		</Button>
	{/snippet}
</Dialog>

<Dialog bind:open={deleteOpen} headline={m.issues_delete()}>
	{#if deleting}
		<p>{m.issues_delete_confirm({ number: deleting.number })}</p>
		<form
			id="delete-issue"
			method="POST"
			action="?/delete"
			use:enhance={() => async ({ update }) => {
				await update();
				deleteOpen = false;
				expanded = null;
			}}
		>
			<input type="hidden" name="number" value={deleting.number} />
		</form>
	{/if}
	{#snippet actions()}
		<Button variant="text" type="button" onclick={() => (deleteOpen = false)}>{m.issues_cancel()}</Button>
		<Button variant="filled" type="submit" form="delete-issue">{m.issues_delete()}</Button>
	{/snippet}
</Dialog>

<style>
	.issues {
		height: 100%;
		overflow-y: auto;
		padding: 20px clamp(14px, 4vw, 36px) 44px;
		box-sizing: border-box;
	}
	.issues__inner {
		display: flex;
		flex-direction: column;
		gap: 14px;
		max-width: 880px;
		margin: 0 auto;
	}
	.back {
		display: inline-flex;
		align-items: center;
		align-self: flex-start;
		gap: 6px;
		min-height: 44px;
		color: var(--md-sys-color-on-surface-variant);
		font-weight: 600;
		text-decoration: none;
	}
	.back:hover {
		color: var(--md-sys-color-on-surface);
	}
	.head {
		display: flex;
		flex-wrap: wrap;
		align-items: flex-end;
		justify-content: space-between;
		gap: 14px;
	}
	.head__text {
		min-width: 0;
	}
	.head__title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
	}
	.head__lede {
		margin: 6px 0 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.bar {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 12px;
		padding-bottom: 12px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.tabs {
		display: flex;
		gap: 4px;
		margin-right: auto;
	}
	.tab {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		min-height: 40px;
		padding: 0 12px;
		border: none;
		border-radius: 6px;
		background: none;
		color: var(--md-sys-color-on-surface-variant);
		font: inherit;
		font-weight: 600;
		cursor: pointer;
	}
	.tab:hover {
		background: var(--md-sys-color-surface-container-low);
	}
	.tab[aria-selected='true'] {
		background: var(--md-sys-color-surface-container-highest);
		color: var(--md-sys-color-on-surface);
	}
	.tab:focus-visible,
	.issue__row:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.tab__count {
		font-variant-numeric: tabular-nums;
		font-weight: 500;
		opacity: 0.75;
	}
	.chips {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}
	.severity-select {
		position: relative;
		display: inline-flex;
		align-items: center;
		color: var(--md-sys-color-on-surface-variant);
	}
	.severity-select select {
		min-height: 32px;
		padding: 0 30px 0 12px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 8px;
		background: var(--md-sys-color-surface);
		color: var(--md-sys-color-on-surface);
		font: inherit;
		font-size: 0.875rem;
		appearance: none;
		cursor: pointer;
	}
	.severity-select select:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.severity-select :global(svg) {
		position: absolute;
		right: 10px;
		pointer-events: none;
	}
	.error {
		margin: 0;
		color: var(--md-sys-color-error);
	}
	.empty {
		margin: 8px 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.list {
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.issue {
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.issue__row {
		display: flex;
		align-items: center;
		gap: 12px;
		width: 100%;
		min-height: 60px;
		padding: 10px 8px;
		border: none;
		background: none;
		color: var(--md-sys-color-on-surface);
		font: inherit;
		text-align: left;
		cursor: pointer;
	}
	.issue__row:hover,
	.issue--open .issue__row {
		background: var(--md-sys-color-surface-container-low);
	}
	.issue__text {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.issue__title {
		font-weight: 600;
		overflow-wrap: anywhere;
	}
	.issue__meta {
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.kind,
	.severity {
		flex: none;
		padding: 2px 8px;
		border-radius: 4px;
		font-size: 0.75rem;
		font-weight: 600;
		white-space: nowrap;
	}
	.kind {
		border: 1px solid var(--md-sys-color-outline-variant);
		color: var(--md-sys-color-on-surface-variant);
	}
	.kind--bug {
		border-color: color-mix(in srgb, var(--md-sys-color-error) 45%, transparent);
		color: var(--md-sys-color-error);
	}
	.kind--feature {
		border-color: color-mix(in srgb, var(--md-sys-color-primary) 45%, transparent);
		color: var(--md-sys-color-primary);
	}
	.severity {
		background: var(--md-sys-color-surface-container-highest);
		color: var(--md-sys-color-on-surface-variant);
	}
	.severity--medium {
		background: var(--md-sys-color-tertiary-container);
		color: var(--md-sys-color-on-tertiary-container);
	}
	.severity--high {
		background: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
	}
	.severity--critical {
		background: var(--md-sys-color-error);
		color: var(--md-sys-color-on-error);
	}
	.issue__detail {
		display: flex;
		flex-direction: column;
		gap: 12px;
		padding: 4px 8px 14px;
		background: var(--md-sys-color-surface-container-low);
	}
	.issue__body {
		margin: 0;
		max-width: 65ch;
		white-space: pre-wrap;
		overflow-wrap: anywhere;
		color: var(--md-sys-color-on-surface);
	}
	.issue__actions {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 4px;
	}
	.issue__actions :global(.danger) {
		color: var(--md-sys-color-error);
	}
	@media (max-width: 600px) {
		.issue__row {
			flex-wrap: wrap;
			gap: 6px 10px;
		}
		.issue__text {
			flex-basis: 100%;
			order: 2;
		}
		.severity {
			margin-left: auto;
		}
		.tabs {
			width: 100%;
		}
	}
	.editor {
		display: flex;
		flex-direction: column;
		gap: 14px;
		min-width: min(520px, 80vw);
	}
	.choice {
		margin: 0;
		padding: 0;
		border: none;
	}
	.choice__label,
	.area__label {
		margin-bottom: 6px;
		font-size: var(--md-sys-typescale-body-small-size);
		color: var(--md-sys-color-on-surface-variant);
	}
	.choice__options {
		display: flex;
		border: 1px solid var(--md-sys-color-outline);
		border-radius: 8px;
		overflow: hidden;
	}
	.choice__option {
		display: inline-flex;
		flex: 1 1 auto;
		align-items: center;
		justify-content: center;
		min-height: 40px;
		padding: 0 10px;
		white-space: nowrap;
		border-left: 1px solid var(--md-sys-color-outline);
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--md-sys-color-on-surface);
		cursor: pointer;
	}
	.choice__option:first-child {
		border-left: none;
	}
	.choice__option:has(input:focus-visible) {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: -2px;
	}
	.choice__option input {
		position: absolute;
		opacity: 0;
		pointer-events: none;
	}
	.choice__option--on {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.choice__option--critical.choice__option--on {
		background: var(--md-sys-color-error);
		color: var(--md-sys-color-on-error);
	}
	.choice__option--high.choice__option--on {
		background: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
	}
	@media (max-width: 600px) {
		.choice__option {
			padding: 0 6px;
			font-size: 0.75rem;
		}
	}
	.area {
		display: flex;
		flex-direction: column;
	}
	.area textarea {
		box-sizing: border-box;
		width: 100%;
		min-height: 140px;
		padding: 12px 14px;
		border: 1.5px solid var(--md-sys-color-outline);
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface);
		color: var(--md-sys-color-on-surface);
		font: inherit;
		resize: vertical;
	}
	.area textarea:focus {
		outline: none;
		border: 2px solid var(--md-sys-color-primary);
		padding: 11.5px 15.5px;
	}
</style>
