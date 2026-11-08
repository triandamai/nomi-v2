<script lang="ts">
	// A new issue, or an existing one, in a full-screen sheet laid out like a page: the title, the
	// issue's properties, then its description, written like a Notion page (Markdown underneath).
	// "Work immediately" starts it today and sends it straight to Ready to dev for the crew.
	import { m } from '$lib/paraglide/messages';
	import { Zap } from '@lucide/svelte';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import DatePicker from '$lib/components/m3/DatePicker.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import Select from '$lib/components/m3/Select.svelte';
	import IconClose from '$lib/components/icons/IconClose.svelte';
	import MarkdownEditor from '$lib/components/MarkdownEditor.svelte';
	import CrewPicker from '$lib/components/project/CrewPicker.svelte';
	import {
		DEFAULT_CREW_AGENT,
		ISSUE_KINDS,
		ISSUE_SEVERITIES,
		MAX_BODY_CHARS,
		MAX_POINTS,
		POINT_SUGGESTIONS,
		crewMember,
		datesInOrder,
		localToday,
	} from '$lib/issues';
	import { projectApi } from '$lib/projectApi';
	import type { IssueBoard, IssueKind, IssueSeverity, ProjectIssue } from '$lib/types';

	let {
		open = $bindable(false),
		projectId,
		issue,
		kindLabel,
		severityLabel,
		onsaved,
	}: {
		open?: boolean;
		projectId: string;
		/** The issue being edited; null for a new one. */
		issue: ProjectIssue | null;
		kindLabel: (kind: IssueKind) => string;
		severityLabel: (severity: IssueSeverity) => string;
		onsaved: () => void | Promise<void>;
	} = $props();

	const ROLE: Record<string, () => string> = {
		coding: m.crew_role_coding,
		planning: m.crew_role_planning,
		money: m.crew_role_money,
		reminders: m.crew_role_reminders,
		workspace: m.crew_role_workspace,
		files: m.crew_role_files,
		health: m.crew_role_health,
		chitchat: m.crew_role_nomi,
	};
	const role = (agent: string) => ROLE[agent]?.() ?? m.crew_role_custom();

	let title = $state('');
	let body = $state('');
	let kind = $state<IssueKind>('task');
	let severity = $state<IssueSeverity>('medium');
	let points = $state('');
	let startOn = $state('');
	let dueOn = $state('');
	let crewAgent = $state(DEFAULT_CREW_AGENT);
	let workNow = $state(false);
	let saving = $state(false);
	let error = $state<string | null>(null);

	// The crew already has it: "Work immediately" has nothing left to do.
	const taken = $derived(issue !== null && issue.crew_state !== null && issue.crew_state !== 'failed');

	// Fill the fields each time the sheet opens.
	let filledFor: string | null = null;
	$effect(() => {
		if (!open) {
			filledFor = null;
			return;
		}
		const key = issue ? `#${issue.number}` : 'new';
		if (filledFor === key) return;
		filledFor = key;
		title = issue?.title ?? '';
		body = issue?.body ?? '';
		kind = issue?.kind ?? 'task';
		severity = issue?.severity ?? 'medium';
		points = issue?.points ? String(issue.points) : '';
		startOn = issue?.start_on ?? '';
		dueOn = issue?.due_on ?? '';
		crewAgent = issue?.crew_agent ?? DEFAULT_CREW_AGENT;
		workNow = false;
		error = null;
	});

	function toggleWorkNow() {
		workNow = !workNow;
		if (workNow) startOn = localToday();
	}

	async function moveToReady(number: number) {
		const board = await projectApi<IssueBoard>(projectId, 'board');
		const ready = board.columns.find((column) => column.fixed === 'ready');
		if (!ready) return;
		await projectApi(projectId, `issues/${number}/move`, { method: 'POST', body: { column_id: ready.id, index: ready.issues.length } });
	}

	async function save(event: SubmitEvent) {
		event.preventDefault();
		if (!title.trim()) {
			error = m.issues_err_title();
			return;
		}
		if (!datesInOrder(startOn, dueOn)) {
			error = m.issues_err_dates();
			return;
		}
		if (body.length > MAX_BODY_CHARS) {
			error = m.issues_err_body({ max: MAX_BODY_CHARS });
			return;
		}
		const estimate = Number(points);
		saving = true;
		try {
			const fields = {
				title: title.trim(),
				body,
				kind,
				severity,
				points: estimate > 0 ? Math.min(Math.round(estimate), MAX_POINTS) : null,
				start_on: startOn || null,
				due_on: dueOn || null,
				crew_agent: crewAgent,
			};
			if (issue === null) {
				await projectApi(projectId, 'issues', { method: 'POST', body: { ...fields, start_now: workNow } });
			} else {
				await projectApi(projectId, `issues/${issue.number}`, { method: 'PATCH', body: fields });
				if (workNow && !taken) await moveToReady(issue.number);
			}
			open = false;
			await onsaved();
		} catch {
			error = m.issues_err_save();
		} finally {
			saving = false;
		}
	}
