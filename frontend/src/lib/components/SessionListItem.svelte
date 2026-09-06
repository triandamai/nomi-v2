<script lang="ts">
	import { deserialize } from '$app/forms';
	import { invalidateAll } from '$app/navigation';
	import BottomSheet from './m3/BottomSheet.svelte';
	import Button from './m3/Button.svelte';
	import IconButton from './m3/IconButton.svelte';
	import Menu from './m3/Menu.svelte';
	import MenuItem from './m3/MenuItem.svelte';
	import IconFolder from './icons/IconFolder.svelte';
	import IconMore from './icons/IconMore.svelte';
	import type { SessionSummary } from '$lib/types';

	let { session }: { session: SessionSummary } = $props();

	let menuOpen = $state(false);
	let confirmOpen = $state(false);
	let deleting = $state(false);

	const href = $derived(session.project_id ? `/projects/session/${session.id}` : `/chat/${session.id}`);
	const title = $derived(session.title ?? 'New chat');

	function timeAgo(iso: string): string {
		const diffMs = Date.now() - new Date(iso).getTime();
		const minutes = Math.floor(diffMs / 60000);
		if (minutes < 1) return 'just now';
		if (minutes < 60) return `${minutes}m`;
		const hours = Math.floor(minutes / 60);
		if (hours < 24) return `${hours}h`;
		return `${Math.floor(hours / 24)}d`;
	}

	// A message that's entirely one fenced code block (the common "show me some code" reply)
	// makes a useless, unreadable list preview — show what kind of content it is instead of
	// dumping raw source. Anything that isn't wholly a single fence falls through unchanged.
	function previewText(content: string | undefined | null): string {
		if (!content) return 'No messages yet';
		const fenceMatch = content.trim().match(/^```(\S*)\r?\n[\s\S]*?```$/);
		if (fenceMatch) {
			const lang = fenceMatch[1]?.trim();
			return lang ? `${lang} code` : 'code';
		}
		return content;
	}

	function openDeleteConfirm(event: MouseEvent) {
		event.preventDefault();
		event.stopPropagation();
		menuOpen = false;
		confirmOpen = true;
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
			{session.project_id ? 'Project' : 'Chat'}
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
					aria-label="More options"
				>
					<IconMore size={18} />
				</IconButton>
			{/snippet}
			<div class="w-full">
				<MenuItem onclick={openDeleteConfirm}>Delete</MenuItem>
			</div>
		</Menu>
	</div>
</div>

<BottomSheet bind:open={confirmOpen}>
	{#snippet children()}
		<h2 class="md-title-large" style="color: var(--md-sys-color-on-surface); margin: 0 0 8px;">Delete this chat?</h2>
		<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant); margin: 0 0 20px;">
			{#if session.project_id}
				This permanently deletes the conversation and its project, including all its files. This can't be undone.
			{:else}
				This permanently deletes the conversation. This can't be undone.
			{/if}
		</p>
		<div class="flex justify-end gap-2">
			<Button variant="text" onclick={() => (confirmOpen = false)} disabled={deleting}>Cancel</Button>
			<Button variant="filled" onclick={confirmDelete} disabled={deleting}>{deleting ? 'Deleting…' : 'Delete'}</Button>
		</div>
	{/snippet}
</BottomSheet>

<style>
	.m3-session-item {
		display: flex;
		align-items: center;
		gap: 4px;
		border-radius: var(--md-sys-shape-corner-small);
		padding: 4px 4px 4px 0;
	}
	.m3-session-item:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}

	.m3-session-item__link {
		display: flex;
		min-width: 0;
		flex: 1;
		flex-direction: column;
		gap: 4px;
		padding: 8px 8px 8px 12px;
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
		font-weight: 500;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.m3-session-item__menu {
		flex-shrink: 0;
	}
</style>
