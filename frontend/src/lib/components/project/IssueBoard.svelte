<script lang="ts">
	// A project's open issues as a board: Backlog and Ready to dev first (every project has them,
	// and they can't be renamed, moved or deleted), then the columns people added. Issues move by
	// dragging, or with each card's "Move to" menu (keyboard, touch). Moving one into Ready to dev
	// sends it to the crew; the card then says who has it and how it's going.
	// On a phone one column shows at a time, picked at the top.
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import Button from '$lib/components/m3/Button.svelte';
	import Dialog from '$lib/components/m3/Dialog.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import IconMoreHoriz from '$lib/components/icons/IconMoreHoriz.svelte';
	import IconPlus from '$lib/components/icons/IconPlus.svelte';
	import Mascot from '$lib/components/ds/agents/Mascot.svelte';
	import { crewMember, dropIndex, localToday, moveOnBoard, overdue } from '$lib/issues';
	import { projectApi } from '$lib/projectApi';
	import type { BoardColumn, IssueBoard, IssueKind, IssueSeverity, ProjectIssue } from '$lib/types';

	let {
		projectId,
		board,
		canEditColumns = false,
		kindLabel,
		severityLabel,
		onopen,
		onchange,
	}: {
		projectId: string;
		board: IssueBoard;
		/** The project's admins add, rename and delete columns. */
		canEditColumns?: boolean;
		kindLabel: (kind: IssueKind) => string;
		severityLabel: (severity: IssueSeverity) => string;
		/** Opens an issue to edit it. */
		onopen: (issue: ProjectIssue) => void;
		/** The board changed on the server: load it again. */
		onchange: () => Promise<void> | void;
	} = $props();

	// Moves show at once; the server's board replaces this when it's loaded again.
	let shown = $state<BoardColumn[]>([]);
	$effect(() => {
		shown = board.columns;
	});
	let error = $state<string | null>(null);
	/** The column a phone shows. */
	let current = $state<string | null>(null);
	const currentId = $derived(shown.some((c) => c.id === current) ? current : (shown[0]?.id ?? null));

	const today = localToday();

	function columnName(column: BoardColumn): string {
		if (column.fixed === 'backlog') return m.board_backlog();
		if (column.fixed === 'ready') return m.board_ready();
		return column.name;
	}

	function formatDate(date: string): string {
		return new Date(`${date}T00:00:00`).toLocaleDateString(getLocale(), { day: 'numeric', month: 'short' });
	}

	/** The card's quiet line: how the crew is getting on, else its deadline. */
	function status(issue: ProjectIssue): string | null {
		const name = crewMember(issue.crew_agent).name;
		switch (issue.crew_state) {
			case 'queued':
				return m.board_queued({ name });
			case 'working':
				return issue.assigned_by_agent
					? m.board_working_from({ name, from: crewMember(issue.assigned_by_agent).name })
					: m.board_working({ name });
			case 'done':
				return m.board_done({ name });
			case 'failed':
				return m.board_failed({ name });
		}
		if (issue.due_on) return overdue(issue.due_on, today) ? m.board_overdue({ date: formatDate(issue.due_on) }) : m.board_due({ date: formatDate(issue.due_on) });
		return null;
	}

	async function move(issue: ProjectIssue, columnId: string, index: number) {
		const before = shown;
		shown = moveOnBoard(shown, issue, columnId, index);
		try {
			await projectApi(projectId, `issues/${issue.number}/move`, { method: 'POST', body: { column_id: columnId, index } });
			error = null;
			await onchange();
		} catch {
			shown = before;
			error = m.board_err();
		}
	}

	async function retry(issue: ProjectIssue) {
		try {
			await projectApi(projectId, `issues/${issue.number}/retry`, { method: 'POST' });
			error = null;
			await onchange();
		} catch {
			error = m.board_err();
		}
	}

	// Dragging with a pointer.
	let dragging = $state<ProjectIssue | null>(null);
	let over = $state<string | null>(null);

	function drop(event: DragEvent, column: BoardColumn) {
		event.preventDefault();
		const issue = dragging;
		dragging = null;
		over = null;
		if (!issue) return;
		const list = (event.currentTarget as HTMLElement).querySelectorAll<HTMLElement>('[data-card]');
		const cards = [...list]
			.filter((card) => card.dataset.card !== String(issue.number))
			.map((card) => {
				const rect = card.getBoundingClientRect();
				return { top: rect.top, height: rect.height };
			});
		void move(issue, column.id, dropIndex(cards, event.clientY));
	}

	// Adding, renaming and deleting columns.
	let columnDialog = $state(false);
	let editingColumn = $state<BoardColumn | null>(null);
	let columnName_ = $state('');
	let savingColumn = $state(false);

	function newColumn() {
		editingColumn = null;
		columnName_ = '';
		columnDialog = true;
	}

	function editColumn(column: BoardColumn) {
		editingColumn = column;
		columnName_ = column.name;
		columnDialog = true;
	}

	async function saveColumn(event: SubmitEvent) {
		event.preventDefault();
		const name = columnName_.trim();
		if (!name) return;
		savingColumn = true;
		try {
			if (editingColumn) await projectApi(projectId, `board/columns/${editingColumn.id}`, { method: 'PATCH', body: { name } });
			else await projectApi(projectId, 'board/columns', { method: 'POST', body: { name } });
			columnDialog = false;
			error = null;
			await onchange();
		} catch {
			error = m.board_err();
		} finally {
			savingColumn = false;
		}
	}

	async function deleteColumn() {
		if (!editingColumn) return;
		savingColumn = true;
		try {
			await projectApi(projectId, `board/columns/${editingColumn.id}`, { method: 'DELETE' });
			columnDialog = false;
			error = null;
			await onchange();
		} catch {
			error = m.board_err();
		} finally {
			savingColumn = false;
		}
	}