</script>

<BottomSheet bind:open full label={issue === null ? m.issues_new() : `#${issue.number}`}>
	<form class="sheet" onsubmit={save}>
		<header class="sheet__bar">
			<span class="sheet__crumb">{issue === null ? m.issues_new() : `#${issue.number}`}</span>
			<IconButton type="button" onclick={() => (open = false)} aria-label={m.issues_cancel()} title={m.issues_cancel()}><IconClose size={20} /></IconButton>
		</header>

		<div class="sheet__page">
			<label class="sr-only" for="issue-title">{m.issues_field_title()}</label>
			<!-- svelte-ignore a11y_autofocus -->
			<textarea
				id="issue-title"
				class="sheet__title"
				rows="1"
				maxlength="200"
				placeholder={m.issues_title_placeholder()}
				autofocus={issue === null}
				bind:value={title}
				onkeydown={(event) => event.key === 'Enter' && event.preventDefault()}
			></textarea>

			<div class="props">
				<Select label={m.issues_type()} name="kind" bind:value={kind} options={ISSUE_KINDS.map((k) => ({ value: k, label: kindLabel(k) }))} />
				<Select label={m.issues_severity()} name="severity" bind:value={severity} options={ISSUE_SEVERITIES.map((s) => ({ value: s, label: severityLabel(s) }))} />
				<label class="points">
					<span class="points__label">{m.issues_field_points()}</span>
					<input type="number" inputmode="numeric" name="points" min="1" max={MAX_POINTS} step="1" list="issue-points" placeholder="–" bind:value={points} />
					<datalist id="issue-points">
						{#each POINT_SUGGESTIONS as p (p)}<option value={p}></option>{/each}
					</datalist>
				</label>
				<CrewPicker bind:value={crewAgent} label={m.issues_field_assignee()} {role} />
				<div class="start">
					<DatePicker label={m.issues_field_start()} bind:value={startOn} max={dueOn || undefined} placeholder={m.issues_date_none()} onchange={() => (workNow = workNow && startOn === localToday())} />
					{#if !taken}
						<button type="button" class="now" aria-pressed={workNow} onclick={toggleWorkNow}>
							<Zap size={16} />{m.issues_work_now()}
						</button>
					{/if}
				</div>
				<DatePicker label={m.issues_field_due()} bind:value={dueOn} min={startOn || undefined} placeholder={m.issues_date_none()} />
			</div>
			<p class="hint">{workNow ? m.issues_work_now_hint({ name: crewMember(crewAgent).name }) : m.issues_field_assignee_hint()}</p>

			<div class="body">
				<MarkdownEditor bind:value={body} label={m.issues_field_body()} placeholder={m.issues_body_placeholder()} />
			</div>
		</div>

		<footer class="sheet__actions">
			{#if error}<p class="error" role="alert">{error}</p>{/if}
			<Button variant="text" type="button" onclick={() => (open = false)}>{m.issues_cancel()}</Button>
			<Button variant="filled" type="submit" disabled={saving || !title.trim()}>
				{#if workNow}{m.issues_create_and_start()}{:else if issue === null}{m.issues_create()}{:else}{m.issues_save()}{/if}
			</Button>
		</footer>
	</form>
</BottomSheet>

<style>
	.sheet {
		display: flex;
		flex: 1;
		flex-direction: column;
		min-height: 0;
	}
	.sheet__bar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 0 12px 6px 20px;
	}
	.sheet__crumb {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
		font-weight: 600;
	}
	.sheet__page {
		flex: 1;
		box-sizing: border-box;
		width: 100%;
		max-width: 820px;
		min-height: 0;
		margin: 0 auto;
		padding: 8px clamp(16px, 4vw, 40px) 24px;
		overflow-y: auto;
		overscroll-behavior: contain;
	}
	.sheet__title {
		box-sizing: border-box;
		width: 100%;
		margin: 0 0 18px;
		padding: 0;
		border: none;
		background: none;
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-ref-typeface-brand);
		font-size: clamp(1.5rem, 4vw, 2rem);
		font-weight: 700;
		line-height: 1.2;
		letter-spacing: -0.015em;
		resize: none;
		field-sizing: content;
		outline: none;
	}
	.sheet__title::placeholder {
		color: var(--md-sys-color-on-surface-variant);
		opacity: 0.6;
	}
	.sheet__title:focus-visible {
		border-radius: 4px;
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 4px;
	}
	.props {
		display: grid;
		grid-template-columns: repeat(3, minmax(0, 1fr));
		gap: 14px 12px;
		align-items: start;
	}
	.start {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.now {
		display: inline-flex;
		align-self: flex-start;
		align-items: center;
		gap: 6px;
		min-height: 36px;
		padding: 0 12px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-full);
		background: none;
		color: var(--md-sys-color-on-surface);
		font: inherit;
		font-size: 0.8125rem;
		font-weight: 600;
		cursor: pointer;
	}
	.now:hover {
		background: var(--md-sys-color-surface-container-highest);
	}
	.now:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.now[aria-pressed='true'] {
		border-color: transparent;
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.points {
		display: flex;
		flex-direction: column;
		gap: 4px;
		min-width: 0;
	}
	.points__label {
		font-size: var(--md-sys-typescale-body-small-size);
		color: var(--md-sys-color-on-surface-variant);
	}
	.points input {
		box-sizing: border-box;
		width: 100%;
		height: 48px;
		padding: 0 14px;
		border: 1.5px solid var(--md-sys-color-outline);
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface);
		color: var(--md-sys-color-on-surface);
		font: inherit;
		font-size: var(--md-sys-typescale-body-large-size);
	}
	.points input:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 4%, var(--md-sys-color-surface));
	}
	.points input:focus-visible {
		outline: none;
		border: 2px solid var(--md-sys-color-primary);
		padding: 0 13.5px;
	}
	.hint {
		margin: 10px 0 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: var(--md-sys-typescale-body-small-size);
	}
	.body {
		margin-top: 20px;
		padding-top: 16px;
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}
	.sheet__actions {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: flex-end;
		gap: 8px;
		padding: 10px clamp(16px, 4vw, 40px) calc(12px + env(safe-area-inset-bottom));
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}
	.error {
		margin: 0 auto 0 0;
		color: var(--md-sys-color-error);
	}
	@media (max-width: 720px) {
		.props {
			grid-template-columns: repeat(2, minmax(0, 1fr));
		}
	}
	@media (max-width: 480px) {
		.props {
			grid-template-columns: minmax(0, 1fr);
		}
		.error {
			flex-basis: 100%;
		}
	}
</style>
