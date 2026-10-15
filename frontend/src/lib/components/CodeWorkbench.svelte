<!-- The project's code, laid out like VS Code: an explorer with the folder tree (every folder
     open), a tab and breadcrumbs for the open file, the editor filling the rest, and a status
     bar. On narrow panes the explorer slides over the editor and hides once a file is picked. -->
<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import CodeEditor from './CodeEditor.svelte';
	import { buildFileTree, ancestorFolders, type TreeNode } from '$lib/code/fileTree';
	import { fileBadge } from '$lib/code/diffStats';
	import { languageName } from '$lib/code/language';

	let {
		projectName,
		files,
		activePath,
		content,
		contentKey = 0,
		dirty = $bindable(false),
		saveError = null,
		onOpen,
		onSave,
		onCreateFile,
		onUploadFiles
	}: {
		projectName: string;
		files: string[];
		activePath: string | null;
		content: string;
		/** Changes when `content` was (re)loaded, so the editor shows it. */
		contentKey?: number;
		/** Unsaved edits in the open file. */
		dirty?: boolean;
		saveError?: string | null;
		onOpen: (path: string) => void;
		onSave: (content: string) => void;
		onCreateFile?: (path: string, content: string) => Promise<void> | void;
		onUploadFiles?: (items: { path: string; content: string }[]) => Promise<void> | void;
	} = $props();

	const tree = $derived(buildFileTree(files));
	// Folders the person closed; everything else starts open.
	let closed = $state(new Set<string>());
	let explorerOpen = $state(true);
	let cursor = $state({ line: 1, column: 1 });
	let editor: CodeEditor | undefined = $state();
	let width = $state(0);
	const narrow = $derived(width > 0 && width < 640);

	let creatingType = $state<'file' | 'folder' | null>(null);
	let createPath = $state('');
	let createInputEl: HTMLInputElement | undefined = $state();
	let fileUploadInput: HTMLInputElement | undefined = $state();
	let isDragOver = $state(false);

	function startCreate(type: 'file' | 'folder') {
		creatingType = type;
		createPath = '';
		setTimeout(() => {
			createInputEl?.focus();
		}, 30);
	}

	async function submitCreate() {
		if (!creatingType) return;
		const raw = createPath.trim();
		const type = creatingType;
		creatingType = null;
		createPath = '';
		if (!raw) return;

		let clean = raw.replace(/^\/+/, '').replace(/\/+$/, '');
		if (!clean) return;

		if (type === 'folder') {
			clean = `${clean}/.gitkeep`;
		}
		if (onCreateFile) {
			await onCreateFile(clean, '');
		}
		open(clean);
	}

	function handleCreateKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			submitCreate();
		} else if (e.key === 'Escape') {
			e.preventDefault();
			creatingType = null;
			createPath = '';
		}
	}

	function handleCreateBlur() {
		if (createPath.trim()) {
			submitCreate();
		} else {
			creatingType = null;
			createPath = '';
		}
	}

	async function handleFileUpload(e: Event) {
		const target = e.target as HTMLInputElement;
		const uploadedFiles = target.files;
		if (!uploadedFiles || uploadedFiles.length === 0) return;

		if (onUploadFiles) {
			const items: { path: string; content: string }[] = [];
			for (let i = 0; i < uploadedFiles.length; i++) {
				const file = uploadedFiles[i];
				const content = await file.text();
				const path = file.name.replace(/^\/+/, '');
				items.push({ path, content });
			}
			await onUploadFiles(items);
		}
		target.value = '';
	}

	function toggleFolder(path: string) {
		const next = new Set(closed);
		if (next.has(path)) next.delete(path);
		else next.add(path);
		closed = next;
	}

	function open(path: string) {
		// Reveal it in the tree.
		const next = new Set(closed);
		for (const folder of ancestorFolders(path)) next.delete(folder);
		closed = next;
		dirty = false;
		onOpen(path);
		if (narrow) explorerOpen = false;
	}

	const crumbs = $derived(activePath ? activePath.split('/') : []);
	const activeBadge = $derived(activePath ? fileBadge(activePath) : null);
</script>

