<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { deserialize } from '$app/forms';
	import { page } from '$app/state';
	import { beforeNavigate } from '$app/navigation';
	import { onMount } from 'svelte';
	import ChatThread from '$lib/components/ChatThread.svelte';
	import CodeWorkbench from '$lib/components/CodeWorkbench.svelte';
	import { relocateFilePath, relocateFolderFilePath } from '$lib/code/fileOperations';
	import ProjectPreview from '$lib/components/ProjectPreview.svelte';
	import { ProjectRuntime } from '$lib/projectRuntime.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import IconClose from '$lib/components/icons/IconClose.svelte';
	import IconSidePanel from '$lib/components/icons/IconSidePanel.svelte';
	import IconRocket from '$lib/components/icons/IconRocket.svelte';
	import IconBug from '$lib/components/icons/IconBug.svelte';
	import IconPulse from '$lib/components/icons/IconPulse.svelte';
	import IconDatabase from '$lib/components/icons/IconDatabase.svelte';
	import IconGroup from '$lib/components/icons/IconGroup.svelte';
	import IconSettings from '$lib/components/icons/IconSettings.svelte';
	import IconHistory from '$lib/components/icons/IconHistory.svelte';
	import IconOpenInNew from '$lib/components/icons/IconOpenInNew.svelte';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import SheetFrame from '$lib/components/project/SheetFrame.svelte';
	import IssuesPanel from '$lib/components/project/IssuesPanel.svelte';
	import DeployPanel from '$lib/components/project/DeployPanel.svelte';
	import MonitorPanel from '$lib/components/project/MonitorPanel.svelte';
	import DatabasePanel from '$lib/components/project/DatabasePanel.svelte';
	import MembersPanel from '$lib/components/project/MembersPanel.svelte';
	import SettingsPanel from '$lib/components/project/SettingsPanel.svelte';
	import { ProjectDeploy } from '$lib/projectDeploy.svelte';
	import { invalidateAll, replaceState } from '$app/navigation';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let view = $state<'preview' | 'code' | 'terminal'>('preview');

	// The project's tools open in full-screen sheets over the project, so its chat, code and
	// running preview stay as they are underneath (closing one never restarts the install).
	type Sheet = 'deploy' | 'monitor' | 'database' | 'members' | 'issues' | 'settings';
	const SHEETS: Sheet[] = ['deploy', 'monitor', 'database', 'members', 'issues', 'settings'];
	let sheet = $state<Sheet | null>(null);
	let sheetOpen = $state(false);
	onMount(() => {
		const asked = page.url.searchParams.get('sheet') as Sheet | null;
		if (asked && SHEETS.includes(asked)) openSheet(asked);
	});
	function openSheet(which: Sheet) {
		sheet = which;
		sheetOpen = true;
		const url = new URL(page.url);
		url.searchParams.set('sheet', which);
		replaceState(url, {});
	}
	$effect(() => {
		if (sheetOpen) return;
		if (!page.url.searchParams.has('sheet')) return;
		const url = new URL(page.url);
		url.searchParams.delete('sheet');
		replaceState(url, {});
	});

	let deploy = $state<ProjectDeploy | null>(null);
	$effect(() => {
		const id = data.project?.id;
		if (!id) return;
		const d = new ProjectDeploy(id);
		deploy = d;
		void d.refresh();
		return () => d.stop();
	});

	let openIssues = $state(0);
	$effect(() => {
		openIssues = data.project?.open_issues ?? 0;
	});

	// Sends a message to the project's chat (an issue handed to Koda), then shows it there.
	async function sendToChat(text: string) {
		const body = new FormData();
		body.set('text', text);
		const response = await fetch('?/sendMessage', { method: 'POST', body, headers: { 'x-sveltekit-action': 'true' } });
		const result = deserialize(await response.text());
		if (result.type !== 'success') throw new Error('send failed');
		sheetOpen = false;
		await invalidateAll();
	}

	const TOOLS: { id: Sheet; label: () => string; icon: typeof IconRocket }[] = [
		{ id: 'deploy', label: m.pj_tab_deploy, icon: IconHistory },
		{ id: 'monitor', label: m.pj_tab_monitor, icon: IconPulse },
		{ id: 'database', label: m.pj_tab_database, icon: IconDatabase },
		{ id: 'members', label: m.pj_tab_members, icon: IconGroup },
		{ id: 'settings', label: m.pj_tab_settings, icon: IconSettings },
	];
	const SHEET_TITLE: Record<Sheet, () => string> = {
		deploy: m.pj_tab_deploy,
		monitor: m.pj_tab_monitor,
		database: m.pj_tab_database,
		members: m.pj_tab_members,
		issues: m.issues_title,
		settings: m.pj_tab_settings,
	};
	const published = $derived(Boolean(deploy?.overview?.app && deploy.overview.publishes.some((p) => p.status === 'succeeded')));
	let runtime = $state<ProjectRuntime | null>(null);

	// Projects with a package.json run in the browser (WebContainer); static ones don't.
	const projectId = $derived(data.project?.id ?? null);
	const runsLive = $derived(Boolean(data.project && data.project.stack !== 'static'));

	$effect(() => {
		if (!projectId || !runsLive) return;
		const live = new ProjectRuntime(projectId);
		runtime = live;
		void live.start();
		return () => {
			runtime = null;
			void live.stop();
		};
	});

	// The page needs its cross-origin isolation headers, which only a full load brings. Arriving
	// by an in-app link, load it once more; leaving, load the next page in full so the isolation
	// doesn't follow the person around the app.
	onMount(() => {
		if (!runsLive || window.crossOriginIsolated) return;
		const key = `nomi-isolated:${location.pathname}`;
		try {
			if (sessionStorage.getItem(key)) return;
			sessionStorage.setItem(key, '1');
		} catch {
			return;
		}
		location.reload();
	});
	beforeNavigate(({ to, cancel, willUnload }) => {
		if (willUnload || !to || !window.crossOriginIsolated) return;
		if (to.url.pathname === location.pathname) return;
		cancel();
		location.href = to.url.href;
	});
	// The file list follows Koda's work without a reload: from the in-browser runtime when it's
	// running, otherwise by checking the project's file list every few seconds while it's on screen.
	let polledFiles = $state<{ path: string; updated_at: string }[] | null>(null);
	$effect(() => {
		if (!projectId || runtime) return;
		const id = projectId;
		let stopped = false;
		const refresh = async () => {
			if (document.hidden) return;
			const response = await fetch(`/projects/${id}/api/manifest`).catch(() => null);
			if (!stopped && response?.ok) polledFiles = ((await response.json()) as { files: { path: string; updated_at: string }[] }).files;
		};
		const timer = setInterval(refresh, 5000);
		return () => {
			stopped = true;
			clearInterval(timer);
			polledFiles = null;
		};
	});
	const liveFiles = $derived<{ path: string; updated_at?: string }[]>(
		runtime && runtime.files.length > 0 ? runtime.files : (polledFiles ?? data.project?.files ?? []),
	);
	const filePaths = $derived(liveFiles.map((f) => f.path));

	let activePath = $state<string | null>(null);
	let activeContent = $state('');
	// When Koda rewrites the open file, show the new version (unless there are unsaved edits).
	let activeVersion = $state<string | undefined>(undefined);
	let editorDirty = $state(false);
	$effect(() => {
		const path = activePath;
		const id = projectId;
		const current = liveFiles.find((f) => f.path === path)?.updated_at;
		if (!path || !id || !current) return;
		if (activeVersion === undefined) {
			activeVersion = current;
			return;
		}
		if (current !== activeVersion && !editorDirty) {
			activeVersion = current;
			void openFile(id, path);
		}
	});
	let saveError = $state<string | null>(null);
	/** Bumped on every load so the editor shows content reloaded for the same file. */
	let contentKey = $state(0);

	async function openFile(projectId: string, path: string) {
		const body = new FormData();
		body.set('projectId', projectId);
		body.set('path', path);
		const response = await fetch('?/loadFile', { method: 'POST', body });
		const result = deserialize(await response.text());
		activeContent = result.type === 'success' && typeof result.data?.content === 'string' ? result.data.content : '';
		if (activePath !== path) activeVersion = undefined;
		activePath = path;
		contentKey += 1;
	}

	async function saveFilePath(projectId: string, path: string, content: string) {
		saveError = null;
		const body = new FormData();
		body.set('projectId', projectId);
		body.set('path', path);
		body.set('content', content);
		const response = await fetch('?/saveFile', { method: 'POST', body });
		const result = deserialize(await response.text());
		if (result.type !== 'success') {
			const err = (result.type === 'failure' && (result.data?.error as string)) || m.project_save_failed();
			saveError = err;
			throw new Error(err);
		}
		await runtime?.writeFile(path, content);
		const now = new Date().toISOString();
		const current: { path: string; updated_at: string }[] = polledFiles
			? [...polledFiles]
			: (data.project?.files?.map((f) => ({ path: f.path, updated_at: now })) ?? []);
		const existingIndex = current.findIndex((f) => f.path === path);
		if (existingIndex >= 0) {
			current[existingIndex] = { ...current[existingIndex], updated_at: now };
		} else {
			current.push({ path, updated_at: now });
		}
		polledFiles = current;
		if (runtime && !runtime.files.some((f) => f.path === path)) {
			runtime.files = [...runtime.files, { path, size_bytes: content.length, updated_at: now }];
		}
	}

	async function saveFile(projectId: string, content: string) {
		if (!activePath) return;
		try {
			await saveFilePath(projectId, activePath, content);
		} catch {
			// saveError already set
		}
	}

	async function createFile(projectId: string, path: string, content = '') {
		await saveFilePath(projectId, path, content);
		await openFile(projectId, path);
	}

	async function uploadFiles(projectId: string, items: { path: string; content: string }[]) {
		let lastPath: string | null = null;
		for (const item of items) {
			await saveFilePath(projectId, item.path, item.content);
			lastPath = item.path;
		}
		if (lastPath) {
			await openFile(projectId, lastPath);
		}
	}

	async function deleteFilePath(projectId: string, path: string) {
		const body = new FormData();
		body.set('projectId', projectId);
		body.set('path', path);
		const response = await fetch('?/deleteFile', { method: 'POST', body });
		const result = deserialize(await response.text());
		if (result.type !== 'success') {
			const err = (result.type === 'failure' && (result.data?.error as string)) || m.code_delete_file_failed();
			saveError = err;
			throw new Error(err);
		}
		await runtime?.removeFile(path);
		const current = polledFiles
			? [...polledFiles]
			: (data.project?.files?.map((f) => ({ path: f.path, updated_at: '' })) ?? []);
		polledFiles = current.filter((f) => f.path !== path);
		if (data.project?.files) {
			data.project.files = data.project.files.filter((f) => f.path !== path);
		}
		if (runtime) {
			runtime.files = runtime.files.filter((f) => f.path !== path);
		}
	}

	async function deletePath(projectId: string, path: string) {
		saveError = null;
		const matching = filePaths.filter((p) => p === path || p.startsWith(`${path}/`));
		if (matching.length === 0) {
			try {
				await deleteFilePath(projectId, path);
			} catch {
				// ignore
			}
		} else {
			for (const p of matching) {
				await deleteFilePath(projectId, p);
			}
		}
		if (activePath === path || (activePath && activePath.startsWith(`${path}/`))) {
			activePath = null;
			activeContent = '';
			activeVersion = undefined;
		}
	}

	async function movePath(projectId: string, sourcePath: string, targetFolder: string) {
		saveError = null;
		const isFolder = !filePaths.includes(sourcePath);
		if (!isFolder) {
			const newPath = relocateFilePath(sourcePath, targetFolder);
			if (newPath === sourcePath) return;

			let contentToMove = '';
			if (sourcePath === activePath) {
				contentToMove = activeContent;
			} else {
				const body = new FormData();
				body.set('projectId', projectId);
				body.set('path', sourcePath);
				const res = await fetch('?/loadFile', { method: 'POST', body });
				const result = deserialize(await res.text());
				if (result.type === 'success' && typeof result.data?.content === 'string') {
					contentToMove = result.data.content;
				}
			}

			await saveFilePath(projectId, newPath, contentToMove);
			await deleteFilePath(projectId, sourcePath);

			if (activePath === sourcePath) {
				activePath = newPath;
			}
		} else {
			const matching = filePaths.filter((p) => p.startsWith(`${sourcePath}/`));
			let newActive: string | null = null;
			for (const p of matching) {
				const newFilePath = relocateFolderFilePath(sourcePath, targetFolder, p);
				let contentToMove = '';
				if (p === activePath) {
					contentToMove = activeContent;
					newActive = newFilePath;
				} else {
					const body = new FormData();
					body.set('projectId', projectId);
					body.set('path', p);
					const res = await fetch('?/loadFile', { method: 'POST', body });
					const result = deserialize(await res.text());
					if (result.type === 'success' && typeof result.data?.content === 'string') {
						contentToMove = result.data.content;
					}
				}
				await saveFilePath(projectId, newFilePath, contentToMove);
				await deleteFilePath(projectId, p);
			}
			if (newActive) {
				activePath = newActive;
			}
		}
	}

	// The chat and the workspace (preview, code, terminal) side by side: the chat's width is
	// dragged on the handle between them, and the workspace can be hidden to give the chat the
	// whole screen. On a phone the workspace opens over the chat instead. Both remembered here.
	const WIDTH_KEY = 'nomi-project-chat-width';
	const PANE_KEY = 'nomi-project-pane';
	const DEFAULT_WIDTH = 440;
	const MIN_CHAT = 320;
	const MIN_PANE = 360;
	let layout = $state<HTMLDivElement | null>(null);
	let chatWidth = $state(DEFAULT_WIDTH);
	let paneOpen = $state(true);
	let narrow = $state(false);
	let dragging = $state(false);

	function remember(key: string, value: string) {
		try {
			localStorage.setItem(key, value);
		} catch {
			// Private windows and blocked storage: fine, it just isn't remembered.
		}
	}
	function clampWidth(width: number): number {
		const total = layout?.clientWidth ?? 1200;
		return Math.round(Math.min(Math.max(width, MIN_CHAT), Math.max(MIN_CHAT, total - MIN_PANE)));
	}
	function setPane(open: boolean) {
		paneOpen = open;
		if (!narrow) remember(PANE_KEY, open ? 'open' : 'closed');
	}

	onMount(() => {
		const query = window.matchMedia('(max-width: 899px)');
		const apply = () => {
			narrow = query.matches;
			let stored: string | null = null;
			try {
				stored = localStorage.getItem(PANE_KEY);
			} catch {
				stored = null;
			}
			// A phone starts on the chat; a bigger screen shows both unless hidden before.
			paneOpen = narrow ? false : stored !== 'closed';
		};
		apply();
		query.addEventListener('change', apply);
		try {
			const width = Number(localStorage.getItem(WIDTH_KEY));
			if (width > 0) chatWidth = clampWidth(width);
		} catch {
			// Keep the default width.
		}
		return () => query.removeEventListener('change', apply);
	});

	function startDrag(event: PointerEvent) {
		if (event.button !== 0) return;
		const handle = event.currentTarget as HTMLElement;
		handle.setPointerCapture(event.pointerId);
		dragging = true;
		const left = layout?.getBoundingClientRect().left ?? 0;
		const move = (e: PointerEvent) => (chatWidth = clampWidth(e.clientX - left));
		const stop = () => {
			dragging = false;
			handle.removeEventListener('pointermove', move);
			handle.removeEventListener('pointerup', stop);
			handle.removeEventListener('pointercancel', stop);
			remember(WIDTH_KEY, String(chatWidth));
		};
		handle.addEventListener('pointermove', move);
		handle.addEventListener('pointerup', stop);
		handle.addEventListener('pointercancel', stop);
	}
	function dragKeys(event: KeyboardEvent) {
		const step = event.shiftKey ? 96 : 24;
		const next = { ArrowLeft: chatWidth - step, ArrowRight: chatWidth + step, Home: MIN_CHAT, End: Number.MAX_SAFE_INTEGER }[event.key];
		if (next === undefined) return;
		event.preventDefault();
		chatWidth = clampWidth(next);
		remember(WIDTH_KEY, String(chatWidth));
	}
	function resetWidth() {
		chatWidth = clampWidth(DEFAULT_WIDTH);
		remember(WIDTH_KEY, String(chatWidth));
	}
