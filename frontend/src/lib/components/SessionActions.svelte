<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { deserialize } from '$app/forms';
	import { invalidateAll } from '$app/navigation';
	import BottomSheet from './m3/BottomSheet.svelte';
	import Button from './m3/Button.svelte';
	import IconButton from './m3/IconButton.svelte';
	import Menu from './m3/Menu.svelte';
	import MenuItem from './m3/MenuItem.svelte';
	import TextField from './m3/TextField.svelte';
	import IconMore from './icons/IconMore.svelte';
	import Search from './ds/inputs/Search.svelte';
	import type { SessionSummary } from '$lib/types';

	// A chat's ⋯ menu (rename, pin, move, delete) with its sheets; posts to the page's
	// renameSession, pinSession, moveSession and deleteSession actions. `projects` are the ones the
	// chat can move into; a project's own chat stays with its project.
	let { session, projects = [] }: { session: SessionSummary; projects?: { id: string; name: string }[] } = $props();

	let menuOpen = $state(false);
	let confirmOpen = $state(false);
	let deleting = $state(false);
	let renameOpen = $state(false);
	let renameValue = $state('');
	let renaming = $state(false);
	let renameError = $state<string | null>(null);

	function openDeleteConfirm(event: MouseEvent) {
		event.preventDefault();
		event.stopPropagation();
		menuOpen = false;
		confirmOpen = true;
	}

	function openRename(event: MouseEvent) {
		event.preventDefault();
		event.stopPropagation();
		menuOpen = false;
		renameValue = session.title ?? '';
		renameError = null;
		renameOpen = true;
	}

	async function saveRename(event: SubmitEvent) {
		event.preventDefault();
		if (!renameValue.trim()) {
			renameError = m.chat_rename_empty();
			return;
		}
		renaming = true;
		const body = new FormData();
		body.set('sessionId', session.id);
		body.set('title', renameValue);
		const result = deserialize(await (await fetch('?/renameSession', { method: 'POST', body })).text());
		renaming = false;
		if (result.type === 'success') {
			renameOpen = false;
			await invalidateAll();
		} else {
			renameError = m.chat_rename_failed();
		}
	}

	let moveOpen = $state(false);
	let moveQuery = $state('');
	let moveTo = $state('');
	let moving = $state(false);
	let updateError = $state<string | null>(null);
	const shownProjects = $derived(projects.filter((project) => project.name.toLowerCase().includes(moveQuery.trim().toLowerCase())));

	/** Posts one change to the page's `action`; reloads the list when it took. */
	async function post(action: 'pinSession' | 'moveSession', fields: Record<string, string>): Promise<boolean> {
		const body = new FormData();
		body.set('sessionId', session.id);
		for (const [key, value] of Object.entries(fields)) body.set(key, value);
		const result = deserialize(await (await fetch(`?/${action}`, { method: 'POST', body })).text());
		if (result.type !== 'success') {
			updateError = m.chat_update_failed();
			return false;
		}
		updateError = null;
		await invalidateAll();
		return true;
	}

	async function togglePin(event: MouseEvent) {
		event.preventDefault();
		event.stopPropagation();
		menuOpen = false;
		await post('pinSession', { pinned: String(!session.pinned) });
	}

	function openMove(event: MouseEvent) {
		event.preventDefault();
		event.stopPropagation();
		menuOpen = false;
		moveQuery = '';
		moveTo = session.in_project_id ?? '';
		updateError = null;
		moveOpen = true;
	}

	async function saveMove(event: SubmitEvent) {
		event.preventDefault();
		moving = true;
		const moved = await post('moveSession', { projectId: moveTo });
		moving = false;
		if (moved) moveOpen = false;
	}

	function suppressMenuNavigation(event: MouseEvent) {
		event.preventDefault();
		event.stopPropagation();
	}

	async function confirmDelete() {
		deleting = true;
		const body = new FormData();
		body.set('sessionId', session.id);
		const response = await fetch('?/deleteSession', { method: 'POST', body });
		const result = deserialize(await response.text());
		deleting = false;
		if (result.type === 'success') {
			confirmOpen = false;
			await invalidateAll();
		}
	}
</script>