{#snippet node(item: TreeNode, depth: number)}
	{#if item.kind === 'folder'}
		{@const isOpen = !closed.has(item.path)}
		<button type="button" class="row" style:--depth={depth} aria-expanded={isOpen} onclick={() => toggleFolder(item.path)}>
			<svg class="row__chevron" class:row__chevron--open={isOpen} width="16" height="16" viewBox="0 0 16 16" aria-hidden="true">
				<path d="M6 4l4 4-4 4" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
			</svg>
			<span class="row__name">{item.name}</span>
		</button>
		{#if isOpen}
			<div class="children" style:--depth={depth}>
				{#each item.children as child (child.path)}
					{@render node(child, depth + 1)}
				{/each}
			</div>
		{/if}
	{:else}
		{@const badge = fileBadge(item.path)}
		<button
			type="button"
			class="row row--file"
			style:--depth={depth}
			style:--badge={badge.color}
			aria-current={activePath === item.path ? 'true' : undefined}
			onclick={() => open(item.path)}
		>
			<span class="row__badge" aria-hidden="true">{badge.label}</span>
			<span class="row__name">{item.name}</span>
		</button>
	{/if}
{/snippet}

<div class="bench" class:bench--narrow={narrow} bind:clientWidth={width}>
	<div class="bench__main">
		{#if explorerOpen}
			<aside class="explorer" aria-label={m.code_explorer()}>
				<div class="explorer__title">{m.code_explorer()}</div>
				<div class="explorer__header">
					<div class="explorer__project" title={projectName}>{projectName}</div>
					<div class="explorer__actions">
						<button
							type="button"
							class="action-btn"
							title={m.code_new_file()}
							aria-label={m.code_new_file()}
							onclick={() => startCreate('file')}
						>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
								<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
								<polyline points="14 2 14 8 20 8" />
								<line x1="12" y1="18" x2="12" y2="12" />
								<line x1="9" y1="15" x2="15" y2="15" />
							</svg>
						</button>
						<button
							type="button"
							class="action-btn"
							title={m.code_new_folder()}
							aria-label={m.code_new_folder()}
							onclick={() => startCreate('folder')}
						>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
								<path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z" />
								<line x1="12" y1="10" x2="12" y2="16" />
								<line x1="9" y1="13" x2="15" y2="13" />
							</svg>
						</button>
						<button
							type="button"
							class="action-btn"
							title={m.code_upload_files()}
							aria-label={m.code_upload_files()}
							onclick={() => fileUploadInput?.click()}
						>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
								<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
								<polyline points="17 8 12 3 7 8" />
								<line x1="12" y1="3" x2="12" y2="15" />
							</svg>
						</button>
						<input
							type="file"
							multiple
							bind:this={fileUploadInput}
							class="sr-only"
							onchange={handleFileUpload}
						/>
					</div>
				</div>
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div
					class="explorer__tree"
					class:explorer__tree--dragover={isDragOver}
					ondragover={(e) => { e.preventDefault(); isDragOver = true; }}
					ondragleave={() => { isDragOver = false; }}
					ondrop={async (e) => {
						e.preventDefault();
						isDragOver = false;
						const droppedFiles = e.dataTransfer?.files;
						if (!droppedFiles || droppedFiles.length === 0) return;
						if (onUploadFiles) {
							const items: { path: string; content: string }[] = [];
							for (let i = 0; i < droppedFiles.length; i++) {
								const file = droppedFiles[i];
								const content = await file.text();
								const path = file.name.replace(/^\/+/, '');
								items.push({ path, content });
							}
							await onUploadFiles(items);
						}
					}}
				>
					{#if creatingType}
						<div class="row row--input">
							{#if creatingType === 'folder'}
								<svg class="row__icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
									<path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z" />
								</svg>
							{:else}
								<svg class="row__icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
									<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
									<polyline points="14 2 14 8 20 8" />
								</svg>
							{/if}
							<input
								bind:this={createInputEl}
								type="text"
								class="create-input"
								placeholder={creatingType === 'folder' ? m.code_folder_name_placeholder() : m.code_file_name_placeholder()}
								bind:value={createPath}
								onkeydown={handleCreateKeydown}
								onblur={handleCreateBlur}
							/>
						</div>
					{/if}
					{#each tree as item (item.path)}
						{@render node(item, 0)}
					{/each}
					{#if files.length === 0 && !creatingType}
						<p class="explorer__empty">{m.project_no_files()}</p>
					{/if}
				</div>
			</aside>
		{/if}

		<section class="editor-group">
			<div class="tabs">
				<button type="button" class="tabs__toggle" aria-label={m.code_toggle_explorer()} aria-pressed={explorerOpen} onclick={() => (explorerOpen = !explorerOpen)}>
					<svg width="16" height="16" viewBox="0 0 16 16" aria-hidden="true">
						<path d="M2.5 3.5h4l1 1.5h6v7.5h-11z" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round" />
					</svg>
				</button>
				{#if activePath && activeBadge}
					<div class="tab" style:--badge={activeBadge.color}>
						<span class="row__badge" aria-hidden="true">{activeBadge.label}</span>
						<span class="tab__name">{crumbs[crumbs.length - 1]}</span>
						{#if dirty}<span class="tab__dirty" title={m.code_unsaved()}></span>{/if}
					</div>
					<button type="button" class="tabs__save" disabled={!dirty} onclick={() => editor?.saveNow()}>{m.common_save()}</button>
				{/if}
			</div>
			{#if activePath}
				<nav class="crumbs" aria-label={activePath}>
					{#each crumbs as crumb, i (i)}
						{#if i > 0}<span class="crumbs__sep" aria-hidden="true">›</span>{/if}
						<span class:crumbs__file={i === crumbs.length - 1}>{crumb}</span>
					{/each}
				</nav>
			{/if}
			{#if saveError}
				<p class="save-error" role="alert">{saveError}</p>
			{/if}
			<div class="editor-body">
				{#if activePath}
					{#key `${activePath}:${contentKey}`}
						<CodeEditor
							bind:this={editor}
							path={activePath}
							value={content}
							{onSave}
							onDirty={(d) => (dirty = d)}
							onCursor={(line, column) => (cursor = { line, column })}
						/>
					{/key}
				{:else}
					<div class="editor-empty">
						<p>{m.project_select_file()}</p>
					</div>
				{/if}
			</div>
		</section>
	</div>

	<footer class="status">
		<span>{projectName}</span>
		{#if activePath}
			<span class="status__right">
				<span>{m.code_ln_col({ line: cursor.line, column: cursor.column })}</span>
				<span>UTF-8</span>
				<span>{languageName(activePath)}</span>
			</span>
		{/if}
	</footer>
</div>

<style>
	.bench {
		--vs-bg: #1f1f1f;
		--vs-side: #181818;
		--vs-border: #2b2b2b;
		--vs-text: #cccccc;
		--vs-muted: #9d9d9d;
		--vs-hover: #2a2d2e;
		--vs-select: #37373d;
		--vs-focus: #0078d4;
		display: flex;
		flex-direction: column;
		height: 100%;
		min-height: 0;
		background: var(--vs-bg);
		color: var(--vs-text);
		font: 13px/1.4 system-ui, -apple-system, 'Segoe UI', sans-serif;
	}
	.bench__main {
		position: relative;
		display: flex;
		flex: 1;
		min-height: 0;
	}

	/* Explorer */
	.explorer {
		display: flex;
		flex-direction: column;
		width: 240px;
		flex: none;
		min-height: 0;
		background: var(--vs-side);
		border-right: 1px solid var(--vs-border);
	}
	.bench--narrow .explorer {
		position: absolute;
		inset: 0 auto 0 0;
		z-index: 2;
		width: min(280px, 85%);
		box-shadow: 4px 0 16px #0008;
	}
	.explorer__title {
		padding: 10px 16px 6px;
		font-size: 11px;
		letter-spacing: 0.04em;
		text-transform: uppercase;
		color: var(--vs-muted);
	}
	.explorer__header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 2px 8px 4px 16px;
		gap: 6px;
	}
	.explorer__project {
		font-size: 11px;
		font-weight: 700;
		text-transform: uppercase;
		color: var(--vs-text);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		flex: 1;
		min-width: 0;
	}
	.explorer__actions {
		display: flex;
		align-items: center;
		gap: 2px;
		flex-shrink: 0;
	}
	.action-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 22px;
		height: 22px;
		padding: 0;
		border: none;
		border-radius: 3px;
		background: transparent;
		color: var(--vs-muted);
		cursor: pointer;
		transition: color 0.15s, background-color 0.15s;
	}
	.action-btn:hover {
		color: var(--vs-text);
		background: var(--vs-hover);
	}
	.explorer__tree {
		flex: 1;
		min-height: 0;
		overflow: auto;
		padding-bottom: 12px;
		transition: outline-color 0.15s;
	}
	.explorer__tree--dragover {
		outline: 2px dashed var(--vs-focus);
		outline-offset: -2px;
		background: rgba(0, 120, 212, 0.05);
	}
	.row--input {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 2px 8px 2px 14px;
		height: 26px;
		background: var(--vs-hover);
	}
	.create-input {
		flex: 1;
		min-width: 0;
		height: 20px;
		padding: 0 4px;
		border: 1px solid var(--vs-focus);
		border-radius: 2px;
		background: var(--vs-bg);
		color: var(--vs-text);
		font: inherit;
		font-size: 12px;
		outline: none;
	}
	.row__icon {
		flex-shrink: 0;
		color: var(--vs-muted);
	}
	.sr-only {
		position: absolute;
		width: 1px;
		height: 1px;
		padding: 0;
		margin: -1px;
		overflow: hidden;
		clip: rect(0, 0, 0, 0);
		white-space: nowrap;
		border: 0;
	}
	.explorer__empty {
		padding: 8px 16px;
		color: var(--vs-muted);
	}
	.row {
		display: flex;
		align-items: center;
		gap: 4px;
		width: 100%;
		height: 22px;
		padding: 0 8px 0 calc(8px + var(--depth) * 12px);
		border: 1px solid transparent;
		background: none;
		color: var(--vs-text);
		font: inherit;
		text-align: left;
		cursor: pointer;
		white-space: nowrap;
	}
	.row:hover {
		background: var(--vs-hover);
	}
	.row:focus-visible {
		outline: none;
		border-color: var(--vs-focus);
	}
	.row[aria-current='true'] {
		background: var(--vs-select);
		color: #ffffff;
	}
	.row--file {
		/* Line up with folder names (past the chevron). */
		padding-left: calc(8px + var(--depth) * 12px + 16px);
	}
	.row__chevron {
		flex: none;
		color: var(--vs-text);
		transition: transform 100ms ease-out;
	}
	.row__chevron--open {
		transform: rotate(90deg);
	}
	.row__badge {
		flex: none;
		min-width: 18px;
		font: 600 10px/1 'Cascadia Code', Menlo, Consolas, monospace;
		color: var(--badge);
		text-align: center;
	}
	.row__name {
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.children {
		position: relative;
	}
	/* Indent guide, as VS Code draws under an open folder. */
	.children::before {
		content: '';
		position: absolute;
		top: 0;
		bottom: 0;
		left: calc(16px + var(--depth) * 12px);
		border-left: 1px solid #585858;
		opacity: 0.4;
		pointer-events: none;
	}

	/* Editor group */
	.editor-group {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-width: 0;
		min-height: 0;
	}
	.tabs {
		display: flex;
		align-items: stretch;
		height: 35px;
		flex: none;
		background: var(--vs-side);
		border-bottom: 1px solid var(--vs-border);
	}
	.tabs__toggle {
		display: grid;
		place-items: center;
		width: 35px;
		flex: none;
		border: none;
		border-right: 1px solid var(--vs-border);
		background: none;
		color: var(--vs-muted);
		cursor: pointer;
	}
	.tabs__toggle[aria-pressed='true'] {
		color: var(--vs-text);
	}
	.tabs__toggle:hover {
		background: var(--vs-hover);
	}
	.tab {
		display: flex;
		align-items: center;
		gap: 6px;
		min-width: 0;
		padding: 0 12px 0 10px;
		background: var(--vs-bg);
		border-top: 1px solid var(--vs-focus);
		border-right: 1px solid var(--vs-border);
		margin-bottom: -1px;
		color: #ffffff;
	}
	.tab__name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.tab__dirty {
		width: 8px;
		height: 8px;
		flex: none;
		border-radius: 50%;
		background: var(--vs-text);
	}
	.tabs__save {
		margin: 5px 8px 5px auto;
		padding: 0 12px;
		border: none;
		border-radius: 2px;
		background: var(--vs-focus);
		color: #ffffff;
		font: inherit;
		cursor: pointer;
	}
	.tabs__save:disabled {
		background: #313131;
		color: var(--vs-muted);
		cursor: default;
	}
	.tabs__save:focus-visible,
	.tabs__toggle:focus-visible {
		outline: 1px solid var(--vs-focus);
		outline-offset: -1px;
	}
	.crumbs {
		display: flex;
		align-items: center;
		gap: 4px;
		height: 22px;
		flex: none;
		padding: 0 14px;
		color: var(--vs-muted);
		white-space: nowrap;
		overflow: hidden;
	}
	.crumbs__sep {
		opacity: 0.7;
	}
	.crumbs__file {
		color: var(--vs-text);
	}
	.save-error {
		margin: 0;
		padding: 6px 14px;
		background: #5a1d1d;
		color: #f8b4b4;
	}
	.editor-body {
		flex: 1;
		min-height: 0;
	}
	.editor-empty {
		display: grid;
		place-items: center;
		height: 100%;
		color: var(--vs-muted);
		padding: 14px;
		text-align: center;
	}

	/* Status bar */
	.status {
		display: flex;
		align-items: center;
		gap: 12px;
		height: 22px;
		flex: none;
		padding: 0 10px;
		background: var(--vs-side);
		border-top: 1px solid var(--vs-border);
		color: var(--vs-muted);
		font-size: 12px;
		white-space: nowrap;
		overflow: hidden;
	}
	.status__right {
		display: flex;
		gap: 14px;
		margin-left: auto;
	}
	@media (prefers-reduced-motion: reduce) {
		.row__chevron {
			transition: none;
		}
	}
</style>
