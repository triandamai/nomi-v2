<!-- The project's code, laid out like VS Code: an explorer with the folder tree (every folder
     open), a tab and breadcrumbs for the open file, the editor filling the rest, and a status
     bar. On narrow panes the explorer slides over the editor and hides once a file is picked. -->
<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import CodeEditor from './CodeEditor.svelte';
	import { buildFileTree, ancestorFolders, type TreeNode } from '$lib/code/fileTree';
	import {
		parentFolder,
		isSubpathOf,
		isZipFile,
		extractZipArchive
	} from '$lib/code/fileOperations';
	import { fileBadge } from '$lib/code/diffStats';
	import { languageName } from '$lib/code/language';
	import Dialog from '$lib/components/m3/Dialog.svelte';
	import Button from '$lib/components/m3/Button.svelte';

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
		onUploadFiles,
		onDeleteFile,
		onMoveFile
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
		onDeleteFile?: (path: string, isFolder: boolean) => Promise<void> | void;
		onMoveFile?: (sourcePath: string, targetFolder: string) => Promise<void> | void;
	} = $props();

	const tree = $derived(buildFileTree(files));
	// Folders the person closed; everything else starts open.
	let closed = $state(new Set<string>());
	let explorerOpen = $state(true);
	let cursor = $state({ line: 1, column: 1 });
	let editor: CodeEditor | undefined = $state();
	let width = $state(0);
	const narrow = $derived(width > 0 && width < 640);

	let selectedFolder = $state<string | null>(null);
	let creatingType = $state<'file' | 'folder' | null>(null);
	let creatingFolder = $state('');
	let createPath = $state('');
	let createInputEl: HTMLInputElement | undefined = $state();
	let fileUploadInput: HTMLInputElement | undefined = $state();
	let uploadTargetFolder = $state('');

	let isTreeDragOver = $state(false);
	let dropTargetFolder = $state<string | null>(null);
	let draggedItem = $state<{ path: string; isFolder: boolean } | null>(null);

	let deleteConfirmOpen = $state(false);
	let itemToDelete = $state<{ path: string; name: string; isFolder: boolean } | null>(null);

	function startCreate(type: 'file' | 'folder', folder = '') {
		creatingType = type;
		creatingFolder = folder;
		createPath = '';
		if (folder) {
			const next = new Set(closed);
			next.delete(folder);
			for (const ancestor of ancestorFolders(folder)) next.delete(ancestor);
			closed = next;
		}
		setTimeout(() => {
			createInputEl?.focus();
		}, 30);
	}

	async function submitCreate() {
		if (!creatingType) return;
		const raw = createPath.trim();
		const type = creatingType;
		const folder = creatingFolder;
		creatingType = null;
		creatingFolder = '';
		createPath = '';
		if (!raw) return;

		let clean = raw.replace(/^\/+/, '').replace(/\/+$/, '');
		if (!clean) return;

		const fullPath = folder ? `${folder}/${clean}` : clean;
		const finalPath = type === 'folder' ? `${fullPath}/.gitkeep` : fullPath;
		if (onCreateFile) {
			await onCreateFile(finalPath, '');
		}
		open(finalPath);
	}

	function handleCreateKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			submitCreate();
		} else if (e.key === 'Escape') {
			e.preventDefault();
			creatingType = null;
			creatingFolder = '';
			createPath = '';
		}
	}

	function handleCreateBlur() {
		if (createPath.trim()) {
			submitCreate();
		} else {
			creatingType = null;
			creatingFolder = '';
			createPath = '';
		}
	}

	function triggerUpload(folder = '') {
		uploadTargetFolder = folder;
		fileUploadInput?.click();
	}

	async function processUploadedFiles(uploadedFiles: FileList | File[], targetDir: string) {
		if (!uploadedFiles || uploadedFiles.length === 0) return;
		if (onUploadFiles) {
			const items: { path: string; content: string }[] = [];
			for (let i = 0; i < uploadedFiles.length; i++) {
				const file = uploadedFiles[i];
				if (isZipFile(file)) {
					const buffer = await file.arrayBuffer();
					const extracted = extractZipArchive(buffer, targetDir);
					items.push(...extracted);
				} else {
					const content = await file.text();
					const cleanName = file.name.replace(/^\/+/, '');
					const path = targetDir ? `${targetDir}/${cleanName}` : cleanName;
					items.push({ path, content });
				}
			}
			if (items.length > 0) {
				await onUploadFiles(items);
				if (targetDir) {
					const next = new Set(closed);
					next.delete(targetDir);
					for (const ancestor of ancestorFolders(targetDir)) next.delete(ancestor);
					closed = next;
				}
			}
		}
	}

	async function handleFileUpload(e: Event) {
		const target = e.target as HTMLInputElement;
		const uploadedFiles = target.files;
		if (uploadedFiles && uploadedFiles.length > 0) {
			await processUploadedFiles(uploadedFiles, uploadTargetFolder);
		}
		target.value = '';
	}

	function promptDelete(path: string, name: string, isFolder: boolean) {
		itemToDelete = { path, name, isFolder };
		deleteConfirmOpen = true;
	}

	async function handleConfirmedDelete() {
		if (!itemToDelete) return;
		const { path, isFolder } = itemToDelete;
		deleteConfirmOpen = false;
		itemToDelete = null;
		if (onDeleteFile) {
			await onDeleteFile(path, isFolder);
		}
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

	function handleDragStart(e: DragEvent, item: TreeNode) {
		draggedItem = { path: item.path, isFolder: item.kind === 'folder' };
		if (e.dataTransfer) {
			e.dataTransfer.setData('text/plain', item.path);
			e.dataTransfer.effectAllowed = 'move';
		}
	}

	function handleDragEnd() {
		draggedItem = null;
		dropTargetFolder = null;
		isTreeDragOver = false;
	}

	function handleDragOverFolder(e: DragEvent, folderPath: string) {
		e.preventDefault();
		e.stopPropagation();
		if (draggedItem) {
			if (draggedItem.path === folderPath || isSubpathOf(folderPath, draggedItem.path)) {
				if (e.dataTransfer) e.dataTransfer.dropEffect = 'none';
				return;
			}
			if (parentFolder(draggedItem.path) === folderPath) {
				if (e.dataTransfer) e.dataTransfer.dropEffect = 'none';
				return;
			}
			if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
		}
		dropTargetFolder = folderPath;
	}

	function handleDragLeaveFolder(e: DragEvent, folderPath: string) {
		e.stopPropagation();
		if (dropTargetFolder === folderPath) {
			dropTargetFolder = null;
		}
	}

	async function handleDropOnFolder(e: DragEvent, folderPath: string) {
		e.preventDefault();
		e.stopPropagation();
		dropTargetFolder = null;
		const droppedFiles = e.dataTransfer?.files;
		if (droppedFiles && droppedFiles.length > 0) {
			await processUploadedFiles(droppedFiles, folderPath);
		} else if (draggedItem) {
			const source = draggedItem.path;
			draggedItem = null;
			if (onMoveFile) {
				await onMoveFile(source, folderPath);
			}
		}
	}

	function handleTreeDragOver(e: DragEvent) {
		e.preventDefault();
		if (!draggedItem && e.dataTransfer?.types.includes('Files')) {
			isTreeDragOver = true;
		} else if (draggedItem && parentFolder(draggedItem.path) !== '') {
			isTreeDragOver = true;
			if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
		}
	}

	function handleTreeDragLeave() {
		isTreeDragOver = false;
	}

	async function handleTreeDrop(e: DragEvent) {
		e.preventDefault();
		isTreeDragOver = false;
		dropTargetFolder = null;
		const droppedFiles = e.dataTransfer?.files;
		if (droppedFiles && droppedFiles.length > 0) {
			await processUploadedFiles(droppedFiles, '');
		} else if (draggedItem) {
			const source = draggedItem.path;
			draggedItem = null;
			if (onMoveFile && parentFolder(source) !== '') {
				await onMoveFile(source, '');
			}
		}
	}

	const crumbs = $derived(activePath ? activePath.split('/') : []);
	const activeBadge = $derived(activePath ? fileBadge(activePath) : null);
</script>

{#snippet node(item: TreeNode, depth: number)}
	{#if item.kind === 'folder'}
		{@const isOpen = !closed.has(item.path)}
		{@const isSelected = selectedFolder === item.path}
		{@const isTarget = dropTargetFolder === item.path}
		<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="row row--folder"
			class:row--selected={isSelected}
			class:row--drop-target={isTarget}
			style:--depth={depth}
			draggable={true}
			ondragstart={(e) => handleDragStart(e, item)}
			ondragend={handleDragEnd}
			ondragover={(e) => handleDragOverFolder(e, item.path)}
			ondragleave={(e) => handleDragLeaveFolder(e, item.path)}
			ondrop={(e) => handleDropOnFolder(e, item.path)}
			onclick={() => { toggleFolder(item.path); selectedFolder = item.path; }}
			onkeydown={(e) => {
				if (e.key === 'Enter' || e.key === ' ') {
					e.preventDefault();
					toggleFolder(item.path);
					selectedFolder = item.path;
				}
			}}
			tabindex="0"
			role="treeitem"
			aria-expanded={isOpen}
			aria-selected={isSelected}
		>
			<svg class="row__chevron" class:row__chevron--open={isOpen} width="16" height="16" viewBox="0 0 16 16" aria-hidden="true">
				<path d="M6 4l4 4-4 4" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
			</svg>
			<span class="row__name" title={item.name}>{item.name}</span>
			<div class="row__actions">
				<button
					type="button"
					class="action-btn action-btn--subtle"
					title={m.code_new_file_in_folder({ folder: item.name })}
					aria-label={m.code_new_file()}
					onclick={(e) => { e.stopPropagation(); startCreate('file', item.path); }}
				>
					<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
						<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
						<polyline points="14 2 14 8 20 8" />
						<line x1="12" y1="18" x2="12" y2="12" />
						<line x1="9" y1="15" x2="15" y2="15" />
					</svg>
				</button>
				<button
					type="button"
					class="action-btn action-btn--subtle"
					title={m.code_new_folder_in_folder({ folder: item.name })}
					aria-label={m.code_new_folder()}
					onclick={(e) => { e.stopPropagation(); startCreate('folder', item.path); }}
				>
					<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
						<path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z" />
						<line x1="12" y1="10" x2="12" y2="16" />
						<line x1="9" y1="13" x2="15" y2="13" />
					</svg>
				</button>
				<button
					type="button"
					class="action-btn action-btn--subtle"
					title={m.code_upload_to_folder({ folder: item.name })}
					aria-label={m.code_upload_files()}
					onclick={(e) => { e.stopPropagation(); triggerUpload(item.path); }}
				>
					<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
						<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
						<polyline points="17 8 12 3 7 8" />
						<line x1="12" y1="3" x2="12" y2="15" />
					</svg>
				</button>
				<button
					type="button"
					class="action-btn action-btn--subtle action-btn--danger"
					title={m.code_delete_file()}
					aria-label={m.code_delete_file()}
					onclick={(e) => { e.stopPropagation(); promptDelete(item.path, item.name, true); }}
				>
					<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
						<polyline points="3 6 5 6 21 6" />
						<path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
					</svg>
				</button>
			</div>
		</div>
		{#if isOpen}
			<div class="children" style:--depth={depth}>
				{#if creatingType && creatingFolder === item.path}
					<div class="row row--input" style:--depth={depth + 1}>
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
				{#each item.children as child (child.path)}
					{@render node(child, depth + 1)}
				{/each}
			</div>
		{/if}
	{:else}
		{@const badge = fileBadge(item.path)}
		<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="row row--file"
			style:--depth={depth}
			style:--badge={badge.color}
			aria-current={activePath === item.path ? 'true' : undefined}
			draggable={true}
			ondragstart={(e) => handleDragStart(e, item)}
			ondragend={handleDragEnd}
			onclick={() => { open(item.path); selectedFolder = parentFolder(item.path); }}
			onkeydown={(e) => {
				if (e.key === 'Enter' || e.key === ' ') {
					e.preventDefault();
					open(item.path);
					selectedFolder = parentFolder(item.path);
				}
			}}
			tabindex="0"
			role="treeitem"
			aria-selected={activePath === item.path}
		>
			<span class="row__badge" aria-hidden="true">{badge.label}</span>
			<span class="row__name" title={item.name}>{item.name}</span>
			<div class="row__actions">
				<button
					type="button"
					class="action-btn action-btn--subtle action-btn--danger"
					title={m.code_delete_file()}
					aria-label={m.code_delete_file()}
					onclick={(e) => { e.stopPropagation(); promptDelete(item.path, item.name, false); }}
				>
					<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
						<polyline points="3 6 5 6 21 6" />
						<path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
					</svg>
				</button>
			</div>
		</div>
	{/if}
{/snippet}

<div class="bench" class:bench--narrow={narrow} bind:clientWidth={width}>
	<div class="bench__main">
		{#if explorerOpen}
			<aside class="explorer" aria-label={m.code_explorer()}>
				<div class="explorer__title">{m.code_explorer()}</div>
				<div class="explorer__header">
					<button
						type="button"
						class="explorer__project"
						title={projectName}
						onclick={() => (selectedFolder = '')}
					>
						{projectName}
					</button>
					<div class="explorer__actions">
						<button
							type="button"
							class="action-btn"
							title={selectedFolder ? m.code_new_file_in_folder({ folder: selectedFolder }) : m.code_new_file()}
							aria-label={m.code_new_file()}
							onclick={() => startCreate('file', selectedFolder ?? '')}
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
							title={selectedFolder ? m.code_new_folder_in_folder({ folder: selectedFolder }) : m.code_new_folder()}
							aria-label={m.code_new_folder()}
							onclick={() => startCreate('folder', selectedFolder ?? '')}
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
							title={selectedFolder ? m.code_upload_to_folder({ folder: selectedFolder }) : m.code_upload_files()}
							aria-label={m.code_upload_files()}
							onclick={() => triggerUpload(selectedFolder ?? '')}
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
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<div
					class="explorer__tree"
					class:explorer__tree--dragover={isTreeDragOver}
					ondragover={handleTreeDragOver}
					ondragleave={handleTreeDragLeave}
					ondrop={handleTreeDrop}
					onclick={(e) => {
						if (e.target === e.currentTarget) {
							selectedFolder = '';
						}
					}}
				>
					{#if creatingType && creatingFolder === ''}
						<div class="row row--input" style:--depth={0}>
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

<Dialog
	bind:open={deleteConfirmOpen}
	headline={m.code_delete_file()}
>
	{#snippet icon()}
		<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
			<polyline points="3 6 5 6 21 6" />
			<path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
		</svg>
	{/snippet}
	{#if itemToDelete}
		<p class="delete-confirm-text">
			{itemToDelete.isFolder
				? m.code_delete_confirm_folder({ name: itemToDelete.name })
				: m.code_delete_confirm_file({ name: itemToDelete.name })}
		</p>
	{/if}
	{#snippet actions()}
		<Button variant="text" onclick={() => (deleteConfirmOpen = false)}>
			{m.common_cancel()}
		</Button>
		<Button variant="filled" onclick={handleConfirmedDelete}>
			{m.code_delete_file()}
		</Button>
	{/snippet}
</Dialog>

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
		background: none;
		border: none;
		padding: 0;
		text-align: left;
		cursor: pointer;
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
	.action-btn--subtle {
		width: 18px;
		height: 18px;
		border-radius: 2px;
	}
	.action-btn--subtle:hover {
		background: rgba(255, 255, 255, 0.1);
	}
	.action-btn--danger:hover {
		color: #f87171;
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
		background: rgba(0, 120, 212, 0.08);
	}
	.row--input {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 2px 8px 2px calc(14px + var(--depth, 0) * 12px);
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
		padding: 0 4px 0 calc(8px + var(--depth) * 12px);
		border: 1px solid transparent;
		background: none;
		color: var(--vs-text);
		font: inherit;
		text-align: left;
		cursor: pointer;
		white-space: nowrap;
		user-select: none;
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
	.row--selected {
		background: rgba(255, 255, 255, 0.05);
	}
	.row--drop-target {
		background: rgba(0, 120, 212, 0.25) !important;
		outline: 1px solid var(--vs-focus) !important;
		outline-offset: -1px;
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
		flex: 1;
		min-width: 0;
	}
	.row__actions {
		display: none;
		align-items: center;
		gap: 2px;
		margin-left: auto;
		flex-shrink: 0;
	}
	.row:hover .row__actions,
	.row:focus-within .row__actions {
		display: flex;
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
	.delete-confirm-text {
		margin: 0;
		line-height: 1.5;
	}
	@media (prefers-reduced-motion: reduce) {
		.row__chevron {
			transition: none;
		}
	}
</style>