<div class="session-actions">
	<Menu bind:open={menuOpen}>
		{#snippet trigger({ toggle })}
			<IconButton
				onclick={(event) => {
					suppressMenuNavigation(event);
					toggle();
				}}
				aria-label={m.common_more_options()}
			>
				<IconMore size={18} />
			</IconButton>
		{/snippet}
		<div class="w-full">
			<MenuItem onclick={openRename}>{m.common_rename()}</MenuItem>
			<MenuItem onclick={togglePin}>{session.pinned ? m.chat_unpin() : m.chat_pin()}</MenuItem>
			{#if !session.project_id}<MenuItem onclick={openMove}>{m.chat_move()}</MenuItem>{/if}
			<MenuItem onclick={openDeleteConfirm}>{m.common_delete()}</MenuItem>
		</div>
	</Menu>
</div>

<BottomSheet bind:open={renameOpen}>
	{#snippet children()}
		<form onsubmit={saveRename}>
			<h2 class="md-title-large" style="color: var(--md-sys-color-on-surface); margin: 0 0 14px;">{m.chat_rename()}</h2>
			<TextField label={m.chat_rename_label()} bind:value={renameValue} maxlength={80} error={!!renameError} supportingText={renameError ?? undefined} />
			<div class="flex justify-end gap-2" style="margin-top: 16px;">
				<Button variant="text" type="button" onclick={() => (renameOpen = false)} disabled={renaming}>{m.common_cancel()}</Button>
				<Button variant="filled" type="submit" disabled={renaming}>{m.common_save()}</Button>
			</div>
		</form>
	{/snippet}
</BottomSheet>

{#if updateError && !moveOpen}<p class="session-actions__error" role="alert">{updateError}</p>{/if}

<BottomSheet bind:open={moveOpen}>
	{#snippet children()}
		<form onsubmit={saveMove} class="move">
			<h2 class="md-title-large" style="color: var(--md-sys-color-on-surface); margin: 0;">{m.move_title()}</h2>
			<Search placeholder={m.move_search()} bind:value={moveQuery} />
			<div role="radiogroup" aria-label={m.move_title()} class="move__list">
				<label class="move__row">
					<input type="radio" name="projectId" value="" bind:group={moveTo} />
					<span>{m.move_out()}</span>
				</label>
				{#each shownProjects as project (project.id)}
					<label class="move__row">
						<input type="radio" name="projectId" value={project.id} bind:group={moveTo} />
						<span class="move__name">{project.name}</span>
					</label>
				{:else}
					{#if moveQuery.trim()}<p class="move__none">{m.move_none()}</p>{/if}
				{/each}
			</div>
			{#if updateError}<p class="move__error" role="alert">{updateError}</p>{/if}
			<div class="flex justify-end gap-2">
				<Button variant="text" type="button" onclick={() => (moveOpen = false)} disabled={moving}>{m.common_cancel()}</Button>
				<Button variant="filled" type="submit" disabled={moving || moveTo === (session.in_project_id ?? '')}>{m.move_confirm()}</Button>
			</div>
		</form>
	{/snippet}
</BottomSheet>

<BottomSheet bind:open={confirmOpen}>
	{#snippet children()}
		<h2 class="md-title-large" style="color: var(--md-sys-color-on-surface); margin: 0 0 8px;">{m.session_delete_title()}</h2>
		<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant); margin: 0 0 16px;">
			{#if session.project_id}
				{m.session_delete_project_body()}
			{:else}
				{m.session_delete_body()}
			{/if}
		</p>
		<div class="flex justify-end gap-2">
			<Button variant="text" onclick={() => (confirmOpen = false)} disabled={deleting}>{m.common_cancel()}</Button>
			<Button variant="filled" onclick={confirmDelete} disabled={deleting}>{deleting ? m.common_deleting() : m.common_delete()}</Button>
		</div>
	{/snippet}
</BottomSheet>

<style>
	.session-actions {
		flex-shrink: 0;
	}
	.session-actions__error,
	.move__error {
		margin: 0;
		color: var(--md-sys-color-error);
		font-size: var(--md-sys-typescale-body-small-size);
	}
	.move {
		display: flex;
		flex-direction: column;
		gap: 16px;
		/* The sheet opens from a table's actions cell, which right-aligns its text. */
		text-align: start;
	}
	.move__list {
		display: flex;
		max-height: 320px;
		flex-direction: column;
		gap: 2px;
		overflow-y: auto;
	}
	.move__row {
		display: flex;
		min-height: 48px;
		align-items: center;
		gap: 12px;
		padding: 0 12px;
		border-radius: var(--md-sys-shape-corner-large);
		color: var(--md-sys-color-on-surface);
		cursor: pointer;
	}
	.move__row:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
	.move__row:has(input:checked) {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.move__row input {
		accent-color: var(--md-sys-color-primary);
	}
	.move__row:has(input:focus-visible) {
		outline: 3px solid var(--md-sys-color-primary);
		outline-offset: -3px;
	}
	.move__name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.move__none {
		margin: 0;
		padding: 8px 12px;
		color: var(--md-sys-color-on-surface-variant);
	}
</style>