</script>

{#snippet paneToggle()}
	{#if data.project}
		<IconButton
			onclick={() => setPane(!paneOpen)}
			aria-label={paneOpen ? m.project_pane_hide() : m.project_pane_show()}
			aria-pressed={paneOpen}
			title={paneOpen ? m.project_pane_hide() : m.project_pane_show()}
		>
			<IconSidePanel open={paneOpen} />
		</IconButton>
	{/if}
{/snippet}

<div class="project-app">
{#if data.project}
	{@const project = data.project}
	<header class="topbar">
		<a class="topbar__brand" href="/projects" aria-label={m.pj_all_projects()} title={m.pj_all_projects()}><AgentShape size={28} face /></a>
		<div class="topbar__titles">
			<h1 class="topbar__name">{project.name}</h1>
			<p class="topbar__meta">
				{project.org_name}
				{#if deploy?.overview?.url}
					· <a class="topbar__url" href={deploy.overview.url} target="_blank" rel="noopener">{deploy.overview.url.replace(/^https?:\/\//, '')}<IconOpenInNew size={12} /></a>
				{:else if deploy?.running}
					· {m.pj_publishing()}
				{/if}
			</p>
		</div>
		<nav class="topbar__tools" aria-label={m.pj_tools()}>
			<button type="button" class="tool" onclick={() => openSheet('issues')}>
				<IconBug size={18} /><span class="tool__label">{m.workspace_issues()}</span>
				{#if openIssues > 0}<span class="tool__count">{openIssues}</span>{/if}
			</button>
			{#each TOOLS as tool (tool.id)}
				<button type="button" class="tool" onclick={() => openSheet(tool.id)} title={tool.label()}>
					<tool.icon size={18} /><span class="tool__label">{tool.label()}</span>
				</button>
			{/each}
		</nav>
		{#if deploy?.overview?.available && deploy.isAdmin}
			<Button variant="filled" size="s" disabled={deploy.busy || deploy.running} onclick={async () => { await deploy?.publish(); openSheet('deploy'); }}>
				<IconRocket size={16} /><span class="publish-label">{deploy.running ? m.pj_publishing() : m.pj_publish()}</span>
			</Button>
		{/if}
	</header>
{/if}
<div
	bind:this={layout}
	class="workspace"
	class:workspace--narrow={narrow}
	class:workspace--pane-hidden={!paneOpen}
	class:workspace--dragging={dragging}
	style="--chat-width: {chatWidth}px"
>
	<section class="workspace__chat" aria-label={m.project_chat_label()}>
		<ChatThread
			sessionId={page.params.sessionId as string}
			title={data.project?.name ?? m.project_new()}
			messages={data.messages}
			hasOlder={data.hasOlder}
			agentActivity={data.agentActivity}
			agentStatus={data.agentStatus}
			sendError={form?.error ?? null}
			thinkingLevel={data.thinkingLevel}
			project={{ status: data.project?.status ?? 'planning' }}
			extraControls={paneToggle}
		/>
	</section>

	{#if paneOpen && !narrow}
		<!-- A focusable separator is a widget (WAI-ARIA window splitter), which the checker doesn't know. -->
		<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
		<div
			class="workspace__handle"
			role="separator"
			aria-orientation="vertical"
			aria-label={m.project_resize()}
			aria-valuemin={MIN_CHAT}
			aria-valuenow={chatWidth}
			tabindex="0"
			onpointerdown={startDrag}
			onkeydown={dragKeys}
			ondblclick={resetWidth}
		>
			<span class="workspace__grip"></span>
		</div>
	{/if}

	<section class="workspace__pane" aria-label={m.project_workspace_label()} hidden={!paneOpen}>
		{#if !data.project}
			<div class="flex h-full items-center justify-center p-8 text-center">
				<p class="md-body-large" style="color: var(--md-sys-color-on-surface-variant)">
					{m.project_empty()}
				</p>
			</div>
		{:else}
			{@const project = data.project}
			<div class="pane-bar">
				<h2 class="pane-bar__title">{project.name}</h2>
				<div class="pane-bar__tabs" role="tablist" aria-label={m.project_workspace_label()}>
					<button type="button" role="tab" class="pane-tab" aria-selected={view === 'preview'} onclick={() => (view = 'preview')}>{m.project_preview()}</button>
					<button type="button" role="tab" class="pane-tab" aria-selected={view === 'code'} onclick={() => (view = 'code')}>{m.project_code()}</button>
					{#if runtime}
						<button type="button" role="tab" class="pane-tab" aria-selected={view === 'terminal'} onclick={() => (view = 'terminal')}>{m.project_terminal()}</button>
					{/if}
				</div>
				<IconButton onclick={() => setPane(false)} aria-label={m.project_pane_hide()} title={m.project_pane_hide()}>
					<IconClose size={18} />
				</IconButton>
			</div>
			{#if runtime?.checkFailed}
				<p class="md-body-small px-4 py-2" style="background: var(--md-sys-color-error-container); color: var(--md-sys-color-on-error-container)">
					{runtime.handedToKoda ? m.project_check_failed_koda() : m.project_check_failed()}
				</p>
			{/if}

			<div class="min-h-0 flex-1">
				{#if view === 'preview'}
					<ProjectPreview
						projectId={project.id}
						{runtime}
						stack={project.stack}
						files={filePaths}
						onShowCode={() => (view = 'code')}
					/>
				{:else if view === 'terminal' && runtime}
					<pre class="terminal">{runtime.log || m.project_terminal_empty()}</pre>
				{:else}
					<CodeWorkbench
						projectName={project.name}
						files={filePaths}
						{activePath}
						content={activeContent}
						{contentKey}
						bind:dirty={editorDirty}
						{saveError}
						onOpen={(path) => openFile(project.id, path)}
						onSave={(content) => saveFile(project.id, content)}
						onCreateFile={(path, content) => createFile(project.id, path, content)}
						onUploadFiles={(items) => uploadFiles(project.id, items)}
						onDeleteFile={(path) => deletePath(project.id, path)}
						onMoveFile={(source, target) => movePath(project.id, source, target)}
					/>
				{/if}
			</div>
		{/if}
	</section>
</div>
</div>

{#if data.project && sheet}
	{@const project = data.project}
	<BottomSheet bind:open={sheetOpen} full label={SHEET_TITLE[sheet]()}>
		<SheetFrame title={SHEET_TITLE[sheet]()} subtitle={project.name} onclose={() => (sheetOpen = false)}>
			{#if sheet === 'issues'}
				<IssuesPanel projectId={project.id} onCountChange={(n) => (openIssues = n)} onAskKoda={sendToChat} />
			{:else if sheet === 'deploy' && deploy}
				<DeployPanel {deploy} />
			{:else if sheet === 'monitor'}
				<MonitorPanel projectId={project.id} {published} />
			{:else if sheet === 'database'}
				<DatabasePanel projectId={project.id} {published} />
			{:else if sheet === 'members'}
				<MembersPanel projectId={project.id} orgId={project.org_id} />
			{:else if sheet === 'settings' && deploy}
				<SettingsPanel {deploy} projectName={project.name} />
			{/if}
		</SheetFrame>
	</BottomSheet>
{/if}

<style>
	.project-app {
		display: flex;
		flex-direction: column;
		height: 100%;
		min-height: 0;
	}
	.project-app > .workspace {
		flex: 1;
		min-height: 0;
		height: auto;
	}
	.topbar {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 8px 12px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
		background: var(--md-sys-color-surface);
	}
	.topbar__brand {
		display: inline-flex;
		flex: none;
		align-items: center;
		justify-content: center;
		width: 44px;
		height: 44px;
		border-radius: var(--md-sys-shape-corner-full);
	}
	.topbar__brand:hover {
		background: var(--md-sys-color-surface-container-high);
	}
	.topbar__brand:focus-visible,
	.tool:focus-visible,
	.topbar__url:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.topbar__titles {
		flex: 1;
		min-width: 0;
	}
	.topbar__name {
		margin: 0;
		overflow: hidden;
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-ref-typeface-brand);
		font-size: 1rem;
		font-weight: 700;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.topbar__meta {
		margin: 0;
		overflow: hidden;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.75rem;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.topbar__url {
		display: inline-flex;
		align-items: center;
		gap: 3px;
		color: var(--md-sys-color-primary);
		text-decoration: none;
	}
	.topbar__url:hover {
		text-decoration: underline;
	}
	.topbar__tools {
		display: flex;
		flex: none;
		gap: 2px;
		overflow-x: auto;
	}
	.tool {
		display: inline-flex;
		flex: none;
		align-items: center;
		gap: 6px;
		min-height: 40px;
		padding: 0 10px;
		border: none;
		border-radius: var(--md-sys-shape-corner-full);
		background: none;
		color: var(--md-sys-color-on-surface-variant);
		font: inherit;
		font-size: 0.8125rem;
		font-weight: 600;
		cursor: pointer;
	}
	.tool:hover {
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface);
	}
	.tool__count {
		min-width: 18px;
		padding: 0 5px;
		border-radius: 4px;
		background: var(--md-sys-color-surface-container-highest);
		font-variant-numeric: tabular-nums;
		text-align: center;
	}
	@media (max-width: 1100px) {
		.tool__label {
			display: none;
		}
		.tool {
			justify-content: center;
			min-width: 44px;
			padding: 0;
		}
	}
	@media (max-width: 640px) {
		.topbar {
			flex-wrap: wrap;
			gap: 4px 8px;
			padding: 6px 8px;
		}
		.topbar__tools {
			order: 3;
			width: 100%;
			justify-content: space-between;
		}
		.tool {
			min-height: 44px;
		}
		.publish-label {
			display: none;
		}
	}

	.workspace {
		display: flex;
		height: 100%;
		min-height: 0;
		position: relative;
	}
	.workspace__chat {
		flex: 0 0 var(--chat-width);
		min-width: 0;
		height: 100%;
		transition: flex-basis var(--nomi-motion-spatial-default, 350ms cubic-bezier(0.27, 1.06, 0.18, 1));
	}
	.workspace--dragging .workspace__chat {
		transition: none;
	}
	.workspace--pane-hidden .workspace__chat,
	.workspace--narrow .workspace__chat {
		flex: 1 1 auto;
	}
	.workspace__pane {
		display: flex;
		flex: 1 1 0;
		flex-direction: column;
		min-width: 0;
		height: 100%;
		background: var(--md-sys-color-surface);
		animation: pane-in var(--nomi-motion-spatial-default, 350ms cubic-bezier(0.27, 1.06, 0.18, 1));
	}
	.workspace__pane[hidden] {
		display: none;
	}
	/* While dragging, the preview's frame mustn't swallow the pointer. */
	.workspace--dragging .workspace__pane {
		pointer-events: none;
		user-select: none;
	}
	.workspace--dragging {
		cursor: col-resize;
		user-select: none;
	}
	@keyframes pane-in {
		from {
			opacity: 0;
			transform: translateX(24px);
		}
	}
	/* Phone: the workspace opens over the chat. */
	.workspace--narrow .workspace__pane {
		position: absolute;
		inset: 0;
		z-index: 5;
		animation-name: pane-up;
	}
	.workspace--narrow .pane-bar__title {
		display: none;
	}
	.workspace--narrow .pane-bar__tabs {
		margin-right: auto;
	}
	@keyframes pane-up {
		from {
			opacity: 0.6;
			transform: translateY(32px);
		}
	}

	.workspace__handle {
		position: relative;
		flex: 0 0 12px;
		margin: 0 -6px;
		z-index: 2;
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: col-resize;
		touch-action: none;
	}
	.workspace__handle::before {
		content: '';
		position: absolute;
		inset: 0 5px;
		background: var(--md-sys-color-outline-variant);
		transition: background-color var(--nomi-motion-effects-fast, 150ms);
	}
	.workspace__grip {
		position: relative;
		width: 4px;
		height: 48px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-outline);
		transition:
			height var(--nomi-motion-spatial-fast, 200ms),
			background-color var(--nomi-motion-effects-fast, 150ms);
	}
	.workspace__handle:hover .workspace__grip,
	.workspace__handle:focus-visible .workspace__grip,
	.workspace--dragging .workspace__grip {
		height: 72px;
		background: var(--md-sys-color-primary);
	}
	.workspace__handle:hover::before,
	.workspace--dragging .workspace__handle::before {
		background: color-mix(in srgb, var(--md-sys-color-primary) 40%, var(--md-sys-color-outline-variant));
	}
	.workspace__handle:focus-visible {
		outline: none;
	}

	.pane-bar {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 8px 8px 8px 14px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.pane-bar__title {
		flex: 1;
		min-width: 0;
		margin: 0;
		overflow: hidden;
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-ref-typeface-brand);
		font-size: 0.9375rem;
		font-weight: 700;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.pane-bar__tabs {
		display: inline-flex;
		gap: 2px;
		padding: 3px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-high);
	}
	.pane-tab {
		height: 32px;
		padding: 0 12px;
		border: none;
		border-radius: var(--md-sys-shape-corner-full);
		background: transparent;
		color: var(--md-sys-color-on-surface-variant);
		font: inherit;
		font-size: 0.8125rem;
		font-weight: 600;
		cursor: pointer;
		transition:
			background-color var(--nomi-motion-effects-fast, 150ms),
			color var(--nomi-motion-effects-fast, 150ms);
	}
	.pane-tab:hover {
		color: var(--md-sys-color-on-surface);
	}
	.pane-tab[aria-selected='true'] {
		background: var(--md-sys-color-surface);
		color: var(--md-sys-color-on-surface);
		box-shadow: 0 1px 2px color-mix(in srgb, var(--md-sys-color-shadow, #000) 18%, transparent);
	}
	.pane-tab:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 1px;
	}
	@media (prefers-reduced-motion: reduce) {
		.workspace__chat,
		.workspace__pane {
			transition: none;
			animation: none;
		}
	}

	.terminal {
		height: 100%;
		margin: 0;
		padding: 12px 14px;
		overflow: auto;
		background: #0f1a14;
		color: #d7e6dc;
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
		line-height: 1.55;
		white-space: pre-wrap;
		word-break: break-word;
	}
</style>
