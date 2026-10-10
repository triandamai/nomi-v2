<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { deserialize } from '$app/forms';
	import { page } from '$app/state';
	import { beforeNavigate } from '$app/navigation';
	import { onMount } from 'svelte';
	import ChatThread from '$lib/components/ChatThread.svelte';
	import CodeEditor from '$lib/components/CodeEditor.svelte';
	import ProjectPreview from '$lib/components/ProjectPreview.svelte';
	import { ProjectRuntime } from '$lib/projectRuntime.svelte';
	import List from '$lib/components/m3/List.svelte';
	import ListItem from '$lib/components/m3/ListItem.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import IconClose from '$lib/components/icons/IconClose.svelte';
	import IconSidePanel from '$lib/components/icons/IconSidePanel.svelte';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let view = $state<'preview' | 'code' | 'terminal'>('preview');
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
	let activePath = $state<string | null>(null);
	let activeContent = $state('');
	let saveError = $state<string | null>(null);

	async function openFile(projectId: string, path: string) {
		const body = new FormData();
		body.set('projectId', projectId);
		body.set('path', path);
		const response = await fetch('?/loadFile', { method: 'POST', body });
		const result = deserialize(await response.text());
		activeContent = result.type === 'success' && typeof result.data?.content === 'string' ? result.data.content : '';
		activePath = path;
	}

	async function saveFile(projectId: string, content: string) {
		if (!activePath) return;
		saveError = null;
		const body = new FormData();
		body.set('projectId', projectId);
		body.set('path', activePath);
		body.set('content', content);
		const response = await fetch('?/saveFile', { method: 'POST', body });
		const result = deserialize(await response.text());
		if (result.type !== 'success') {
			saveError = (result.type === 'failure' && (result.data?.error as string)) || m.project_save_failed();
			return;
		}
		await runtime?.writeFile(activePath, content);
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
						files={project.files.map((f) => f.path)}
						onShowCode={() => (view = 'code')}
					/>
				{:else if view === 'terminal' && runtime}
					<pre class="terminal">{runtime.log || m.project_terminal_empty()}</pre>
				{:else}
					<div class="flex h-full">
						<aside class="w-56 shrink-0 overflow-y-auto p-3" style="border-right: 1px solid var(--md-sys-color-outline-variant)">
							<List>
								{#each project.files as file (file.path)}
									<button
										type="button"
										class="w-full text-left"
										style="background: none; border: none; padding: 0; cursor: pointer"
										onclick={() => openFile(project.id, file.path)}
									>
										<ListItem headline={file.path} selected={activePath === file.path} />
									</button>
								{/each}
							</List>
							{#if project.files.length === 0}
								<p class="md-body-small" style="color: var(--md-sys-color-on-surface-variant)">{m.project_no_files()}</p>
							{/if}
						</aside>
						<div class="min-w-0 flex-1">
							{#if saveError}
								<p class="md-body-medium p-2" style="color: var(--md-sys-color-error)">{saveError}</p>
							{/if}
							{#if activePath}
								{#key activePath}
									<CodeEditor path={activePath} value={activeContent} onSave={(content) => saveFile(project.id, content)} />
								{/key}
							{:else}
								<div class="flex h-full items-center justify-center">
									<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">{m.project_select_file()}</p>
								</div>
							{/if}
						</div>
					</div>
				{/if}
			</div>
		{/if}
	</section>
</div>

<style>
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
		padding: 8px 8px 8px 16px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.pane-bar__title {
		flex: 1;
		min-width: 0;
		margin: 0;
		overflow: hidden;
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-ref-typeface-brand);
		font-size: 1rem;
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
		padding: 0 14px;
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
		padding: 12px 16px;
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