</script>

{#if error}<p class="error" role="alert">{error}</p>{/if}

<div class="picker" role="tablist" aria-label={m.board_columns()}>
	{#each shown as column (column.id)}
		<button type="button" role="tab" class="picker__tab" aria-selected={column.id === currentId} onclick={() => (current = column.id)}>
			{columnName(column)}<span class="count">{column.issues.length}</span>
		</button>
	{/each}
</div>

<div class="board">
	{#each shown as column (column.id)}
		<section
			class="column"
			class:column--current={column.id === currentId}
			class:column--over={over === column.id}
			aria-label={columnName(column)}
			ondragover={(event) => {
				if (!dragging) return;
				event.preventDefault();
				over = column.id;
			}}
			ondragleave={() => (over = over === column.id ? null : over)}
			ondrop={(event) => drop(event, column)}
		>
			<header class="column__head">
				<h3 class="column__name">{columnName(column)}<span class="count">{column.issues.length}</span></h3>
				{#if canEditColumns && !column.fixed}
					<IconButton type="button" aria-label={m.board_edit_column({ name: column.name })} title={m.board_edit_column({ name: column.name })} onclick={() => editColumn(column)}>
						<IconMoreHoriz size={18} />
					</IconButton>
				{/if}
			</header>
			{#if column.fixed === 'ready'}<p class="column__hint">{m.board_ready_hint()}</p>{/if}

			{#if column.issues.length === 0}
				<p class="column__empty">{m.board_empty_column()}</p>
			{:else}
				<ul class="cards">
					{#each column.issues as issue (issue.number)}
						{@const who = crewMember(issue.crew_agent)}
						{@const line = status(issue)}
						<li
							class="card"
							class:card--dragging={dragging?.number === issue.number}
							data-card={issue.number}
							draggable="true"
							ondragstart={(event) => {
								dragging = issue;
								event.dataTransfer?.setData('text/plain', String(issue.number));
							}}
							ondragend={() => {
								dragging = null;
								over = null;
							}}
						>
							<button type="button" class="card__open" onclick={() => onopen(issue)}>
								<span class="card__title">#{issue.number} {issue.title}</span>
								<span class="card__line">
									{kindLabel(issue.kind)} · {severityLabel(issue.severity)}{#if issue.points}&nbsp;· {m.board_points({ count: issue.points })}{/if}
								</span>
								{#if line}
									<span class="card__meta" class:card__meta--late={!issue.crew_state && overdue(issue.due_on, today)} class:card__meta--failed={issue.crew_state === 'failed'}>
										{#if issue.crew_state && who.crew}<Mascot agent={who.crew} size={16} />{/if}
										{line}
									</span>
								{/if}
							</button>
							<div class="card__actions">
								<label class="move">
									<span class="sr-only">{m.board_move_label({ number: issue.number })}</span>
									<select
										value={column.id}
										onchange={(event) => {
											const target = (event.currentTarget as HTMLSelectElement).value;
											if (target !== column.id) void move(issue, target, shown.find((c) => c.id === target)?.issues.length ?? 0);
										}}
									>
										{#each shown as option (option.id)}
											<option value={option.id}>{columnName(option)}</option>
										{/each}
									</select>
								</label>
								{#if issue.crew_state === 'failed'}
									<Button type="button" variant="text" size="s" onclick={() => retry(issue)}>{m.board_retry()}</Button>
								{/if}
								{#if issue.session_id}
									<a class="chat-link" href="/projects/session/{issue.session_id}">{m.board_open_chat()}</a>
								{/if}
							</div>
						</li>
					{/each}
				</ul>
			{/if}
		</section>
	{/each}
	{#if canEditColumns}
		<div class="add">
			<Button type="button" variant="text" size="s" onclick={newColumn}><IconPlus size={18} />{m.board_add_column()}</Button>
		</div>
	{/if}
</div>

<Dialog bind:open={columnDialog} headline={editingColumn ? m.board_edit_column({ name: editingColumn.name }) : m.board_add_column()}>
	<form id="column-form" class="column-form" onsubmit={saveColumn}>
		<TextField name="name" label={m.board_column_name()} bind:value={columnName_} maxlength={40} required />
		{#if editingColumn}<p class="column-form__hint">{m.board_delete_column_hint()}</p>{/if}
	</form>
	{#snippet actions()}
		{#if editingColumn}
			<Button variant="text" type="button" class="danger" disabled={savingColumn} onclick={deleteColumn}>{m.board_delete_column()}</Button>
		{/if}
		<Button variant="text" type="button" onclick={() => (columnDialog = false)}>{m.issues_cancel()}</Button>
		<Button variant="filled" type="submit" form="column-form" disabled={savingColumn || !columnName_.trim()}>{editingColumn ? m.issues_save() : m.board_add_column()}</Button>
	{/snippet}
</Dialog>

<style>
	.error {
		margin: 0;
		color: var(--md-sys-color-error);
	}
	.count {
		margin-left: 6px;
		font-variant-numeric: tabular-nums;
		font-weight: 500;
		color: var(--md-sys-color-on-surface-variant);
	}
	/* The phone's column picker; on wider screens every column shows. */
	.picker {
		display: none;
	}
	.board {
		display: flex;
		align-items: flex-start;
		gap: 12px;
		overflow-x: auto;
		padding-bottom: 8px;
		overscroll-behavior-x: contain;
	}
	.column {
		display: flex;
		flex: 0 0 272px;
		flex-direction: column;
		gap: 8px;
		min-height: 160px;
		padding: 10px;
		border-radius: 12px;
		background: var(--md-sys-color-surface-container-low);
	}
	.column--over {
		outline: 2px dashed var(--md-sys-color-primary);
		outline-offset: -2px;
	}
	.column__head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		min-height: 40px;
	}
	.column__name {
		margin: 0;
		font-size: 0.9375rem;
		font-weight: 600;
		color: var(--md-sys-color-on-surface);
	}
	.column__hint,
	.column__empty {
		margin: 0;
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.column__empty {
		padding: 12px 4px;
	}
	.cards {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.card {
		display: flex;
		flex-direction: column;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 10px;
		background: var(--md-sys-color-surface-container-lowest);
		cursor: grab;
	}
	.card--dragging {
		opacity: 0.5;
	}
	.card__open {
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: 10px 12px 6px;
		border: none;
		border-radius: 10px 10px 0 0;
		background: none;
		color: var(--md-sys-color-on-surface);
		font: inherit;
		text-align: left;
		cursor: pointer;
	}
	.card__open:hover {
		background: var(--md-sys-color-surface-container);
	}
	.card__open:focus-visible,
	.picker__tab:focus-visible,
	.chat-link:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.card__title {
		font-weight: 600;
		overflow-wrap: anywhere;
	}
	.card__line,
	.card__meta {
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.card__meta {
		display: inline-flex;
		align-items: center;
		gap: 6px;
	}
	.card__meta--late,
	.card__meta--failed {
		color: var(--md-sys-color-error);
	}
	.card__actions {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 4px 8px;
		padding: 2px 8px 8px;
	}
	.move select {
		min-height: 32px;
		max-width: 150px;
		padding: 0 8px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 8px;
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
		font: inherit;
		font-size: 0.8125rem;
		cursor: pointer;
	}
	.move select:hover {
		background: var(--md-sys-color-surface-container);
	}
	.move select:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.chat-link {
		border-radius: 4px;
		font-size: 0.8125rem;
		font-weight: 600;
		color: var(--md-sys-color-primary);
		text-decoration: none;
	}
	.chat-link:hover {
		text-decoration: underline;
	}
	.add {
		flex: 0 0 auto;
		padding-top: 8px;
	}
	.column-form {
		display: flex;
		flex-direction: column;
		gap: 10px;
		min-width: min(360px, 80vw);
	}
	.column-form__hint {
		margin: 0;
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	:global(.danger) {
		color: var(--md-sys-color-error);
	}
	@media (max-width: 600px) {
		.picker {
			display: flex;
			gap: 4px;
			overflow-x: auto;
			padding-bottom: 4px;
		}
		.picker__tab {
			flex: none;
			min-height: 44px;
			padding: 0 12px;
			border: none;
			border-radius: 8px;
			background: none;
			color: var(--md-sys-color-on-surface-variant);
			font: inherit;
			font-weight: 600;
			cursor: pointer;
		}
		.picker__tab:hover {
			background: var(--md-sys-color-surface-container);
		}
		.picker__tab[aria-selected='true'] {
			background: var(--md-sys-color-surface-container-highest);
			color: var(--md-sys-color-on-surface);
		}
		.board {
			display: block;
			overflow: visible;
		}
		.column {
			display: none;
		}
		.column--current {
			display: flex;
		}
		.move select {
			min-height: 44px;
			max-width: none;
		}
		.chat-link {
			display: inline-flex;
			align-items: center;
			min-height: 44px;
		}
		.add {
			padding-top: 12px;
		}
	}
</style>
