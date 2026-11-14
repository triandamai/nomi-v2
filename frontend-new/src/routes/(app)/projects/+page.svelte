<script lang="ts">
	import { Building2, CalendarCheck, FileText, FolderSearch, SearchX, Store } from '@lucide/svelte';
	import { goto, invalidate } from '$app/navigation';
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import { pageAction } from '$lib/pageAction';
	import { filterCounts, projectCards, visibleCards, type ProjectCardData } from '$lib/projects/list';
	import { toasts } from '$lib/ds/overlays/toasts.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import ProjectPageHeader from '$lib/ds/projects/ProjectPageHeader.svelte';
	import ProjectComposer from '$lib/ds/projects/ProjectComposer.svelte';
	import ProjectToolbar, { type ProjectFilter } from '$lib/ds/projects/ProjectToolbar.svelte';
	import type { ProjectSort } from '$lib/ds/projects/SortMenu.svelte';
	import ProjectCard from '$lib/ds/projects/ProjectCard.svelte';
	import ProjectEmpty from '$lib/ds/projects/ProjectEmpty.svelte';
	import FirstProject from '$lib/ds/projects/FirstProject.svelte';
	import ProjectDialog from '$lib/ds/projects/ProjectDialog.svelte';

	// Projects (App / Projects JyWON): the projects of the workspace they come from. The composer
	// starts one (its chat opens with Koda on it); the toolbar filters, finds and sorts; each card
	// says where the project is, and its menu opens, visits, exports or deletes it (asked first).
	// Before any project, the starters fill the composer; before a workspace is chosen, the page asks
	// for one. A card's footer action (Deploy, Stop, Resume) waits for the project's workspace, which
	// does them; until then a project opens from its menu, or from its address when it's live. Left
	// out for now (AGENTS.md §9, BACKEND_TODO.md): the detail panel, Community, importing, and the
	// menu's rename, duplicate, move, share and take down.
	let { data } = $props();

	const PATH = '/projects';
	const cards = $derived(projectCards(data.list.projects, new Date()));
	let filter = $state<ProjectFilter>('all');
	let query = $state('');
	let sort = $state<ProjectSort>('updated');
	const shown = $derived(visibleCards(cards, filter, query, sort));
	const counts = $derived(filterCounts(cards));

	const workspaceName = $derived(page.data.workspaces?.find((workspace: { id: string }) => workspace.id === data.list.org_id)?.name ?? '');
	// Each tab's own empty state (App / Projects → Empty / Tab eI37L draws Needs you's).
	const TAB_EMPTY: Record<ProjectFilter, { title: () => string; body: () => string }> = {
		all: { title: m.projects_tab_empty_title, body: m.projects_tab_empty_body },
		live: { title: m.projects_live_empty_title, body: m.projects_live_empty_body },
		building: { title: m.projects_building_empty_title, body: m.projects_building_empty_body },
		attention: { title: m.projects_attention_empty_title, body: m.projects_attention_empty_body },
	};

	let description = $state('');
	let starting = $state(false);
	async function build(text: string) {
		if (starting) return;
		starting = true;
		const result = await pageAction(PATH, 'start', { description: text, org: data.list.org_id ?? '' });
		starting = false;
		if (!result.ok && result.message) toasts.show({ kind: 'error', title: m.projects_start_failed(), detail: result.message });
	}

	const starters = [
		{ icon: CalendarCheck, label: m.projects_starter_booking(), meta: m.projects_starter_app_db() },
		{ icon: Store, label: m.projects_starter_warung(), meta: m.projects_starter_app_db() },
		{ icon: FileText, label: m.projects_starter_portfolio(), meta: m.projects_starter_site() },
	].map((starter) => ({ ...starter, onpick: (label: string) => (description = label) }));

	let deleting = $state<ProjectCardData | null>(null);
	let busy = $state(false);
	async function remove() {
		if (!deleting) return;
		busy = true;
		const name = deleting.name;
		const result = await pageAction(PATH, 'delete', { id: deleting.id });
		busy = false;
		if (!result.ok) {
			if (result.message) toasts.show({ kind: 'error', title: m.action_failed(), detail: result.message });
			return;
		}
		deleting = null;
		toasts.show({ kind: 'success', title: m.projects_deleted({ name }) });
		await invalidate('app:projects');
	}
</script>

<svelte:head><title>{m.nav_projects()} · {m.app_name()}</title></svelte:head>

<div class="h-full overflow-y-auto">
	<div class="flex max-w-260 flex-col gap-6 px-4 pt-8 pb-12 lg:px-8">
		<ProjectPageHeader />
		{#if !data.list.org_id}
			<ProjectEmpty
				icon={Building2}
				title={m.projects_no_workspace_title()}
				body={m.projects_no_workspace_body()}
				action={{ label: m.projects_choose_workspace(), onclick: () => goto('/settings/workspaces') }}
			/>
		{:else}
			<ProjectComposer bind:value={description} domain={data.list.base_domain ?? ''} onbuild={build} />
			{#if cards.length}
				<ProjectToolbar {counts} bind:filter bind:query bind:sort />
				{#if shown.length}
					<ul class="flex flex-wrap gap-4">
						{#each shown as card (card.id)}
							<li>
								<ProjectCard
									name={card.name}
									description={card.description}
									tone={card.tone}
									status={card.status}
									url={card.url}
									footer={card.footer}
									href={card.href}
									actions={{
										onopen: () => goto(card.href),
										...(card.url && { onvisit: () => window.open(card.url, '_blank', 'noopener') }),
										onexport: () => (location.href = `/projects/${card.id}/export`),
										ondelete: () => (deleting = card),
									}}
								/>
							</li>
						{/each}
					</ul>
				{:else if query.trim()}
					<ProjectEmpty
						icon={SearchX}
						title={m.projects_no_match_title({ query: query.trim() })}
						body={m.projects_no_match_body({ workspace: workspaceName })}
						action={{ label: m.projects_clear_search(), onclick: () => (query = '') }}
					/>
				{:else}
					<ProjectEmpty icon={FolderSearch} title={TAB_EMPTY[filter].title()} body={TAB_EMPTY[filter].body()} />
				{/if}
			{:else}
				<FirstProject {starters} />
			{/if}
		{/if}
	</div>
</div>

<ProjectDialog
	open={deleting !== null}
	title={m.projects_delete_title({ name: deleting?.name ?? '' })}
	description={m.projects_delete_body()}
	width={420}
	onclose={() => (deleting = null)}
>
	{#snippet actions()}
		<Button variant="secondary" label={m.action_cancel()} onclick={() => (deleting = null)} />
		<Button variant="danger" label={m.projects_delete()} loading={busy} onclick={remove} />
	{/snippet}
</ProjectDialog>
