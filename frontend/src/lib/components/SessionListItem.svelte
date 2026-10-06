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
	import IconFolder from './icons/IconFolder.svelte';
	import IconMore from './icons/IconMore.svelte';
	import { timeAgo } from '$lib/i18n';
	import type { SessionSummary } from '$lib/types';

	let { session }: { session: SessionSummary } = $props();

	let menuOpen = $state(false);
	let confirmOpen = $state(false);
	let deleting = $state(false);
	let renameOpen = $state(false);
	let renameValue = $state('');
	let renaming = $state(false);
	let renameError = $state<string | null>(null);

	const href = $derived(session.project_id ? `/projects/session/${session.id}` : `/chat/${session.id}`);
	const title = $derived(session.title ?? m.nav_new_chat());

	// A message that's entirely one fenced code block (the common "show me some code" reply)
	// makes a useless, unreadable list preview — show what kind of content it is instead of
	// dumping raw source. Anything that isn't wholly a single fence falls through unchanged.
	function previewText(content: string | undefined | null): string {
		if (!content) return m.session_no_messages();
		const fenceMatch = content.trim().match(/^```(\S*)\r?\n[\s\S]*?```$/);
		if (fenceMatch) {
			const lang = fenceMatch[1]?.trim();
			return lang ? m.session_lang_code({ lang }) : m.session_code();
		}
		return content;
	}

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

<div class="m3-session-item">
	<a {href} class="m3-session-item__link">
		<span
			class="md-label-small m3-session-item__badge"
			class:m3-session-item__badge--project={!!session.project_id}
		>
			{#if session.project_id}<IconFolder size={12} />{/if}
			{session.project_id ? m.session_project() : m.session_chat()}
		</span>
		<p class="md-body-large m3-session-item__title">{title}</p>
		<div class="flex items-center gap-2">
			<span class="md-body-small flex-1 truncate" style="color: var(--md-sys-color-on-surface-variant)">
				{previewText(session.last_message?.content)}
			</span>
			<span class="md-body-small shrink-0" style="color: var(--md-sys-color-outline)">{timeAgo(session.updated_at)}</span>
		</div>
	</a>
	<div class="m3-session-item__menu">
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
	.m3-session-item {
		display: flex;
		align-items: center;
		gap: 4px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		padding: 4px 4px 4px 0;
		background: var(--md-sys-color-surface-container-lowest);
		transition:
			border-radius var(--nomi-motion-spatial-fast),
			background-color var(--nomi-motion-effects-fast);
	}
	.m3-session-item:hover {
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-primary-container);
	}

	.m3-session-item__link {
		display: flex;
		min-width: 0;
		flex: 1;
		flex-direction: column;
		gap: 4px;
		padding: 12px 8px 12px 18px;
		text-decoration: none;
	}

	.m3-session-item__badge {
		display: inline-flex;
		width: fit-content;
		align-items: center;
		gap: 4px;
		border-radius: var(--md-sys-shape-corner-full);
		padding: 1px 8px;
		color: var(--md-sys-color-on-surface-variant);
		background: var(--md-sys-color-surface-container-highest);
	}
	.m3-session-item__badge--project {
		color: var(--md-sys-color-on-secondary-container);
		background: var(--md-sys-color-secondary-container);
	}

	.m3-session-item__title {
		color: var(--md-sys-color-on-surface);
		font-weight: 650;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.m3-session-item__menu {
		flex-shrink: 0;
	}
</style>
