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
	import type { SessionSummary } from '$lib/types';

	// A chat's ⋯ menu (rename, delete) with its sheets; posts to the page's renameSession and
	// deleteSession actions.
	let { session }: { session: SessionSummary } = $props();

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
			<MenuItem onclick={openDeleteConfirm}>{m.common_delete()}</MenuItem>
		</div>
	</Menu>
</div>

<BottomSheet bind:open={renameOpen}>
	{#snippet children()}
		<form onsubmit={saveRename}>
			<h2 class="md-title-large" style="color: var(--md-sys-color-on-surface); margin: 0 0 16px;">{m.chat_rename()}</h2>
			<TextField label={m.chat_rename_label()} bind:value={renameValue} maxlength={80} error={!!renameError} supportingText={renameError ?? undefined} />
			<div class="flex justify-end gap-2" style="margin-top: 20px;">
				<Button variant="text" type="button" onclick={() => (renameOpen = false)} disabled={renaming}>{m.common_cancel()}</Button>
				<Button variant="filled" type="submit" disabled={renaming}>{m.common_save()}</Button>
			</div>
		</form>
	{/snippet}
</BottomSheet>

<BottomSheet bind:open={confirmOpen}>
	{#snippet children()}
		<h2 class="md-title-large" style="color: var(--md-sys-color-on-surface); margin: 0 0 8px;">{m.session_delete_title()}</h2>
		<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant); margin: 0 0 20px;">
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
</style>
