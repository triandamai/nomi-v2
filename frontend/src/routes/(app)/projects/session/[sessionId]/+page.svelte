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
	import IconLink from '$lib/components/icons/IconLink.svelte';
	import IconHistory from '$lib/components/icons/IconHistory.svelte';
	import IconOpenInNew from '$lib/components/icons/IconOpenInNew.svelte';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import { ProjectPresence } from '$lib/projectPresence.svelte';
	import Snackbar from '$lib/components/m3/Snackbar.svelte';
	import ModelManager from '$lib/components/models/ModelManager.svelte';
	import ModelSelector from '$lib/components/models/ModelSelector.svelte';
	import { applyChanges, choiceFor, type ModelChange, type ModelChoice } from '$lib/modelPicker';
	import { postAction } from '$lib/providerActions';
	import IssuesPanel from '$lib/components/project/IssuesPanel.svelte';
	import DeployPanel from '$lib/components/project/DeployPanel.svelte';
	import MonitorPanel from '$lib/components/project/MonitorPanel.svelte';
	import DatabasePanel from '$lib/components/project/DatabasePanel.svelte';
	import RelayPanel from '$lib/components/project/RelayPanel.svelte';
	import MembersPanel from '$lib/components/project/MembersPanel.svelte';
	import SettingsPanel from '$lib/components/project/SettingsPanel.svelte';
	import DomainPanel from '$lib/components/project/DomainPanel.svelte';
	import { ProjectDeploy } from '$lib/projectDeploy.svelte';
	import { invalidateAll, replaceState } from '$app/navigation';
	import { isBackendEvent } from '$lib/presenceView';
	import ResizeHandle from '$lib/components/ds/build/ResizeHandle.svelte';
	import WorkspaceSheet from '$lib/components/ds/build/WorkspaceSheet.svelte';
	import PublishButton from '$lib/components/ds/build/PublishButton.svelte';
	import Terminal from '$lib/components/ds/build/Terminal.svelte';
	import { chatWidth as splitWidth, SPLIT } from '$lib/components/ds/build/split';
	import { terminalLines } from '$lib/terminalLines';
	import Presence from '$lib/components/ds/build/Presence.svelte';
	import PresenceMenu from '$lib/components/ds/build/PresenceMenu.svelte';
	import FollowBar from '$lib/components/ds/build/FollowBar.svelte';
	import PeerCursor from '$lib/components/ds/build/PeerCursor.svelte';
	import PublishRequest from '$lib/components/ds/build/PublishRequest.svelte';
	import RequestQueue from '$lib/components/ds/build/RequestQueue.svelte';
	import { splitAttachments } from '$lib/attachments';
	import { initials, peerFor, whereLabel } from '$lib/peers';
	import { projectApi } from '$lib/projectApi';
	import { buildAgentPlansFetchUrl } from '$lib/buildAgentPlansFetchUrl';
	import type { ChecklistItem } from '$lib/planChecklist';
	import { Activity, Bug, Database, History, Link2, Settings, Users } from '@lucide/svelte';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let view = $state<'preview' | 'code' | 'terminal'>('preview');

	// The composer's model picker, as on a chat (what this chat replies with, and what Koda builds
	// the project with). Both follow the page's data, and are moved ahead of the server while a
	// change is on its way (a pick that fails is put back).
	let modelGroups = $derived(data.modelOptions.groups);
	let sessionModel = $derived(data.sessionModel);
	let manageOpen = $state(false);
	let modelErrorOpen = $state(false);

	async function pickModel(key: string) {
		const previous = sessionModel;
		sessionModel = choiceFor(modelGroups, key) ?? previous;
		const body = new FormData();
		body.set('key', key);
		const outcome = await postAction<{ sessionModel: ModelChoice }>('setSessionModel', body);
		if (outcome.ok) {
			sessionModel = outcome.data.sessionModel;
		} else {
			sessionModel = previous;
			modelErrorOpen = true;
		}
	}

	const modelsSaved = (changes: ModelChange[]) => (modelGroups = applyChanges(modelGroups, changes));

	// The project's tools open in full-screen sheets over the project, so its chat, code and
	// running preview stay as they are underneath (closing one never restarts the install).
	type Sheet = 'deploy' | 'domain' | 'monitor' | 'database' | 'members' | 'issues' | 'settings';
	const SHEETS: Sheet[] = ['deploy', 'domain', 'monitor', 'database', 'members', 'issues', 'settings'];
	let sheet = $state<Sheet | null>(null);
	let sheetOpen = $state(false);
	// Sheets open below the toolbar, which stays in view: where its bottom edge is.
	let topbar: HTMLElement | undefined = $state();
	let sheetTop = $state(0);
	$effect(() => {
		if (!topbar || !sheetOpen) return;
		const bar = topbar;
		const measure = () => (sheetTop = Math.max(0, Math.round(bar.getBoundingClientRect().bottom)));
		measure();
		const observer = new ResizeObserver(measure);
		observer.observe(bar);
		window.addEventListener('resize', measure);
		return () => {
			observer.disconnect();
			window.removeEventListener('resize', measure);
		};
	});
	onMount(() => {
		const asked = page.url.searchParams.get('sheet') as Sheet | null;
		if (asked && SHEETS.includes(asked)) openSheet(asked);
	});
	// The open sheet is kept in the address (`?sheet=`) so a link opens it. While the page is
	// still starting (a link that opens a sheet), SvelteKit's router can't change the address yet,
	// and throwing then would leave the router broken; the address is already right then anyway.
	function keepSheetInAddress(which: Sheet | null) {
		const url = new URL(page.url);
		if (which) url.searchParams.set('sheet', which);
		else url.searchParams.delete('sheet');
		try {
			replaceState(url, {});
		} catch {
			// Before the router started: nothing to change.
		}
	}
	function openSheet(which: Sheet) {
		sheet = which;
		sheetOpen = true;
		keepSheetInAddress(which);
	}
	$effect(() => {
		if (sheetOpen) return;
		if (!page.url.searchParams.has('sheet')) return;
		keepSheetInAddress(null);
	});

	let deploy = $state<ProjectDeploy | null>(null);
	// Keyed on the id, not `data`: every reload brings a new `data`, and a fresh (empty) deploy
	// would blank an open Deploy, Domain or Settings sheet until it loads again.
	$effect(() => {
		const id = projectId;
		if (!id) return;
		const d = new ProjectDeploy(id);
		deploy = d;
		void d.refresh();
		d.listen();
		// Rena puts a project on Relay after the page has loaded: show its Relay sheet then.
		const stopBackend = d.onEvent((event) => {
			if (isBackendEvent(event)) {
				void invalidateAll();
				void runtime?.refreshRelay();
			}
		});
		const p = new ProjectPresence(id);
		presence = p;
		p.start(d);
		return () => {
			stopBackend();
			p.stop();
			d.stop();
		};
	});

	let presence = $state<ProjectPresence | null>(null);
	/** Shown at most this many faces in the toolbar; the rest are counted in the label. */
	const TOOLBAR_FACES = 3;
	const typingLabel = $derived.by(() => {
		const names = presence?.typing ?? [];
		if (names.length === 0) return null;
		if (names.length === 1) return m.pj_typing_one({ name: names[0] });
		if (names.length === 2) return m.pj_typing_two({ first: names[0], second: names[1] });
		return m.pj_typing_many({ first: names[0], second: names[1], count: names.length - 2 });
	});
	let membersTab = $state<'people' | 'activity'>('people');
	function openActivity() {
		membersTab = 'activity';
		openSheet('members');
	}

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

	const PROJECT_STATUS: Record<'planning' | 'building' | 'ready', () => string> = {
		planning: m.projects_planning,
		building: m.projects_building,
		ready: m.projects_ready,
	};

	/** The data sheet is Relay's console for a project on Relay, the Postgres one otherwise. */
	function dataTitle(): string {
		return data.project?.backend === 'relay' ? m.pj_tab_relay() : m.pj_tab_database();
	}

	const TOOLS: { id: Sheet; label: () => string; icon: typeof IconRocket }[] = [
		{ id: 'deploy', label: m.pj_tab_deploy, icon: IconHistory },
		{ id: 'domain', label: m.pj_tab_domain, icon: IconLink },
		{ id: 'monitor', label: m.pj_tab_monitor, icon: IconPulse },
		{ id: 'database', label: () => dataTitle(), icon: IconDatabase },
		{ id: 'members', label: m.pj_tab_members, icon: IconGroup },
		{ id: 'settings', label: m.pj_tab_settings, icon: IconSettings },
	];
	const SHEET_ICON = { deploy: History, domain: Link2, monitor: Activity, database: Database, members: Users, issues: Bug, settings: Settings } as const;
	const SHEET_TITLE: Record<Sheet, () => string> = {
		deploy: m.pj_tab_deploy,
		domain: m.pj_tab_domain,
		monitor: m.pj_tab_monitor,
		database: () => dataTitle(),
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

	const isSettingUp = $derived(
		Boolean(runtime && ['booting', 'loading', 'waiting', 'installing', 'checking', 'starting'].includes(runtime.phase))
	);

	// The page needs its cross-origin isolation headers, which only a full load brings. Arriving
	// by an in-app link, load it once more; leaving, load the next page in full so the isolation
	// doesn't follow the person around the app.
	onMount(() => {
		const handleBeforeUnload = (e: BeforeUnloadEvent) => {
			if (isSettingUp || editorDirty) {
				e.preventDefault();
				e.returnValue = '';
				return '';
			}
		};
		window.addEventListener('beforeunload', handleBeforeUnload);

		if (!runsLive || window.crossOriginIsolated) {
			return () => window.removeEventListener('beforeunload', handleBeforeUnload);
		}
		const key = `nomi-isolated:${location.pathname}`;
		try {
			if (sessionStorage.getItem(key)) {
				return () => window.removeEventListener('beforeunload', handleBeforeUnload);
			}
			sessionStorage.setItem(key, '1');
		} catch {
			return () => window.removeEventListener('beforeunload', handleBeforeUnload);
		}
		location.reload();
		return () => window.removeEventListener('beforeunload', handleBeforeUnload);
	});
	beforeNavigate(({ to, cancel, willUnload }) => {
		if (to && to.url.pathname !== location.pathname) {
			if (isSettingUp || editorDirty) {
				const confirmed = window.confirm(m.project_leave_confirm());
				if (!confirmed) {
					cancel();
					return;
				}
			}
		}
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

	// The chat and the workspace (preview, code, terminal) side by side (Build → Workspace): the
	// chat's width is dragged on the resize handle between them (360–720, the preview never under
	// 480), and the workspace can be hidden to give the chat the whole screen. On a phone the
	// workspace opens over the chat instead. Both remembered here.
	const WIDTH_KEY = 'nomi-project-chat-width';
	const PANE_KEY = 'nomi-project-pane';
	let layout = $state<HTMLDivElement | null>(null);
	let chatWidth = $state<number>(SPLIT.reset);
	let paneOpen = $state(true);
	let narrow = $state(false);
	let layoutWidth = $state(1200);

	function remember(key: string, value: string) {
		try {
			localStorage.setItem(key, value);
		} catch {
			// Private windows and blocked storage: fine, it just isn't remembered.
		}
	}
	function setPane(open: boolean) {
		paneOpen = open;
		if (!narrow) remember(PANE_KEY, open ? 'open' : 'closed');
	}
	function resize(width: number) {
		chatWidth = width;
		remember(WIDTH_KEY, String(width));
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
		const measure = () => (layoutWidth = layout?.clientWidth ?? window.innerWidth);
		measure();
		window.addEventListener('resize', measure);
		try {
			const width = Number(localStorage.getItem(WIDTH_KEY));
			if (width > 0) chatWidth = splitWidth(width, layoutWidth);
		} catch {
			// Keep the default width.
		}
		return () => {
			query.removeEventListener('change', apply);
			window.removeEventListener('resize', measure);
		};
	});

	// The newest plan's steps, for the empty preview while the first build runs.
	let planSteps = $state<{ label: string; state: 'done' | 'running' | 'waiting' }[]>([]);
	$effect(() => {
		const plan = [...data.messages].reverse().flatMap((message) => message.content_blocks ?? []).find((block) => block.kind === 'plan');
		if (!plan || plan.kind !== 'plan' || filePaths.length > 0) {
			planSteps = [];
			return;
		}
		void fetch(buildAgentPlansFetchUrl(page.url.pathname, plan.agent_session_id))
			.then((response) => (response.ok ? response.json() : null))
			.then((found: { plans: { version: number; checklist: ChecklistItem[] }[] } | null) => {
				const latest = found?.plans.reduce<{ version: number; checklist: ChecklistItem[] } | null>((best, p) => (!best || p.version > best.version ? p : best), null);
				planSteps = (latest?.checklist ?? []).map((item) => ({
					label: item.text,
					state: item.status === 'done' ? 'done' : item.status === 'in_progress' ? 'running' : 'waiting',
				}));
			})
			.catch(() => (planSteps = []));
	});

	// Building together (Build together → Presence, Follow, Peer cursor): who's here and where,
	// following someone to see what they see, and their pointer over the same view.
	const WHERE = $derived({ preview: m.project_preview(), code: m.project_code(), terminal: m.project_terminal() });
	const peers = $derived((presence?.people ?? []).map((person) => ({ ...person, initials: initials(person.name), peer: peerFor(person.name) })));
	const myName = $derived(page.data.profile?.display_name || page.data.profile?.username || (page.data.userEmail ?? '').split('@')[0] || m.msg_you());
	let presenceMenuOpen = $state(false);
	let following = $state<string | null>(null);
	const followed = $derived(peers.find((person) => person.user_id === following) ?? null);

	// Others see where the person is.
	$effect(() => {
		presence?.moveTo({ view, path: view === 'code' ? activePath : null });
	});
	// Following someone: go where they go. Moving on your own stops following.
	$effect(() => {
		const person = followed;
		if (!person || !data.project) return;
		const where = person.view ?? 'preview';
		if (where !== view && (where !== 'terminal' || runtime)) view = where;
		if (where === 'code' && person.path && person.path !== activePath && !editorDirty) void openFile(data.project.id, person.path);
	});
	$effect(() => {
		if (following && !followed) following = null;
	});

	// Pointers: the person's goes out a few times a second; others' fade after a few seconds still.
	let pointerClock = $state(Date.now());
	$effect(() => {
		if (!presence || Object.keys(presence.pointers).length === 0) return;
		const timer = setInterval(() => (pointerClock = Date.now()), 1000);
		return () => clearInterval(timer);
	});
	const pointers = $derived(presence ? presence.pointersIn(view, pointerClock).map((pointer) => ({ ...pointer, peer: peers.find((p) => p.user_id === pointer.user_id)?.peer ?? 1 })) : []);
	function trackPointer(event: PointerEvent) {
		const box = (event.currentTarget as HTMLElement).getBoundingClientRect();
		if (box.width === 0 || box.height === 0) return;
		presence?.point(view, (event.clientX - box.left) / box.width, (event.clientY - box.top) / box.height);
	}

	// Publish requests (Build together → Publish request): an admin sees who asked to publish and
	// publishes or reviews it right there. They change live over the project's stream.
	type PendingRequest = { id: string; requested_by_name: string; note: string | null; environment: string; created_at: string; status: string };
	let requests = $state<PendingRequest[]>([]);
	async function loadRequests() {
		if (!projectId || !deploy?.isAdmin) return;
		try {
			const list = await projectApi<{ requests: PendingRequest[] }>(projectId, 'deploy/requests');
			requests = list.requests.filter((request) => request.status === 'pending');
		} catch {
			// The deploy sheet still lists them.
		}
	}
	$effect(() => {
		if (!deploy?.isAdmin) return;
		void loadRequests();
		return deploy.onEvent((raw) => {
			try {
				if (JSON.parse(raw).event === 'publish_request.changed') void loadRequests();
			} catch {
				// Not ours.
			}
		});
	});
	async function answerRequest(id: string, action: 'approve' | 'decline') {
		if (!projectId) return;
		try {
			await projectApi(projectId, `deploy/requests/${id}`, { method: 'POST', body: { action } });
		} finally {
			await loadRequests();
			if (action === 'approve') await deploy?.refresh();
		}
	}

	// The asks waiting for Koda (Build together → Asks in line): one change at a time, who asked
	// what, and the next one moved up. Looked at again every few seconds while the chat is open.
	type QueuedAsk = { id: string; text: string; who: string; mine: boolean; state: 'running' | 'next' | 'queued' };
	let asks = $state<QueuedAsk[]>([]);
	async function loadQueue() {
		const response = await fetch(`${page.url.pathname}/queue`).catch(() => null);
		if (response?.ok) asks = ((await response.json()) as { asks: QueuedAsk[] }).asks;
	}
	$effect(() => {
		if (!data.project) return;
		void loadQueue();
		const timer = setInterval(() => {
			if (document.visibilityState === 'visible') void loadQueue();
		}, 4000);
		return () => clearInterval(timer);
	});
	async function moveFirst(index: number) {
		const ask = asks[index];
		if (!ask) return;
		const response = await fetch(`${page.url.pathname}/queue/${ask.id}/first`, { method: 'POST' }).catch(() => null);
		if (response?.ok) asks = ((await response.json()) as { asks: QueuedAsk[] }).asks;
	}
	const viewer = $derived(deploy?.overview?.role === 'viewer');

	// The publish button (Build → Workspace / Publish): what will go live, publishing with its
	// time, or a failed publish to try again. Someone who isn't an admin asks the admins instead.
	let clock = $state(Date.now());
	$effect(() => {
		if (!deploy?.running) return;
		const timer = setInterval(() => (clock = Date.now()), 1000);
		return () => clearInterval(timer);
	});
	const publishState = $derived.by((): 'idle' | 'ready' | 'publishing' | 'failed' => {
		if (!deploy) return 'idle';
		if (deploy.running || deploy.preparing || deploy.busy) return 'publishing';
		if (deploy.latest?.status === 'failed') return 'failed';
		const changes = deploy.overview?.unpublished_changes;
		return changes === 0 ? 'idle' : 'ready';
	});
	const publishElapsed = $derived.by(() => {
		const started = deploy?.latest?.status === 'running' ? Date.parse(deploy.latest.created_at) : NaN;
		if (Number.isNaN(started)) return '';
		const seconds = Math.max(0, Math.floor((clock - started) / 1000));
		return `${String(Math.floor(seconds / 60)).padStart(2, '0')}:${String(seconds % 60).padStart(2, '0')}`;
	});
	// Asking the admins to publish (the publish requests they approve or decline).
	let askedNote = $state<string | null>(null);
	let askedOpen = $state(false);
	async function askToPublish() {
		if (!projectId) return;
		try {
			await projectApi(projectId, 'deploy/requests', { method: 'POST', body: {} });
			askedNote = m.ws_publish_asked();
		} catch (e) {
			askedNote = (e as Error).message || m.ws_publish_ask_failed();
		}
		askedOpen = true;
	}

	async function publishNow() {
		if (!deploy) return;
		if (deploy.isAdmin) {
			await deploy.publish();
			openSheet('deploy');
		} else {
			await askToPublish();
		}
	}
</script>

{#snippet modelControl()}
	{#if modelGroups.length > 0 || sessionModel}
		<ModelSelector
			groups={modelGroups}
			selected={sessionModel}
			onselect={pickModel}
			onmanage={() => (manageOpen = true)}
			hint={m.msel_project_hint()}
		/>
	{/if}
{/snippet}

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
	<header class="topbar" bind:this={topbar}>
		<a class="topbar__brand" href="/projects" aria-label={m.pj_all_projects()} title={m.pj_all_projects()}><AgentShape size={28} face /></a>
		<div class="topbar__titles">
			<h1 class="topbar__name">{project.name}<span class="topbar__status" data-status={project.status}>{PROJECT_STATUS[project.status]()}</span></h1>
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
		{#if presence && peers.length > 0}
			<div class="here-wrap">
				<button type="button" class="here" aria-haspopup="dialog" aria-expanded={presenceMenuOpen} aria-label={m.pj_here_now_people({ names: peers.map((p) => p.name).join(', ') })} onclick={() => (presenceMenuOpen = !presenceMenuOpen)}>
					<Presence people={peers.slice(0, TOOLBAR_FACES).map((person, index) => ({ name: person.name, initials: person.initials, peer: person.peer, online: index === Math.min(peers.length, TOOLBAR_FACES) - 1 }))} />
					{#if peers.length > TOOLBAR_FACES}<span class="here__more">+{peers.length - TOOLBAR_FACES}</span>{/if}
				</button>
				{#if presenceMenuOpen}
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div class="here-menu" onkeydown={(event) => event.key === 'Escape' && (presenceMenuOpen = false)}>
						<PresenceMenu
							people={[
								...peers.map((person) => ({ name: person.name, initials: person.initials, peer: person.peer, where: whereLabel(person.view, person.path, WHERE) })),
								{ name: m.presence_you({ name: myName }), initials: initials(myName), where: whereLabel(view, activePath, WHERE), you: true },
							]}
							onfollow={(index) => {
								following = peers[index]?.user_id ?? null;
								presenceMenuOpen = false;
							}}
						/>
						<button type="button" class="here-menu__activity" onclick={() => { presenceMenuOpen = false; openActivity(); }}>{m.pj_act_tab_activity()}</button>
					</div>
				{/if}
			</div>
		{/if}
		{#if deploy?.overview?.available}
			<PublishButton
				state={publishState}
				count={deploy.overview.unpublished_changes ?? 0}
				elapsed={publishElapsed}
				onpublish={publishNow}
				onretry={publishNow}
			/>
		{/if}
	</header>
{/if}
<div
	bind:this={layout}
	class="workspace"
	class:workspace--narrow={narrow}
	class:workspace--pane-hidden={!paneOpen}
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
			placeholder={m.composer_placeholder_project()}
			project={{ status: data.project?.status ?? 'planning' }}
			{modelControl}
			extraControls={paneToggle}
			compactHeader={Boolean(data.project)}
			{typingLabel}
			readOnlyNote={viewer ? m.pj_viewer_note() : null}
			typingPeople={(presence?.typing ?? []).map((name) => ({ name, initials: initials(name), peer: peerFor(name) }))}
			ontyping={() => presence?.ping()}
		>
			{#snippet aboveComposer()}
				{#each requests as request (request.id)}
					<PublishRequest
						who={{ name: request.requested_by_name, initials: initials(request.requested_by_name), peer: peerFor(request.requested_by_name) }}
						meta={[request.environment, request.note].filter(Boolean).join(' · ')}
						onpublish={() => answerRequest(request.id, 'approve')}
						onreview={() => openSheet('deploy')}
					/>
				{/each}
				{#if asks.length > 1}
					<RequestQueue
						asks={asks.map((ask) => ({
							text: splitAttachments(ask.text).text || ask.text,
							who: { name: ask.who, initials: initials(ask.who), peer: ask.mine ? undefined : peerFor(ask.who) },
							state: ask.state,
							meta: ask.mine ? m.queue_yours() : undefined,
						}))}
						onfirst={viewer ? undefined : moveFirst}
					/>
				{/if}
			{/snippet}
		</ChatThread>
	</section>

	{#if paneOpen && !narrow}
		<ResizeHandle width={chatWidth} available={layoutWidth} onresize={resize} />
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
					<button type="button" role="tab" class="pane-tab" aria-selected={view === 'preview'} onclick={() => { view = 'preview'; following = null; }}>{m.project_preview()}</button>
					<button type="button" role="tab" class="pane-tab" aria-selected={view === 'code'} onclick={() => { view = 'code'; following = null; }}>{m.project_code()}</button>
					{#if runtime}
						<button type="button" role="tab" class="pane-tab" aria-selected={view === 'terminal'} onclick={() => { view = 'terminal'; following = null; }}>{m.project_terminal()}</button>
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

			{#if followed}
				<FollowBar name={followed.name} peer={followed.peer} where={whereLabel(followed.view, followed.path, WHERE)} onstop={() => (following = null)} />
			{/if}
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div class="pane-body min-h-0 flex-1" onpointermove={trackPointer}>
				{#each pointers as pointer (pointer.user_id)}
					<span class="pane-pointer" style="left: {pointer.x * 100}%; top: {pointer.y * 100}%"><PeerCursor name={pointer.name} peer={pointer.peer} /></span>
				{/each}
				{#if view === 'preview'}
					<ProjectPreview
						projectId={project.id}
						{runtime}
						stack={project.stack}
						files={filePaths}
						steps={planSteps}
						liveUrl={deploy?.overview?.url ?? null}
						onShowCode={() => (view = 'code')}
					/>
				{:else if view === 'terminal' && runtime}
					<!-- What ran in the preview, in order (Build → Terminal); a failed check offers the fix. -->
					<div class="terminal-pane">
						<Terminal
							run={m.term_run_preview({ name: project.name })}
							lines={runtime.log ? terminalLines(runtime.log) : [{ kind: 'out', text: m.project_terminal_empty() }]}
							failure={runtime.checkFailed && !runtime.handedToKoda ? { title: m.project_check_failed(), onfix: () => sendToChat(m.term_fix_message()) } : undefined}
						/>
					</div>
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

<!-- Outside the thread: its composer is a form, and these hold forms of their own. -->
<ModelManager bind:open={manageOpen} groups={modelGroups} onsaved={modelsSaved} />
<Snackbar bind:open={modelErrorOpen} message={m.msel_pick_failed()} />
<Snackbar bind:open={askedOpen} message={askedNote ?? ''} />

{#if data.project && sheet}
	{@const project = data.project}
	<BottomSheet bind:open={sheetOpen} full top={sheetTop} label={SHEET_TITLE[sheet]()}>
		<WorkspaceSheet icon={SHEET_ICON[sheet]} title={SHEET_TITLE[sheet]()} meta={project.name} onclose={() => (sheetOpen = false)}>
			{#if sheet === 'issues'}
				<IssuesPanel projectId={project.id} onCountChange={(n) => (openIssues = n)} onAskKoda={sendToChat} />
			{:else if sheet === 'deploy' && deploy}
				<DeployPanel {deploy} onaskfix={sendToChat} />
			{:else if sheet === 'domain' && deploy}
				<DomainPanel {deploy} />
			{:else if sheet === 'monitor'}
				<MonitorPanel projectId={project.id} {published} {deploy} />
			{:else if sheet === 'database'}
				{#if project.backend === 'relay'}
					<RelayPanel projectId={project.id} {published} />
				{:else}
					<DatabasePanel projectId={project.id} {published} />
				{/if}
			{:else if sheet === 'members'}
				<MembersPanel projectId={project.id} orgId={project.org_id} projectName={project.name} bind:tab={membersTab} {presence} {deploy} />
			{:else if sheet === 'settings' && deploy}
				<SettingsPanel {deploy} projectName={project.name} />
			{/if}
		</WorkspaceSheet>
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
	.topbar__status {
		margin-left: 8px;
		padding: 1px 7px;
		border-radius: 4px;
		background: var(--md-sys-color-surface-container-highest);
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-ref-typeface-plain);
		font-size: 0.6875rem;
		font-weight: 600;
		vertical-align: 2px;
	}
	.topbar__status[data-status='ready'] {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.topbar {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 4px 10px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
		background: var(--nomi-color-ground);
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
		background: var(--nomi-color-ground);
		animation: pane-in var(--nomi-motion-spatial-default, 350ms cubic-bezier(0.27, 1.06, 0.18, 1));
	}
	.workspace__pane[hidden] {
		display: none;
	}
	/* While dragging, the preview's frame mustn't swallow the pointer. */
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

	.pane-bar {
		display: flex;
		align-items: center;
		gap: 12px;
		min-height: 44px;
		padding: 2px 8px 2px 10px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	/* The project's name is already in the bar above. */
	.pane-bar__title {
		display: none;
	}
	.pane-bar__tabs {
		margin-right: auto;
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

	.here {
		display: inline-flex;
		align-items: center;
		min-height: 44px;
		padding: 0 6px;
		border: 0;
		border-radius: 999px;
		background: transparent;
		cursor: pointer;
	}
	.here:hover {
		background: var(--md-sys-color-surface-container-low);
	}
	.here:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.here__more {
		margin-left: 6px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
		font-weight: 600;
	}
	.here-wrap {
		position: relative;
	}
	.here-menu {
		position: absolute;
		top: calc(100% + 6px);
		right: 0;
		z-index: 30;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.here-menu__activity {
		align-self: flex-end;
		padding: 4px 8px;
		border: none;
		background: none;
		color: var(--md-sys-color-on-surface-variant);
		font: inherit;
		font-size: 0.8125rem;
		cursor: pointer;
	}
	.pane-body {
		position: relative;
	}
	.pane-pointer {
		position: absolute;
		z-index: 20;
		pointer-events: none;
		transition:
			left 120ms linear,
			top 120ms linear;
	}
	.terminal-pane {
		display: flex;
		height: 100%;
		padding: 12px;
	}
	.terminal-pane :global(section) {
		height: 100%;
	}
</style>
