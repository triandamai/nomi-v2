<script lang="ts">
	// Above the project chat's composer: the issues the crew took from Ready to dev, each one
	// line ("Koda is working on issue #1"), opening that issue's chat. It follows the board live.
	import { m } from '$lib/paraglide/messages';
	import { ChevronRight } from '@lucide/svelte';
	import Mascot from '$lib/components/ds/agents/Mascot.svelte';
	import { MAX_WORK_ROWS, crewMember, crewWork } from '$lib/issues';
	import { isIssuesEvent } from '$lib/presenceView';
	import { projectApi } from '$lib/projectApi';
	import type { IssueBoard, ProjectIssue } from '$lib/types';

	let {
		projectId,
		openIssues = 1,
		subscribe,
		onmore,
	}: {
		projectId: string;
		/** How many open issues the project has; when zero, nothing on the board can be in crew work. */
		openIssues?: number;
		/** The project's live events: the strip reloads when the board changes. */
		subscribe?: (listener: (raw: string) => void) => () => void;
		/** Opens the board, for the ones that don't fit. */
		onmore: () => void;
	} = $props();

	let work = $state<ProjectIssue[]>([]);

	async function load() {
		try {
			work = crewWork((await projectApi<IssueBoard>(projectId, 'board')).columns);
		} catch {
			// The strip is a convenience: the board itself says what went wrong.
		}
	}

	$effect(() => {
		void projectId;
		if (openIssues <= 0) {
			work = [];
			return;
		}
		void load();
	});

	$effect(() => {
		if (!subscribe) return;
		return subscribe((raw) => {
			if (isIssuesEvent(raw)) void load();
		});
	});

	const STATE_TEXT = {
		queued: m.issue_work_queued,
		working: m.issue_work_working,
		done: m.issue_work_done,
		failed: m.issue_work_failed,
	};

	const shown = $derived(work.slice(0, MAX_WORK_ROWS));
</script>

{#if work.length > 0}
	<ul class="work" aria-label={m.issue_work_label()}>
		{#each shown as issue (issue.number)}
			{@const who = crewMember(issue.crew_agent)}
			{@const text = STATE_TEXT[issue.crew_state ?? 'queued']({ name: who.name, number: issue.number })}
			<li>
				{#if issue.session_id}
					<a class="row" href="/projects/session/{issue.session_id}" data-state={issue.crew_state}>
						{#if who.crew}<Mascot agent={who.crew} size={20} />{/if}
						<span class="row__text"><span class="row__what">{text}</span><span class="row__title">{issue.title}</span></span>
						<ChevronRight size={18} />
					</a>
				{:else}
					<div class="row" data-state={issue.crew_state}>
						{#if who.crew}<Mascot agent={who.crew} size={20} />{/if}
						<span class="row__text"><span class="row__what">{text}</span><span class="row__title">{issue.title}</span></span>
					</div>
				{/if}
			</li>
		{/each}
		{#if work.length > shown.length}
			<li><button type="button" class="more" onclick={onmore}>{m.issue_work_more({ count: work.length - shown.length })}</button></li>
		{/if}
	</ul>
{/if}

<style>
	.work {
		display: flex;
		flex-direction: column;
		margin: 0 0 8px;
		padding: 2px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 12px;
		background: var(--md-sys-color-surface-container-low);
		list-style: none;
	}
	.row {
		display: flex;
		align-items: center;
		gap: 10px;
		min-height: 44px;
		padding: 4px 10px;
		border-radius: 10px;
		color: var(--md-sys-color-on-surface);
		text-decoration: none;
	}
	a.row:hover {
		background: var(--md-sys-color-surface-container-highest);
	}
	a.row:focus-visible,
	.more:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: -2px;
	}
	.row :global(svg) {
		flex: none;
		color: var(--md-sys-color-on-surface-variant);
	}
	.row__text {
		display: flex;
		flex: 1;
		align-items: baseline;
		gap: 8px;
		min-width: 0;
	}
	.row__what {
		flex: none;
		font-size: 0.875rem;
		font-weight: 600;
	}
	.row[data-state='failed'] .row__what {
		color: var(--md-sys-color-error);
	}
	.row__title {
		overflow: hidden;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.more {
		width: 100%;
		min-height: 40px;
		padding: 0 10px;
		border: none;
		border-radius: 10px;
		background: none;
		color: var(--md-sys-color-primary);
		font: inherit;
		font-size: 0.8125rem;
		font-weight: 600;
		text-align: left;
		cursor: pointer;
	}
	.more:hover {
		background: var(--md-sys-color-surface-container-highest);
	}
	@media (max-width: 600px) {
		.row__text {
			flex-direction: column;
			gap: 0;
		}
		.row__what {
			flex: initial;
		}
	}
</style>
