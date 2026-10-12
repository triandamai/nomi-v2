<script lang="ts">
	// A project's issues: feature requests, enhancements and bugs. Opened in a sheet over the
	// project so its chat, code and running preview stay as they are.
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import Button from '$lib/components/m3/Button.svelte';
	import Chip from '$lib/components/m3/Chip.svelte';
	import Dialog from '$lib/components/m3/Dialog.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import IconPlus from '$lib/components/icons/IconPlus.svelte';
	import IconChevronDown from '$lib/components/icons/IconChevronDown.svelte';
	import { ISSUE_KINDS, ISSUE_SEVERITIES, issuesQuery, type IssueFilters } from '$lib/issues';
	import { ago, projectApi } from '$lib/projectApi';
	import type { IssueKind, IssueList, IssueSeverity, ProjectIssue } from '$lib/types';

	let {
		projectId,
		onCountChange,
		onAskKoda,
	}: {
		projectId: string;
		/** The open issue count changed (the toolbar shows it). */
		onCountChange?: (open: number) => void;
		/** Sends a message to the project's chat. */
		onAskKoda: (text: string) => Promise<void>;
	} = $props();

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

	let filters = $state<IssueFilters>({ status: 'open', kind: null, severity: null });
	let list = $state<IssueList | null>(null);
	let loadError = $state<string | null>(null);
	let error = $state<string | null>(null);

	async function load() {
		try {
			list = await projectApi<IssueList>(projectId, `issues?${issuesQuery(filters)}`);
			loadError = null;
			onCountChange?.(list.counts.open);
		} catch {
			loadError = m.issues_err_load();
		}
	}

	$effect(() => {
		void filters.status;
		void filters.kind;
		void filters.severity;
		void load();
	});

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
	let asking = $state<number | null>(null);

	const filtered = $derived(filters.kind !== null || filters.severity !== null);

	function openNew() {
		editing = null;
		title = '';
		body = '';
		kind = 'bug';
		severity = 'medium';
		error = null;
		editorOpen = true;
	}

	function openEdit(issue: ProjectIssue) {
		editing = issue.number;
		title = issue.title;
		body = issue.body;
		kind = issue.kind;
		severity = issue.severity;
		error = null;
		editorOpen = true;
	}

	async function save(event: SubmitEvent) {
		event.preventDefault();
		if (!title.trim()) {
			error = m.issues_err_title();
			return;
		}
		saving = true;
		try {
			const fields = { title: title.trim(), body, kind, severity };
			if (editing === null) await projectApi(projectId, 'issues', { method: 'POST', body: fields });
			else await projectApi(projectId, `issues/${editing}`, { method: 'PATCH', body: fields });
			editorOpen = false;
			await load();
		} catch {
			error = m.issues_err_save();
		} finally {
			saving = false;
		}
	}

	async function setStatus(issue: ProjectIssue) {
		try {
			await projectApi(projectId, `issues/${issue.number}`, { method: 'PATCH', body: { status: issue.status === 'open' ? 'closed' : 'open' } });
			expanded = null;
			await load();
		} catch {
			error = m.issues_err_save();
		}
	}

	async function remove() {
		if (!deleting) return;
		try {
			await projectApi(projectId, `issues/${deleting.number}`, { method: 'DELETE' });
			deleteOpen = false;
			expanded = null;
			await load();
		} catch {
			error = m.issues_err_save();
		}
	}

	async function askKoda(issue: ProjectIssue) {
		asking = issue.number;
		const text = [
			m.issues_koda_message({
				number: issue.number,
				kind: KIND[issue.kind]().toLowerCase(),
				severity: SEVERITY[issue.severity]().toLowerCase(),
				title: issue.title,
			}),
			issue.body.trim(),
		]
			.filter(Boolean)
			.join('\n\n');
		try {
			await onAskKoda(text);
		} catch {
			error = m.issues_err_save();
		} finally {
			asking = null;
		}
	}

	function filter(change: Partial<IssueFilters>) {
		expanded = null;
		filters = { ...filters, ...change };
	}

	const emptyText = $derived(
		filtered ? m.issues_empty_filtered() : filters.status === 'open' ? m.issues_empty_open() : m.issues_empty_closed(),
	);
</script>

