<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { enhance, deserialize } from '$app/forms';
	import { onMount, tick } from 'svelte';
	import {
		ArrowRight,
		ArrowUpRight,
		ChevronDown,
		Database,
		ExternalLink,
		FileCode,
		GitBranch,
		GitFork,
		Globe,
		MoreHorizontal,
		Plus,
		Rocket,
		Search as SearchIcon,
		Square,
		Trash2,
		UserPlus,
		Users,
		Wrench,
		X
	} from '@lucide/svelte';
	import Dialog from '$lib/components/m3/Dialog.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import OrgPicker from '$lib/components/OrgPicker.svelte';
	import Icon from '$lib/components/ds/foundations/Icon.svelte';
	import Button from '$lib/components/ds/actions/Button.svelte';
	import IconButton from '$lib/components/ds/actions/IconButton.svelte';
	import Chip from '$lib/components/ds/inputs/Chip.svelte';
	import Status from '$lib/components/ds/status/Status.svelte';
	import SectionLabel from '$lib/components/ds/status/SectionLabel.svelte';
	import KVRow from '$lib/components/ds/conversation/KVRow.svelte';
	import Mascot from '$lib/components/ds/agents/Mascot.svelte';
	import Menu from '$lib/components/ds/overlays/Menu.svelte';
	import ExportDialog from '$lib/components/ds/build/ExportDialog.svelte';
	import GithubMark from '$lib/components/ds/foundations/GithubMark.svelte';
	import ImportCode from '$lib/components/project/ImportCode.svelte';
	import { timeAgo } from '$lib/i18n';
	import type { ActionData, PageData } from './$types';
	import type { ProjectSummary } from '$lib/types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	const currentOrg = $derived(data.orgs.find((org) => org.id === data.orgId) ?? null);
	const orgError = $derived(form && 'orgError' in form ? (form.orgError as string) : null);

	// Org switcher
	let switcherOpen = $state(false);
	let newOrgOpen = $state(false);
	let newOrgName = $state('');
	let switchForm: HTMLFormElement | undefined = $state();
	let switchTo = $state('');

	function switchOrg(orgId: string) {
		switcherOpen = false;
		if (orgId === data.orgId) return;
		switchTo = orgId;
		queueMicrotask(() => switchForm?.requestSubmit());
	}

	// Code Import
	let importOpen = $state(false);
	let importSession = $state<string | null>(null);
	let importProject = $state<string | null>(null);
	async function prepareImport(): Promise<string> {
		if (importProject) return importProject;
		const body = new FormData();
		body.set('org_id', currentOrg?.id ?? '');
		const response = await fetch('?/createForImport', { method: 'POST', body, headers: { 'x-sveltekit-action': 'true' } });
		const result = deserialize(await response.text());
		const made = result.type === 'success' ? (result.data?.imported as { session_id: string; project_id: string } | undefined) : undefined;
		if (!made) throw new Error(m.err_new_project());
		importSession = made.session_id;
		importProject = made.project_id;
		return made.project_id;
	}

	// Code Export
	let exportOpen = $state(false);
	let exportProject = $state<ProjectSummary | null>(null);

	// Delete Project Dialog
	let deleteDialogOpen = $state(false);
	let projectToDelete = $state<ProjectSummary | null>(null);
	let deleteConfirmName = $state('');

	// Prompt builder input
	let promptText = $state('');
	let promptInputEl = $state<HTMLInputElement | null>(null);
	let createFormEl = $state<HTMLFormElement | null>(null);

	function startBuilding() {
		if (!createFormEl) return;
		createFormEl.requestSubmit();
	}

	// Selection & Filtering
	let selectedProject = $state<ProjectSummary | null>(null);
	let filter = $state<'all' | 'live' | 'building' | 'attention'>('all');
	let query = $state('');
	let sortBy = $state<'updated' | 'name'>('updated');

	// Color tones for project letter tiles
	const TONES: Record<string, { bg: string; text: string }> = {
		green: { bg: 'color-mix(in srgb, #22c55e 18%, var(--md-sys-color-surface-container-high))', text: '#22c55e' },
		orange: { bg: 'color-mix(in srgb, #f97316 18%, var(--md-sys-color-surface-container-high))', text: '#f97316' },
		yellow: { bg: 'color-mix(in srgb, #eab308 18%, var(--md-sys-color-surface-container-high))', text: '#eab308' },
		blue: { bg: 'color-mix(in srgb, #3b82f6 18%, var(--md-sys-color-surface-container-high))', text: '#3b82f6' },
		purple: { bg: 'color-mix(in srgb, #a855f7 18%, var(--md-sys-color-surface-container-high))', text: '#a855f7' },
		pink: { bg: 'color-mix(in srgb, #ec4899 18%, var(--md-sys-color-surface-container-high))', text: '#ec4899' },
	};

	function toneFor(name: string) {
		const keys = ['green', 'orange', 'yellow', 'blue', 'purple', 'pink'];
		let hash = 0;
		for (let i = 0; i < name.length; i++) hash = (hash << 5) - hash + name.charCodeAt(i);
		const key = keys[Math.abs(hash) % keys.length];
		return TONES[key];
	}

	function projectStatusKind(project: ProjectSummary): 'live' | 'working' | 'failed' | 'idle' {
		if (project.open_issues > 0) return 'failed';
		if (project.status === 'ready') return 'live';
		if (project.status === 'building') return 'working';
		return 'idle';
	}

	function projectSubdomain(project: ProjectSummary): string {
		const slug = project.name.toLowerCase().replace(/[^a-z0-9]/g, '-');
		const org = currentOrg?.name?.toLowerCase().replace(/[^a-z0-9]/g, '') || 'trian';
		return `${slug}-${project.id.slice(0, 6)}.${org}.space`;
	}

	// Filtered & Sorted Projects
	const liveCount = $derived(data.projects.filter((p) => p.status === 'ready' && p.open_issues === 0).length);
	const buildingCount = $derived(data.projects.filter((p) => p.status === 'building').length);
	const attentionCount = $derived(data.projects.filter((p) => p.open_issues > 0).length);

	const filteredProjects = $derived.by(() => {
		let list = data.projects;
		if (filter === 'live') list = list.filter((p) => p.status === 'ready' && p.open_issues === 0);
		else if (filter === 'building') list = list.filter((p) => p.status === 'building');
		else if (filter === 'attention') list = list.filter((p) => p.open_issues > 0);

		if (query.trim()) {
			const q = query.trim().toLowerCase();
			list = list.filter((p) => p.name.toLowerCase().includes(q) || (p.description ?? '').toLowerCase().includes(q));
		}

		return [...list].sort((a, b) => {
			if (sortBy === 'name') return a.name.localeCompare(b.name);
			return new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime();
		});
	});

	// Chats related to selected project
	const selectedProjectChats = $derived.by(() => {
		if (!selectedProject) return [];
		return (data.sessions ?? []).filter(
			(s) => s.project_id === selectedProject?.id || s.in_project_id === selectedProject?.id
		);
	});

	// Select first project by default on wide desktop screens if none selected
	onMount(() => {
		if (!selectedProject && data.projects.length > 0 && window.innerWidth >= 1280) {
			selectedProject = data.projects[0];
		}
	});

	interface RentalTemplate {
		title: string;
		author: string;
		location: string;
		remixes: number;
		prompt: string;
		previewType: 'rental';
		rows: { name: string; status: string; statusColor: string }[];
	}

	interface MenuTemplate {
		title: string;
		author: string;
		location: string;
		remixes: number;
		prompt: string;
		previewType: 'menu';
		rows: { name: string; price: string }[];
	}

	interface QuizTemplate {
		title: string;
		author: string;
		location: string;
		remixes: number;
		prompt: string;
		previewType: 'quiz';
		question: string;
		options: string[];
	}

	type CommunityTemplate = RentalTemplate | MenuTemplate | QuizTemplate;

	// Community Remix Templates
	const COMMUNITY_TEMPLATES: CommunityTemplate[] = [
		{
			title: 'Kos Melati rent tracker',
			author: 'Bu Sari',
			location: 'Yogyakarta',
			remixes: 214,
			prompt: 'Build an app like Kos Melati rent tracker: room rental tracking and payment management with monthly due dates and paid statuses.',
			previewType: 'rental',
			rows: [
				{ name: 'Room 1', status: 'paid', statusColor: 'text-emerald-400' },
				{ name: 'Room 2', status: 'due', statusColor: 'text-amber-400' },
				{ name: 'Room 3', status: 'paid', statusColor: 'text-emerald-400' },
			]
		},
		{
			title: 'Warung pre-order',
			author: 'Mas Adit',
			location: 'Semarang',
			remixes: 187,
			prompt: 'Build an app like Warung pre-order: online food menu with ordering and instant pricing calculation.',
			previewType: 'menu',
			rows: [
				{ name: 'Nasi rames', price: '15k' },
				{ name: 'Soto ayam', price: '18k' },
			]
		},
		{
			title: 'Quiz Biologi kelas 11',
			author: 'Bu Wulan',
			location: 'Malang',
			remixes: 156,
			prompt: 'Build an app like Quiz Biologi kelas 11: interactive biology multiple choice quiz with score calculation and explanations.',
			previewType: 'quiz',
			question: 'Organel energi sel…',
			options: ['Mitokondria', 'Ribosom'],
		}
	];

	async function remixTemplate(prompt: string) {
		promptText = prompt;
		await tick();
		promptInputEl?.scrollIntoView({ behavior: 'smooth', block: 'center' });
		promptInputEl?.focus();
	}
