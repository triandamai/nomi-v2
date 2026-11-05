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

	// Select first project by default on wide screens if none selected
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
		repo={`23 files · git history · main@${exportProject.id.slice(0, 7)}`}
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

<div class="h-full w-full overflow-y-auto bg-bg text-text">
	<!-- Top Bar -->
	<header class="flex h-14 w-full shrink-0 items-center justify-between border-b border-border px-6">
		<div class="flex items-center gap-3">
			{#if currentOrg}
				<div class="relative">
					<button
						type="button"
						class="flex items-center gap-2 rounded-md px-2 py-1.5 text-ui font-medium text-text transition-colors hover:bg-surface-2"
						onclick={() => (switcherOpen = !switcherOpen)}
						aria-haspopup="menu"
						aria-expanded={switcherOpen}
					>
						<span class="flex h-5 w-5 items-center justify-center rounded-sm bg-accent/20 font-mono text-caption font-semibold text-accent">
							{currentOrg.name.trim().charAt(0).toUpperCase()}
						</span>
						<span>{currentOrg.name}</span>
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
				<span class="text-text-3 font-mono text-label">/</span>
				<span class="text-ui font-medium text-text-2">{m.projects_title()}</span>
			{/if}
		</div>

		{#if currentOrg}
			<div class="flex items-center gap-3">
				<div class="flex items-center gap-2">
					<div class="flex -space-x-1.5 overflow-hidden">
						<span class="inline-flex h-6 w-6 items-center justify-center rounded-full bg-emerald-700 font-mono text-[10px] font-bold text-white ring-1 ring-bg">T</span>
						<span class="inline-flex h-6 w-6 items-center justify-center rounded-full bg-blue-700 font-mono text-[10px] font-bold text-white ring-1 ring-bg">MF</span>
						<span class="inline-flex h-6 w-6 items-center justify-center rounded-full bg-amber-700 font-mono text-[10px] font-bold text-white ring-1 ring-bg">SR</span>
					</div>
					<span class="text-label text-text-3 font-mono">{m.projects_members_count({ count: currentOrg.members || 3 })}</span>
				</div>
				<a
					href="/organizations/{currentOrg.id}"
					class="inline-flex items-center gap-1.5 rounded-md border border-border px-2.5 py-1 text-ui text-text transition-colors hover:bg-surface-2 hover:border-border-strong"
				>
					<span>{m.projects_invite_member()}</span>
					<Icon icon={UserPlus} size={13} />
				</a>
			</div>
			<form method="POST" action="?/chooseOrg" bind:this={switchForm} use:enhance hidden>
				<input type="hidden" name="org_id" value={switchTo} />
			</form>
		{/if}
	</header>

	{#if !currentOrg || (form && 'needsOrg' in form)}
		<div class="mx-auto max-w-xl p-8">
			<section class="rounded-xl border border-border bg-surface p-6 shadow-sm">
				<h2 class="text-h3 font-medium text-text">{m.org_pick_title()}</h2>
				<p class="mt-2 text-ui text-text-2">{m.org_pick_lede()}</p>
				<div class="mt-6">
					<OrgPicker orgs={data.orgs} current={data.orgId} error={orgError} />
				</div>
			</section>
		</div>
	{:else}
		<!-- Main Content Container with optional right drawer -->
		<div class="flex min-h-[calc(100%-3.5rem)] w-full">
			<!-- Main Left/Center column -->
			<div class="min-w-0 flex-1 p-6 lg:p-8">
				<!-- Header Title & Lede -->
				<div class="mb-6">
					<h1 class="text-h2 font-semibold tracking-tight text-text">{m.projects_title()}</h1>
					<p class="mt-2 max-w-3xl text-ui leading-relaxed text-text-2">{m.projects_hero_lede()}</p>
				</div>

				<!-- Describe an app to build (Hero Card) -->
				<form
					bind:this={createFormEl}
					method="POST"
					action="?/createProject"
					use:enhance
					class="mb-6 rounded-xl border border-border bg-surface p-4 shadow-sm transition-colors focus-within:border-border-strong"
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
							class="min-w-0 flex-1 border-0 bg-transparent text-body text-text placeholder:text-text-3 focus:outline-none"
							onkeydown={(e) => {
								if (e.key === 'Enter') {
									e.preventDefault();
									startBuilding();
								}
							}}
						/>
					</div>
					<div class="mt-3.5 flex flex-wrap items-center justify-between gap-3 border-t border-border pt-3">
						<div class="flex flex-wrap items-center gap-2 text-caption text-text-3">
							<span class="mr-1 text-ui text-text-3">{m.projects_koda_sets_up()}</span>
							<Chip kind="provision" icon={GitBranch} label={m.projects_tag_git()} />
							<Chip kind="provision" icon={Globe} label={`*.${currentOrg.name.toLowerCase().replace(/[^a-z0-9]/g, '') || 'trian'}.space`} />
							<Chip kind="provision" icon={Database} label={m.projects_tag_database()} />
						</div>
						<div class="flex items-center gap-2">
							<button
								type="button"
								class="inline-flex items-center gap-2 rounded-md border border-border px-3 py-1.5 text-ui font-medium text-text transition-colors hover:bg-surface-2"
								onclick={() => (importOpen = true)}
							>
								<Icon icon={GithubMark} size={15} />
								<span>{m.projects_import_github()}</span>
							</button>
							<button
								type="submit"
								class="inline-flex items-center gap-2 rounded-md bg-emerald-600 px-3.5 py-1.5 text-ui font-medium text-white transition-colors hover:bg-emerald-500 cursor-pointer"
							>
								<span>{m.projects_start_building()}</span>
								<Icon icon={ArrowRight} size={15} />
							</button>
						</div>
					</div>
				</form>

				<!-- Filter & Search Toolbar -->
				<div class="mb-6 flex flex-wrap items-center justify-between gap-4">
					<!-- Filter tabs -->
					<div class="inline-flex rounded-lg border border-border bg-surface p-0.5">
						<button
							type="button"
							class="rounded-md px-3 py-1.5 text-ui font-medium transition-colors cursor-pointer {filter === 'all' ? 'bg-surface-2 text-text font-semibold shadow-xs' : 'text-text-2 hover:text-text'}"
							onclick={() => (filter = 'all')}
						>
							{m.projects_filter_all()} {data.projects.length}
						</button>
						<button
							type="button"
							class="rounded-md px-3 py-1.5 text-ui font-medium transition-colors cursor-pointer {filter === 'live' ? 'bg-surface-2 text-text font-semibold shadow-xs' : 'text-text-2 hover:text-text'}"
							onclick={() => (filter = 'live')}
						>
							{m.projects_filter_live()} {liveCount}
						</button>
						<button
							type="button"
							class="rounded-md px-3 py-1.5 text-ui font-medium transition-colors cursor-pointer {filter === 'building' ? 'bg-surface-2 text-text font-semibold shadow-xs' : 'text-text-2 hover:text-text'}"
							onclick={() => (filter = 'building')}
						>
							{m.projects_filter_building()} {buildingCount}
						</button>
						<button
							type="button"
							class="rounded-md px-3 py-1.5 text-ui font-medium transition-colors cursor-pointer {filter === 'attention' ? 'bg-surface-2 text-text font-semibold shadow-xs' : 'text-text-2 hover:text-text'}"
							onclick={() => (filter = 'attention')}
						>
							{m.projects_filter_attention()} {attentionCount}
						</button>
					</div>

					<!-- Search & Sort -->
					<div class="flex items-center gap-3">
						<div class="relative flex items-center">
							<span class="pointer-events-none absolute left-3 text-text-3">
								<Icon icon={SearchIcon} size={15} />
							</span>
							<input
								type="search"
								bind:value={query}
								placeholder={m.projects_search()}
								class="h-9 w-60 rounded-md border border-border bg-surface pl-9 pr-3 text-ui text-text placeholder:text-text-3 transition-colors focus:border-border-strong focus:outline-none"
							/>
						</div>
						<button
							type="button"
							class="flex h-9 items-center gap-1.5 rounded-md border border-border bg-surface px-3 text-ui text-text-2 transition-colors hover:bg-surface-2 hover:text-text cursor-pointer"
							onclick={() => (sortBy = sortBy === 'updated' ? 'name' : 'updated')}
						>
							<span>{sortBy === 'updated' ? m.projects_sort_updated() : m.projects_sort_name()}</span>
							<Icon icon={ChevronDown} size={14} />
						</button>
					</div>
				</div>

				<!-- Projects Grid -->
				{#if filteredProjects.length === 0}
					<div class="rounded-xl border border-dashed border-border p-12 text-center">
						<p class="text-body text-text-2">{query ? m.list_no_match({ query }) : m.projects_empty()}</p>
					</div>
				{:else}
					<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
						{#each filteredProjects as project (project.id)}
							{@const tone = toneFor(project.name)}
							{@const statusKind = projectStatusKind(project)}
							{@const isSelected = selectedProject?.id === project.id}
							<!-- svelte-ignore a11y_click_events_have_key_events -->
							<!-- svelte-ignore a11y_no_static_element_interactions -->
							<div
								class="group flex flex-col justify-between rounded-xl border bg-surface p-4 transition-all duration-150 cursor-pointer {isSelected ? 'border-text ring-1 ring-text shadow-sm' : 'border-border hover:border-border-strong hover:bg-surface-2/40'}"
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
													'divider',
													{
														label: m.projects_visit(),
														icon: ExternalLink,
														onselect: () => {
															window.open(`https://${projectSubdomain(project)}`, '_blank');
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
											<span class="truncate font-mono text-caption text-text-3">writing src/routes/+page.svelte · 01:12</span>
										{:else if statusKind === 'live'}
											<Status kind="live" label={m.projects_live_dot()} small />
											<span class="truncate font-mono text-caption text-text-3">{projectSubdomain(project)}</span>
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
									<span class="font-mono text-caption text-text-3">
										1 chat · deploy #1 · {timeAgo(project.updated_at)}
									</span>

									<div onclick={(e) => e.stopPropagation()}>
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
												href="/projects/session/{project.session_id}"
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
				<section class="mt-12 border-t border-border pt-8">
					<div class="mb-4 flex items-center justify-between">
						<div>
							<div class="flex items-center gap-2">
								<Icon icon={GitFork} size={16} />
								<h2 class="text-body font-semibold text-text">{m.projects_remix_community()}</h2>
							</div>
							<p class="mt-0.5 text-label text-text-2">{m.projects_remix_lede()}</p>
						</div>
						<a
							href="/showcase"
							class="inline-flex items-center gap-1 text-ui text-text-2 transition-colors hover:text-text"
						>
							<span>{m.projects_browse_showcase()}</span>
							<Icon icon={ArrowUpRight} size={14} />
						</a>
					</div>

					<div class="grid grid-cols-1 gap-4 md:grid-cols-3">
						{#each COMMUNITY_TEMPLATES as tmpl (tmpl.title)}
							<div class="flex flex-col justify-between rounded-xl border border-border bg-surface p-4 shadow-sm transition-colors hover:border-border-strong">
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

			<!-- Right Details Panel (when a project is selected) -->
			{#if selectedProject}
				{@const project = selectedProject}
				{@const tone = toneFor(project.name)}
				<aside class="w-full border-l border-border bg-surface p-5 xl:w-[380px] shrink-0">
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
									<span class="h-1.5 w-1.5 rounded-full bg-emerald-500"></span>
									<span>live · deploy #14 · {timeAgo(project.updated_at)}</span>
								</div>
							</div>
						</div>
						<IconButton icon={X} label={m.action_close()} variant="ghost" onclick={() => (selectedProject = null)} />
					</div>

					<!-- Visual Mini Preview Widget -->
					<div class="mt-5 overflow-hidden rounded-xl border border-border bg-white p-5 text-neutral-900 shadow-sm dark:bg-neutral-900 dark:text-neutral-100">
						{#if project.name.includes('salah') || project.name.includes('prayer')}
							<span class="font-mono text-[10px] tracking-wider text-neutral-500 uppercase dark:text-neutral-400">NEXT PRAYER</span>
							<div class="mt-1 flex items-baseline justify-between">
								<span class="font-serif text-3xl font-bold">Dhuhr</span>
								<span class="font-mono text-lg font-semibold text-emerald-600 dark:text-emerald-400">00:27:39</span>
							</div>
							<p class="mt-2 text-xs text-neutral-500 dark:text-neutral-400">1 of 5 completed</p>
						{:else if project.name.includes('menu') || project.name.includes('warung')}
							<span class="font-mono text-[10px] tracking-wider text-neutral-500 uppercase dark:text-neutral-400">WARUNG KOPI BU SITI</span>
							<div class="mt-2 space-y-1 font-mono text-xs">
								<div class="flex justify-between">
									<span>Kopi Tubruk</span>
									<span class="font-semibold text-emerald-600 dark:text-emerald-400">5.000</span>
								</div>
								<div class="flex justify-between">
									<span>Pisang Goreng</span>
									<span class="font-semibold text-emerald-600 dark:text-emerald-400">8.000</span>
								</div>
							</div>
							<p class="mt-2 text-xs text-neutral-500 dark:text-neutral-400">Open · Closes at 22:00</p>
						{:else}
							<span class="font-mono text-[10px] tracking-wider text-neutral-500 uppercase dark:text-neutral-400">APP PREVIEW</span>
							<h4 class="mt-1 text-xl font-bold">{project.name}</h4>
							<p class="mt-1 text-xs text-neutral-500 dark:text-neutral-400">{project.description || 'Full-stack application'}</p>
							<div class="mt-3 inline-flex items-center gap-1.5 rounded bg-emerald-500/10 px-2 py-0.5 font-mono text-[11px] text-emerald-600 dark:text-emerald-400">
								<span>Vite dev server running</span>
							</div>
						{/if}
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
							href="https://{projectSubdomain(project)}"
							target="_blank"
							rel="noopener noreferrer"
							class="flex w-full items-center justify-center gap-1.5 rounded-lg border border-border py-2 text-ui text-text transition-colors hover:bg-surface-2"
						>
							<span>{m.projects_visit()}</span>
							<Icon icon={ExternalLink} size={14} />
						</a>
					</div>

					<!-- WHAT'S IN THIS PROJECT -->
					<div class="mt-6 border-t border-border pt-5">
						<SectionLabel label={m.projects_whats_in()} />
						<div class="mt-3 space-y-1">
							<div class="flex items-center justify-between text-caption">
								<KVRow icon={FileCode} label={m.projects_code_kv()} value={`23 files · main@${project.id.slice(0, 7)}`} />
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
							<KVRow icon={Globe} label={m.projects_url_kv()} value={`...${projectSubdomain(project).slice(-18)}`} />
							<KVRow icon={Database} label={m.projects_data_kv()} value={project.backend === 'relay' ? 'Nomi Relay' : 'localStorage only · no server'} />
							<KVRow icon={Rocket} label={m.projects_deploys_kv()} value="14 · last built in 41s" />
							<KVRow icon={Users} label={m.projects_members_kv()} value={`${currentOrg.name} (owner), you`} />
						</div>
					</div>

					<!-- CHATS IN THIS PROJECT -->
					<div class="mt-6 border-t border-border pt-5">
						<div class="flex items-center justify-between">
							<SectionLabel label={m.projects_chats_in()} count={2} />
						</div>

						<div class="mt-3 space-y-2">
							<a
								href="/projects/session/{project.session_id}"
								class="flex items-center justify-between rounded-lg border border-border p-2.5 text-caption transition-colors hover:bg-surface-2"
							>
								<div class="flex items-center gap-2 min-w-0">
									<Mascot agent="koda" size={16} />
									<span class="truncate text-ui text-text font-medium">Build a daily {project.name}</span>
								</div>
								<span class="shrink-0 text-text-3 font-mono text-[11px]">2d</span>
							</a>

							<a
								href="/projects/session/{project.session_id}"
								class="flex items-center justify-between rounded-lg border border-border p-2.5 text-caption transition-colors hover:bg-surface-2"
							>
								<div class="flex items-center gap-2 min-w-0">
									<Mascot agent="koda" size={16} />
									<span class="truncate text-ui text-text font-medium">Add Hijri offset setting</span>
								</div>
								<span class="shrink-0 text-text-3 font-mono text-[11px]">2d</span>
							</a>

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