<div class="issues">
	<div class="bar">
		<div class="tabs" role="tablist" aria-label={m.issues_filters()}>
			<button type="button" role="tab" class="tab" aria-selected={filters.status === 'open'} onclick={() => filter({ status: 'open' })}>
				{m.issues_open()}<span class="tab__count">{list?.counts.open ?? '–'}</span>
			</button>
			<button type="button" role="tab" class="tab" aria-selected={filters.status === 'closed'} onclick={() => filter({ status: 'closed' })}>
				{m.issues_closed()}<span class="tab__count">{list?.counts.closed ?? '–'}</span>
			</button>
		</div>
		<div class="chips" role="group" aria-label={m.issues_type()}>
			{#each ISSUE_KINDS as k (k)}
				<Chip variant="filter" selected={filters.kind === k} onclick={() => filter({ kind: filters.kind === k ? null : k })}>{KIND[k]()}</Chip>
			{/each}
		</div>
		<label class="severity-select">
			<span class="sr-only">{m.issues_severity()}</span>
			<select
				value={filters.severity ?? ''}
				onchange={(event) => filter({ severity: ((event.currentTarget as HTMLSelectElement).value || null) as IssueSeverity | null })}
			>
				<option value="">{m.issues_all_severities()}</option>
				{#each ISSUE_SEVERITIES as s (s)}
					<option value={s}>{SEVERITY[s]()}</option>
				{/each}
			</select>
			<IconChevronDown size={16} />
		</label>
		<Button variant="filled" size="s" onclick={openNew}><IconPlus size={18} />{m.issues_new()}</Button>
	</div>

	{#if error && !editorOpen}<p class="error" role="alert">{error}</p>{/if}
	{#if loadError}<p class="error" role="alert">{loadError}</p>{/if}

	{#if list && list.issues.length === 0}
		<p class="empty">{emptyText}</p>
	{:else if list}
		<ul class="list">
			{#each list.issues as issue (issue.number)}
				{@const open = expanded === issue.number}
				<li class="issue" class:issue--open={open}>
					<button type="button" class="issue__row" aria-expanded={open} onclick={() => (expanded = open ? null : issue.number)}>
						<span class="kind kind--{issue.kind}">{KIND[issue.kind]()}</span>
						<span class="issue__text">
							<span class="issue__title">{issue.title}</span>
							<span class="issue__meta">
								{m.issues_opened_by({ number: issue.number, name: issue.created_by ?? '—', when: ago(issue.created_at, getLocale()) })}
								{#if issue.closed_at}· {m.issues_closed_on({ when: ago(issue.closed_at, getLocale()) })}{/if}
							</span>
						</span>
						<span class="severity severity--{issue.severity}">{SEVERITY[issue.severity]()}</span>
					</button>
					{#if open}
						<div class="issue__detail">
							{#if issue.body}<p class="issue__body">{issue.body}</p>{/if}
							<div class="issue__actions">
								{#if issue.status === 'open'}
									<Button type="button" variant="tonal" size="s" disabled={asking === issue.number} onclick={() => askKoda(issue)}>{m.issues_send_to_koda()}</Button>
								{/if}
								<Button type="button" variant="text" size="s" onclick={() => openEdit(issue)}>{m.issues_edit()}</Button>
								<Button type="button" variant="text" size="s" onclick={() => setStatus(issue)}>{issue.status === 'open' ? m.issues_close() : m.issues_reopen()}</Button>
								<Button type="button" variant="text" size="s" class="danger" onclick={() => { deleting = issue; deleteOpen = true; }}>{m.issues_delete()}</Button>
							</div>
						</div>
					{/if}
				</li>
			{/each}
		</ul>
	{/if}
</div>

<Dialog bind:open={editorOpen} headline={editing === null ? m.issues_new() : `#${editing}`} class="issue-dialog">
	<form id="issue-form" class="editor" onsubmit={save}>
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
	{#if deleting}<p>{m.issues_delete_confirm({ number: deleting.number })}</p>{/if}
	{#snippet actions()}
		<Button variant="text" type="button" onclick={() => (deleteOpen = false)}>{m.issues_cancel()}</Button>
		<Button variant="filled" type="button" onclick={remove}>{m.issues_delete()}</Button>
	{/snippet}
</Dialog>

<style>
	.issues {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.bar {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 10px;
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
		background: var(--md-sys-color-surface-container);
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
		min-height: 36px;
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
		min-height: 56px;
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
		background: var(--md-sys-color-surface-container);
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
		background: var(--md-sys-color-surface-container);
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
		padding: 11.5px 13.5px;
	}
</style>