</script>

<Dialog bind:open={importOpen} headline={m.import_title()}>
	<p class="mb-4 text-ui text-text-2">{m.import_new_lede()}</p>
	<ImportCode
		projectId={importProject ?? undefined}
		prepare={prepareImport}
		ondone={() => {
			if (importSession) location.href = `/projects/session/${importSession}`;
		}}
	/>
</Dialog>

{#if exportProject}
	<ExportDialog
		bind:open={exportOpen}
		project={exportProject.name}
		repo={`${exportProject.files_count ?? 0} files · git history · main@${exportProject.id.slice(0, 7)}`}
		database="database.sql"
		env="DATABASE_URL · PUBLIC_SITE_URL"
		onpush={() => {}}
		ondownload={() => {
			if (exportProject) window.location.href = `/projects/${exportProject.id}/api/export/zip`;
		}}
		onclose={() => {
			exportOpen = false;
			exportProject = null;
		}}
	/>
{/if}

<!-- Delete Project Confirmation Dialog -->
<Dialog bind:open={deleteDialogOpen} headline={m.pj_delete()}>
	{#if projectToDelete}
		<form
			method="POST"
			action="?/deleteProject"
			use:enhance={() => {
				return async ({ result, update }) => {
					await update();
					if (result.type === 'success') {
						deleteDialogOpen = false;
						if (selectedProject?.id === projectToDelete?.id) {
							selectedProject = null;
						}
						projectToDelete = null;
						deleteConfirmName = '';
					}
				};
			}}
		>
			<input type="hidden" name="project_id" value={projectToDelete.id} />
			<p class="mb-4 text-ui text-text-2">
				{m.pj_delete_confirm({ name: projectToDelete.name })}
			</p>
			<p class="mb-2 text-caption text-text-3">
				{m.pj_delete_type()}: <strong class="font-mono text-text">{projectToDelete.name}</strong>
			</p>
			<TextField
				name="confirm_name"
				label={projectToDelete.name}
				bind:value={deleteConfirmName}
				required
			/>
			<div class="mt-6 flex justify-end gap-2">
				<Button label={m.org_cancel()} variant="ghost" onclick={() => (deleteDialogOpen = false)} />
				<button
					type="submit"
					disabled={deleteConfirmName !== projectToDelete.name}
					class="inline-flex items-center gap-1.5 rounded-md bg-danger px-3.5 py-1.5 text-ui font-medium text-white transition-opacity disabled:opacity-40 cursor-pointer disabled:cursor-not-allowed"
				>
					<Icon icon={Trash2} size={14} />
					<span>{m.common_delete()}</span>
				</button>
			</div>
		</form>
	{/if}
</Dialog>

<div class="min-h-[100dvh] w-full overflow-x-hidden overflow-y-auto bg-bg text-text pb-[calc(env(safe-area-inset-bottom,0px)+2rem)]">
	<!-- Top Bar -->
	<header class="flex min-h-14 w-full shrink-0 items-center justify-between border-b border-border px-3 sm:px-6 py-2 gap-2">
		<div class="flex items-center gap-2 sm:gap-3 min-w-0">
			{#if currentOrg}
				<div class="relative min-w-0">
					<button
						type="button"
						class="flex items-center gap-2 rounded-md px-2 py-1.5 text-ui font-medium text-text transition-colors hover:bg-surface-2 max-w-[180px] sm:max-w-none"
						onclick={() => (switcherOpen = !switcherOpen)}
						aria-haspopup="menu"
						aria-expanded={switcherOpen}
					>
						<span class="flex h-5 w-5 shrink-0 items-center justify-center rounded-sm bg-accent/20 font-mono text-caption font-semibold text-accent">
							{currentOrg.name.trim().charAt(0).toUpperCase()}
						</span>
						<span class="truncate">{currentOrg.name}</span>
						<Icon icon={ChevronDown} size={14} />
					</button>

					{#if switcherOpen}
						<!-- svelte-ignore a11y_click_events_have_key_events -->
						<!-- svelte-ignore a11y_no_static_element_interactions -->
						<div class="fixed inset-0 z-40" onclick={() => (switcherOpen = false)}></div>
						<div class="absolute top-full left-0 z-50 mt-1 min-w-48 overflow-hidden rounded-md border border-border bg-surface p-1 shadow-lg">
							{#each data.orgs as org (org.id)}
								<button
									type="button"
									class="flex w-full items-center justify-between rounded-sm px-3 py-2 text-left text-ui text-text transition-colors hover:bg-surface-2"
									onclick={() => switchOrg(org.id)}
								>
									<span>{org.name}</span>
									{#if org.id === data.orgId}
										<span class="text-accent text-caption">✓</span>
									{/if}
								</button>
							{/each}
							<div class="my-1 border-t border-border"></div>
							<button
								type="button"
								class="flex w-full items-center gap-2 rounded-sm px-3 py-2 text-left text-ui text-text transition-colors hover:bg-surface-2"
								onclick={() => { switcherOpen = false; newOrgOpen = true; }}
							>
								<Icon icon={Plus} size={14} />
								<span>{m.projects_new_org()}</span>
							</button>
						</div>
					{/if}
				</div>
				<span class="hidden sm:inline text-text-3 font-mono text-label">/</span>
				<span class="hidden sm:inline text-ui font-medium text-text-2">{m.projects_title()}</span>
			{/if}
		</div>

		{#if currentOrg}
			<div class="flex items-center gap-2 sm:gap-3 shrink-0">
				<div class="hidden sm:flex items-center gap-2">
					<div class="flex -space-x-1.5 overflow-hidden">
						{#if (data.members ?? []).length > 0}
							{#each (data.members.slice(0, 3)) as member (member.user_id)}
								<span
									class="inline-flex h-6 w-6 items-center justify-center rounded-full bg-surface-2 border border-border font-mono text-[10px] font-bold text-text ring-1 ring-bg"
									title={member.name}
								>
									{#if member.avatar_url}
										<img src={member.avatar_url} alt={member.name} class="h-full w-full rounded-full object-cover" />
									{:else}
										{member.name.slice(0, 2).toUpperCase()}
									{/if}
								</span>
							{/each}
						{:else}
							<span class="inline-flex h-6 w-6 items-center justify-center rounded-full bg-emerald-700 font-mono text-[10px] font-bold text-white ring-1 ring-bg">
								{currentOrg.name.slice(0, 1).toUpperCase()}
							</span>
						{/if}
					</div>
					<span class="text-label text-text-3 font-mono">
						{m.projects_members_count({ count: data.members?.length || currentOrg.members || 1 })}
					</span>
				</div>
				<a
					href="/organizations/{currentOrg.id}"
					class="inline-flex items-center gap-1.5 rounded-md border border-border px-2.5 py-1 text-caption sm:text-ui text-text transition-colors hover:bg-surface-2 hover:border-border-strong"
				>
					<span class="hidden sm:inline">{m.projects_invite_member()}</span>
					<Icon icon={UserPlus} size={13} />
				</a>
			</div>
			<form method="POST" action="?/chooseOrg" bind:this={switchForm} use:enhance hidden>
				<input type="hidden" name="org_id" value={switchTo} />
			</form>
		{/if}
	</header>

	{#if !currentOrg || (form && 'needsOrg' in form)}
		<div class="mx-auto max-w-xl p-4 sm:p-8">
			<section class="rounded-xl border border-border bg-surface p-5 sm:p-6 shadow-sm">
				<h2 class="text-h3 font-medium text-text">{m.org_pick_title()}</h2>
				<p class="mt-2 text-ui text-text-2">{m.org_pick_lede()}</p>
				<div class="mt-6">
					<OrgPicker orgs={data.orgs} current={data.orgId} error={orgError} />
				</div>
			</section>
		</div>
	{:else}
		<!-- Main Content Container with responsive side-by-side or slide-over drawer -->
		<div class="relative flex min-h-[calc(100%-3.5rem)] w-full">
			<!-- Main Left/Center column -->
			<div class="min-w-0 flex-1 p-4 sm:p-6 lg:p-8">
				<!-- Header Title & Lede -->
				<div class="mb-5 sm:mb-6">
					<h1 class="text-h2 font-semibold tracking-tight text-text">{m.projects_title()}</h1>
					<p class="mt-1.5 sm:mt-2 max-w-3xl text-caption sm:text-ui leading-relaxed text-text-2">{m.projects_hero_lede()}</p>
				</div>

				<!-- Describe an app to build (Hero Card) -->
				<form
					bind:this={createFormEl}
					method="POST"
					action="?/createProject"
					use:enhance
					class="mb-6 rounded-xl border border-border bg-surface p-3.5 sm:p-5 shadow-sm transition-colors focus-within:border-border-strong"
				>
					<input type="hidden" name="org_id" value={currentOrg.id} />
					<div class="flex items-center gap-3">
						<Mascot agent="koda" size={32} />
						<input
							bind:this={promptInputEl}
							type="text"
							name="prompt"
							bind:value={promptText}
							placeholder={m.projects_prompt_placeholder()}
							class="min-w-0 flex-1 border-0 bg-transparent text-[16px] sm:text-body text-text placeholder:text-text-3 focus:outline-none"
							onkeydown={(e) => {
								if (e.key === 'Enter') {
									e.preventDefault();
									startBuilding();
								}
							}}
						/>
					</div>
					<div class="mt-3.5 flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-t border-border pt-3">
						<div class="flex flex-wrap items-center gap-1.5 sm:gap-2 text-caption text-text-3">
							<span class="mr-1 text-ui text-text-3">{m.projects_koda_sets_up()}</span>
							<Chip kind="provision" icon={GitBranch} label={m.projects_tag_git()} />
							<Chip kind="provision" icon={Globe} label={`*.${currentOrg.name.toLowerCase().replace(/[^a-z0-9]/g, '') || 'trian'}.space`} />
							<Chip kind="provision" icon={Database} label={m.projects_tag_database()} />
						</div>
						<div class="flex items-center gap-2 w-full sm:w-auto">
							<button
								type="button"
								class="flex-1 sm:flex-none inline-flex items-center justify-center gap-2 rounded-md border border-border px-3 py-1.5 text-ui font-medium text-text transition-colors hover:bg-surface-2 cursor-pointer"
								onclick={() => (importOpen = true)}
							>
								<Icon icon={GithubMark} size={15} />
								<span>{m.projects_import_github()}</span>
							</button>
							<button
								type="submit"
								class="flex-1 sm:flex-none inline-flex items-center justify-center gap-2 rounded-md bg-emerald-600 px-3.5 py-1.5 text-ui font-medium text-white transition-colors hover:bg-emerald-500 cursor-pointer"
							>
								<span>{m.projects_start_building()}</span>
								<Icon icon={ArrowRight} size={15} />
							</button>
						</div>
					</div>
				</form>

				<!-- Filter & Search Toolbar -->
				<div class="mb-6 flex flex-col sm:flex-row sm:items-center justify-between gap-3 sm:gap-4">
					<!-- Horizontally scrollable filter tabs (iOS / mobile friendly) -->
					<div class="flex items-center overflow-x-auto no-scrollbar max-w-full rounded-lg border border-border bg-surface p-0.5 -mx-1 px-1 sm:mx-0 sm:px-0.5">
						<button
							type="button"
							class="shrink-0 whitespace-nowrap rounded-md px-2.5 sm:px-3 py-1.5 text-caption sm:text-ui font-medium transition-colors cursor-pointer {filter === 'all' ? 'bg-surface-2 text-text font-semibold shadow-xs' : 'text-text-2 hover:text-text'}"
							onclick={() => (filter = 'all')}
						>
							{m.projects_filter_all()} {data.projects.length}
						</button>
						<button
							type="button"
							class="shrink-0 whitespace-nowrap rounded-md px-2.5 sm:px-3 py-1.5 text-caption sm:text-ui font-medium transition-colors cursor-pointer {filter === 'live' ? 'bg-surface-2 text-text font-semibold shadow-xs' : 'text-text-2 hover:text-text'}"
							onclick={() => (filter = 'live')}
						>
							{m.projects_filter_live()} {liveCount}
						</button>
						<button
							type="button"
							class="shrink-0 whitespace-nowrap rounded-md px-2.5 sm:px-3 py-1.5 text-caption sm:text-ui font-medium transition-colors cursor-pointer {filter === 'building' ? 'bg-surface-2 text-text font-semibold shadow-xs' : 'text-text-2 hover:text-text'}"
							onclick={() => (filter = 'building')}
						>
							{m.projects_filter_building()} {buildingCount}
						</button>
						<button
							type="button"
							class="shrink-0 whitespace-nowrap rounded-md px-2.5 sm:px-3 py-1.5 text-caption sm:text-ui font-medium transition-colors cursor-pointer {filter === 'attention' ? 'bg-surface-2 text-text font-semibold shadow-xs' : 'text-text-2 hover:text-text'}"
							onclick={() => (filter = 'attention')}
						>
							{m.projects_filter_attention()} {attentionCount}
						</button>
					</div>

					<!-- Search & Sort -->
					<div class="flex w-full sm:w-auto items-center gap-2 sm:gap-3">
						<div class="relative flex min-w-0 flex-1 sm:flex-none items-center">
							<span class="pointer-events-none absolute left-3 text-text-3">
								<Icon icon={SearchIcon} size={15} />
							</span>
							<input
								type="search"
								bind:value={query}
								placeholder={m.projects_search()}
								class="h-9 w-full sm:w-56 lg:w-64 rounded-md border border-border bg-surface pl-9 pr-3 text-[16px] sm:text-ui text-text placeholder:text-text-3 transition-colors focus:border-border-strong focus:outline-none"
							/>
						</div>
						<button
							type="button"
							class="flex h-9 shrink-0 items-center gap-1.5 rounded-md border border-border bg-surface px-3 text-caption sm:text-ui text-text-2 transition-colors hover:bg-surface-2 hover:text-text cursor-pointer"
							onclick={() => (sortBy = sortBy === 'updated' ? 'name' : 'updated')}
						>
							<span>{sortBy === 'updated' ? m.projects_sort_updated() : m.projects_sort_name()}</span>
							<Icon icon={ChevronDown} size={14} />
						</button>
					</div>
				</div>

				<!-- Projects Grid -->
				{#if filteredProjects.length === 0}
					<div class="rounded-xl border border-dashed border-border p-8 sm:p-12 text-center">
						<p class="text-body text-text-2">{query ? m.list_no_match({ query }) : m.projects_empty()}</p>
					</div>
				{:else}
					<div class="grid grid-cols-1 gap-3.5 sm:gap-4 lg:grid-cols-2">
						{#each filteredProjects as project (project.id)}
							{@const tone = toneFor(project.name)}
							{@const statusKind = projectStatusKind(project)}
							{@const isSelected = selectedProject?.id === project.id}
							<!-- svelte-ignore a11y_click_events_have_key_events -->
							<!-- svelte-ignore a11y_no_static_element_interactions -->
							<div
								class="group flex flex-col justify-between rounded-xl border bg-surface p-3.5 sm:p-4 transition-all duration-150 cursor-pointer {isSelected ? 'border-text ring-1 ring-text shadow-sm' : 'border-border hover:border-border-strong hover:bg-surface-2/40'}"
								onclick={() => (selectedProject = project)}
							>
								<!-- Card Top -->
								<div>
									<div class="flex items-start justify-between gap-3">
										<div class="flex items-start gap-3 min-w-0">
											<span
												class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg font-mono text-title font-semibold"
												style="background: {tone.bg}; color: {tone.text};"
											>
												{project.name.charAt(0).toUpperCase()}
											</span>
											<div class="min-w-0">
												<h3 class="truncate text-body font-semibold text-text group-hover:text-accent">
													{project.name}
												</h3>
												<p class="mt-0.5 truncate text-label text-text-2">
													{project.description || m.projects_no_description()}
												</p>
											</div>
										</div>
										<div class="relative shrink-0" onclick={(e) => e.stopPropagation()}>
											<Menu
												items={[
													{
														label: m.projects_action_open(),
														icon: ArrowRight,
														onselect: () => {
															window.location.href = `/projects/session/${project.session_id}`;
														}
													},
													{
														label: m.export_repo(),
														icon: GitBranch,
														onselect: () => {
															exportProject = project;
															exportOpen = true;
														}
													},
													{
														label: m.projects_visit(),
														icon: ExternalLink,
														onselect: () => {
															window.open(project.deploy_url || `https://${projectSubdomain(project)}`, '_blank');
														}
													},
													'divider',
													{
														label: m.pj_delete(),
														icon: Trash2,
														onselect: () => {
															projectToDelete = project;
															deleteConfirmName = '';
															deleteDialogOpen = true;
														}
													}
												]}
											>
												{#snippet trigger(props)}
													<button
														type="button"
														class="flex h-8 w-8 items-center justify-center rounded-md text-text-3 transition-colors hover:bg-surface-2 hover:text-text cursor-pointer"
														{...props}
													>
														<Icon icon={MoreHorizontal} size={16} />
													</button>
												{/snippet}
											</Menu>
										</div>
									</div>

									<!-- Status Row -->
									<div class="mt-3.5 flex items-center gap-2">
										{#if statusKind === 'working'}
											<Status kind="working" label={m.projects_koda_is_building()} agent="koda" small />
											<span class="truncate font-mono text-caption text-text-3">
												{project.stack ? `${project.stack} app` : 'writing code'}
											</span>
										{:else if statusKind === 'live'}
											<Status kind="live" label={m.projects_live_dot()} small />
											<span class="truncate font-mono text-caption text-text-3">
												{project.deploy_url || projectSubdomain(project)}
											</span>
										{:else if statusKind === 'failed'}
											<Status kind="failed" label={m.projects_deploy_failed()} small />
											<span class="truncate font-mono text-caption text-text-3">{project.open_issues} issue(s)</span>
										{:else}
											<Status kind="idle" label={m.projects_draft()} small />
											<span class="truncate font-mono text-caption text-text-3">not deployed yet</span>
										{/if}
									</div>
								</div>

								<!-- Card Footer -->
								<div class="mt-4 flex items-center justify-between border-t border-border pt-3">
									<span class="font-mono text-caption text-text-3 truncate mr-2">
										{project.chats_count ?? 1} {project.chats_count === 1 ? 'chat' : 'chats'} · {project.deploys_count ? `deploy #${project.deploys_count}` : 'draft'} · {timeAgo(project.updated_at)}
									</span>

									<div onclick={(e) => e.stopPropagation()} class="shrink-0">
										{#if statusKind === 'working'}
											<button
												type="button"
												class="inline-flex items-center gap-1.5 rounded-md border border-border px-3 py-1 text-label text-text transition-colors hover:bg-surface-2 cursor-pointer"
												onclick={() => (selectedProject = project)}
											>
												<Icon icon={Square} size={12} />
												<span>{m.projects_action_stop()}</span>
											</button>
										{:else if statusKind === 'failed'}
											<a
												href="/projects/session/{project.session_id}?prompt=Fix the build errors"
												class="inline-flex items-center gap-1.5 rounded-md border border-border px-3 py-1 text-label text-text transition-colors hover:bg-surface-2 hover:border-border-strong"
											>
												<span>{m.projects_action_fix()}</span>
												<Icon icon={Wrench} size={13} />
											</a>
										{:else}
											<a
												href="/projects/session/{project.session_id}"
												class="inline-flex items-center gap-1.5 rounded-md border border-border px-3 py-1 text-label text-text transition-colors hover:bg-surface-2 hover:border-border-strong"
											>
												<span>{m.projects_action_open()}</span>
												<Icon icon={ArrowRight} size={13} />
											</a>
										{/if}
									</div>
								</div>
							</div>
						{/each}
					</div>
				{/if}

				<!-- Remix from the community Section -->
				<section class="mt-10 sm:mt-12 border-t border-border pt-6 sm:pt-8">
					<div class="mb-4 flex items-center justify-between gap-3">
						<div>
							<div class="flex items-center gap-2">
								<Icon icon={GitFork} size={16} />
								<h2 class="text-body font-semibold text-text">{m.projects_remix_community()}</h2>
							</div>
							<p class="mt-0.5 text-label text-text-2">{m.projects_remix_lede()}</p>
						</div>
						<a
							href="/showcase"
							class="inline-flex shrink-0 items-center gap-1 text-ui text-text-2 transition-colors hover:text-text"
						>
							<span>{m.projects_browse_showcase()}</span>
							<Icon icon={ArrowUpRight} size={14} />
						</a>
					</div>

					<div class="grid grid-cols-1 gap-3.5 sm:gap-4 md:grid-cols-3">
						{#each COMMUNITY_TEMPLATES as tmpl (tmpl.title)}
							<div class="flex flex-col justify-between rounded-xl border border-border bg-surface p-3.5 sm:p-4 shadow-sm transition-colors hover:border-border-strong">
								<!-- Mock App UI Box -->
								<div class="mb-3 flex h-28 flex-col justify-center rounded-lg bg-black/40 p-3 font-mono text-caption text-text-2">
									{#if tmpl.previewType === 'rental'}
										{@const rental = tmpl as RentalTemplate}
										<div class="space-y-1.5">
											{#each rental.rows as row}
												<div class="flex justify-between">
													<span>{row.name}</span>
													<span class={row.statusColor}>{row.status}</span>
												</div>
											{/each}
										</div>
									{:else if tmpl.previewType === 'menu'}
										{@const menu = tmpl as MenuTemplate}
										<div class="space-y-1.5">
											{#each menu.rows as row}
												<div class="flex justify-between">
													<span>{row.name}</span>
													<span class="text-text-3">{row.price}</span>
												</div>
											{/each}
										</div>
									{:else}
										{@const quiz = tmpl as QuizTemplate}
										<div>
											<p class="text-[11px] text-text-3">{quiz.question}</p>
											{#if quiz.options[0]}
												<div class="mt-2 rounded border border-emerald-500/50 bg-emerald-500/10 px-2 py-0.5 text-[11px] text-emerald-300">
													{quiz.options[0]}
												</div>
											{/if}
											{#if quiz.options[1]}
												<div class="mt-1 px-2 text-[11px] text-text-3">
													{quiz.options[1]}
												</div>
											{/if}
										</div>
									{/if}
								</div>

								<!-- Template Meta & Remix Button -->
								<div>
									<div class="flex items-start justify-between gap-2">
										<div class="min-w-0">
											<h4 class="truncate text-ui font-semibold text-text">{tmpl.title}</h4>
											<p class="text-caption text-text-3">{tmpl.author} · {tmpl.location}</p>
										</div>
										<button
											type="button"
											class="inline-flex shrink-0 items-center gap-1 rounded-md border border-border px-2.5 py-1 text-caption font-medium text-text transition-colors hover:bg-surface-2 cursor-pointer"
											onclick={() => remixTemplate(tmpl.prompt)}
										>
											<span>{m.projects_remix_btn()}</span>
											<Icon icon={GitFork} size={12} />
										</button>
									</div>
									<div class="mt-2.5 flex items-center gap-1.5 border-t border-border pt-2 text-[11px] text-text-3">
										<span class="h-2 w-2 rounded-full bg-amber-600/70"></span>
										<span>{tmpl.author} · {tmpl.location} · {tmpl.remixes} remixes</span>
									</div>
								</div>
							</div>
						{/each}
					</div>
				</section>
			</div>

			<!-- Right Details Panel: Slide-over drawer on mobile (< xl), persistent side-panel on desktop (>= xl) -->
			{#if selectedProject}
				{@const project = selectedProject}
				{@const tone = toneFor(project.name)}
				{@const statusKind = projectStatusKind(project)}

				<!-- Mobile backdrop overlay -->
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div
					class="fixed inset-0 z-50 bg-black/60 backdrop-blur-xs transition-opacity xl:hidden"
					onclick={() => (selectedProject = null)}
				></div>

				<aside
					class="fixed inset-y-0 right-0 z-50 flex h-full w-full max-w-md flex-col overflow-y-auto border-l border-border bg-surface p-4 sm:p-5 shadow-2xl transition-transform duration-200 ease-out xl:static xl:z-auto xl:h-auto xl:max-w-none xl:w-[380px] xl:shadow-none xl:shrink-0"
				>
					<!-- Panel Header -->
					<div class="flex items-start justify-between gap-3">
						<div class="flex items-center gap-3 min-w-0">
							<span
								class="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg font-mono text-ui font-semibold"
								style="background: {tone.bg}; color: {tone.text};"
							>
								{project.name.charAt(0).toUpperCase()}
							</span>
							<div class="min-w-0">
								<h3 class="truncate text-body font-semibold text-text">{project.name}</h3>
								<div class="flex items-center gap-1.5 text-caption text-text-3">
									<span class="h-1.5 w-1.5 rounded-full {statusKind === 'live' ? 'bg-emerald-500' : 'bg-text-3'}"></span>
									<span>
										{statusKind} · {project.deploys_count ? `deploy #${project.deploys_count}` : 'draft'} · {timeAgo(project.updated_at)}
									</span>
								</div>
							</div>
						</div>
						<IconButton icon={X} label={m.action_close()} variant="ghost" onclick={() => (selectedProject = null)} />
					</div>

					<!-- Visual Dynamic App Preview Widget -->
					<div class="mt-4 sm:mt-5 overflow-hidden rounded-xl border border-border bg-surface-2 p-4 text-text shadow-sm">
						<div class="flex items-center justify-between border-b border-border pb-2.5 mb-3 font-mono text-[10px] tracking-wider text-text-3 uppercase">
							<div class="flex items-center gap-1.5">
								<span class="h-2 w-2 rounded-full {statusKind === 'live' ? 'bg-emerald-500 animate-pulse' : 'bg-text-3'}"></span>
								<span>{project.stack ? `${project.stack.toUpperCase()} APP` : 'APPLICATION PREVIEW'}</span>
							</div>
							{#if project.deploy_url}
								<span class="text-emerald-500 font-semibold">LIVE</span>
							{/if}
						</div>
						<h4 class="text-title font-semibold text-text">{project.name}</h4>
						<p class="mt-1 text-caption text-text-2 line-clamp-2">{project.description || m.projects_hero_lede()}</p>
						<div class="mt-3 flex flex-wrap items-center gap-2">
							<span class="inline-flex items-center gap-1 rounded bg-surface px-2 py-0.5 font-mono text-caption text-text-2 border border-border">
								<Icon icon={FileCode} size={12} />
								<span>{project.files_count ?? 0} files</span>
							</span>
							<span class="inline-flex items-center gap-1 rounded bg-surface px-2 py-0.5 font-mono text-caption text-text-2 border border-border">
								<Icon icon={Database} size={12} />
								<span>{project.backend === 'relay' ? 'Relay' : (project.backend === 'drizzle' ? 'PostgreSQL' : 'None')}</span>
							</span>
							{#if project.deploys_count}
								<span class="inline-flex items-center gap-1 rounded bg-emerald-500/10 px-2 py-0.5 font-mono text-caption text-emerald-500 border border-emerald-500/20">
									<Icon icon={Rocket} size={12} />
									<span>deploy #{project.deploys_count}</span>
								</span>
							{/if}
						</div>
					</div>

					<!-- Panel Action Buttons -->
					<div class="mt-4 flex flex-col gap-2">
						<a
							href="/projects/session/{project.session_id}"
							class="flex w-full items-center justify-center gap-2 rounded-lg bg-text py-2.5 text-ui font-medium text-bg transition-opacity hover:opacity-90"
						>
							<span>{m.projects_open_workspace()}</span>
							<Icon icon={ArrowRight} size={15} />
						</a>
						<a
							href={project.deploy_url || `https://${projectSubdomain(project)}`}
							target="_blank"
							rel="noopener noreferrer"
							class="flex w-full items-center justify-center gap-1.5 rounded-lg border border-border py-2 text-ui text-text transition-colors hover:bg-surface-2"
						>
							<span>{m.projects_visit()}</span>
							<Icon icon={ExternalLink} size={14} />
						</a>
					</div>

					<!-- WHAT'S IN THIS PROJECT -->
					<div class="mt-5 sm:mt-6 border-t border-border pt-4 sm:pt-5">
						<SectionLabel label={m.projects_whats_in()} />
						<div class="mt-3 space-y-1">
							<div class="flex items-center justify-between text-caption">
								<KVRow icon={FileCode} label={m.projects_code_kv()} value={`${project.files_count ?? 0} files · main@${project.id.slice(0, 7)}`} />
								<button
									type="button"
									class="inline-flex shrink-0 items-center gap-1 text-label text-text-2 hover:text-text cursor-pointer"
									onclick={() => {
										exportProject = project;
										exportOpen = true;
									}}
								>
									<Icon icon={GitBranch} size={12} />
									<span>Export</span>
								</button>
							</div>
							<KVRow icon={Globe} label={m.projects_url_kv()} value={project.deploy_url ? `...${project.deploy_url.slice(-24)}` : `...${projectSubdomain(project).slice(-20)}`} />
							<KVRow icon={Database} label={m.projects_data_kv()} value={project.backend === 'relay' ? 'Nomi Relay' : (project.backend === 'drizzle' ? 'PostgreSQL' : 'localStorage only · no server')} />
							<KVRow icon={Rocket} label={m.projects_deploys_kv()} value={project.deploys_count ? `${project.deploys_count} deploys` : 'Draft'} />
							<KVRow icon={Users} label={m.projects_members_kv()} value={`${currentOrg.name} (owner), you`} />
						</div>
					</div>

					<!-- CHATS IN THIS PROJECT -->
					<div class="mt-5 sm:mt-6 border-t border-border pt-4 sm:pt-5">
						<div class="flex items-center justify-between">
							<SectionLabel label={m.projects_chats_in()} count={selectedProjectChats.length || 1} />
						</div>

						<div class="mt-3 space-y-2">
							{#if selectedProjectChats.length > 0}
								{#each selectedProjectChats as chat (chat.id)}
									<a
										href="/projects/session/{chat.id}"
										class="flex items-center justify-between rounded-lg border border-border p-2.5 text-caption transition-colors hover:bg-surface-2"
									>
										<div class="flex items-center gap-2 min-w-0">
											<Mascot agent="koda" size={16} />
											<span class="truncate text-ui text-text font-medium">{chat.title || project.name}</span>
										</div>
										<span class="shrink-0 text-text-3 font-mono text-[11px]">{timeAgo(chat.updated_at)}</span>
									</a>
								{/each}
							{:else}
								<a
									href="/projects/session/{project.session_id}"
									class="flex items-center justify-between rounded-lg border border-border p-2.5 text-caption transition-colors hover:bg-surface-2"
								>
									<div class="flex items-center gap-2 min-w-0">
										<Mascot agent="koda" size={16} />
										<span class="truncate text-ui text-text font-medium">{project.name}</span>
									</div>
									<span class="shrink-0 text-text-3 font-mono text-[11px]">{timeAgo(project.updated_at)}</span>
								</a>
							{/if}

							<a
								href="/projects/session/{project.session_id}"
								class="mt-2 flex items-center gap-1.5 text-ui text-text-3 transition-colors hover:text-text"
							>
								<Icon icon={Plus} size={14} />
								<span>{m.projects_ask_koda_change()}</span>
							</a>
						</div>
					</div>
				</aside>
			{/if}
		</div>
	{/if}
</div>

<!-- New Organization Dialog -->
<Dialog bind:open={newOrgOpen} headline={m.org_new_title()}>
	<form
		id="new-org"
		method="POST"
		action="?/createOrg"
		use:enhance={() => async ({ update, result }) => {
			await update();
			if (result.type === 'success') {
				newOrgOpen = false;
				newOrgName = '';
			}
		}}
	>
		<p class="mb-4 text-ui text-text-2">{m.org_pick_new_hint()}</p>
		<TextField name="name" label={m.org_name()} bind:value={newOrgName} maxlength={80} required />
		{#if orgError && newOrgOpen}<p class="mt-2 text-caption text-danger" role="alert">{orgError}</p>{/if}
	</form>
	{#snippet actions()}
		<Button label={m.org_cancel()} variant="ghost" onclick={() => (newOrgOpen = false)} />
		<Button label={m.org_create()} variant="primary" type="submit" disabled={!newOrgName.trim()} />
	{/snippet}
</Dialog>
