<script lang="ts">
	// Members → Versions: every version of the project's files, newest first ("Version 4 by Koda"),
	// with the prompt it answered, and Restore to put any earlier one back as a new version
	// (backend routes::project_versions).
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import Button from '$lib/components/m3/Button.svelte';
	import { ago, projectApi } from '$lib/projectApi';
	import { canRestore, versionAuthor } from '$lib/projectVersions';
	import type { ProjectVersion, ProjectVersions } from '$lib/types';

	const PAGE_SIZE = 50;

	let { projectId, canBuild = true }: { projectId: string; canBuild?: boolean } = $props();

	const locale = getLocale();
	let versions = $state<ProjectVersion[]>([]);
	let hasMore = $state(false);
	let loaded = $state(false);
	let error = $state<string | null>(null);
	let notice = $state<string | null>(null);
	let restoring = $state<number | null>(null);

	async function refresh() {
		try {
			const page = await projectApi<ProjectVersions>(projectId, `versions?limit=${PAGE_SIZE}`);
			versions = page.versions ?? [];
			hasMore = page.has_more;
			error = null;
		} catch (e) {
			error = (e as Error).message;
		} finally {
			loaded = true;
		}
	}

	async function more() {
		const before = versions.at(-1)?.version;
		if (before === undefined) return;
		try {
			const page = await projectApi<ProjectVersions>(projectId, `versions?limit=${PAGE_SIZE}&before=${before}`);
			versions = [...versions, ...(page.versions ?? [])];
			hasMore = page.has_more;
		} catch (e) {
			error = (e as Error).message;
		}
	}

	async function restore(version: ProjectVersion) {
		restoring = version.version;
		error = null;
		notice = null;
		try {
			const done = await projectApi<{ version: number }>(projectId, `versions/${version.version}/restore`, { method: 'POST' });
			notice = m.pj_ver_restored({ version: version.version, next: done.version });
			await refresh();
		} catch (e) {
			error = (e as Error).message;
		} finally {
			restoring = null;
		}
	}

	$effect(() => {
		void projectId;
		void refresh();
	});

	function title(version: ProjectVersion): string {
		const name = versionAuthor(version);
		return name ? m.pj_ver_by({ version: version.version, name }) : m.pj_ver_plain({ version: version.version });
	}

	function detail(version: ProjectVersion): string {
		if (version.restored_from !== null) return m.pj_ver_restored_from({ version: version.restored_from });
		if (!version.restorable) return m.pj_ver_not_kept();
		return version.summary ?? m.pj_ver_files({ count: version.files });
	}
</script>

<section aria-label={m.pj_ver_tab()}>
	{#if notice}<p class="notice" role="status">{notice}</p>{/if}
	{#if error}<p class="error" role="alert">{error}</p>{/if}
	{#if loaded && versions.length === 0 && !error}
		<p class="muted">{m.pj_ver_empty()}</p>
	{:else if versions.length > 0}
		<ul class="list">
			{#each versions as version (version.version)}
				<li class="row">
					<span class="row__text">
						<span class="row__title">{title(version)}</span>
						<span class="row__detail">{detail(version)}</span>
					</span>
					<time class="row__when" datetime={version.at}>{ago(version.at, locale)}</time>
					{#if version.current}
						<span class="row__current">{m.pj_ver_current()}</span>
					{:else if canBuild && canRestore(version)}
						<Button variant="text" size="s" disabled={restoring !== null} onclick={() => restore(version)}>{m.pj_ver_restore()}</Button>
					{/if}
				</li>
			{/each}
		</ul>
		{#if hasMore}
			<div class="more"><Button variant="text" size="m" onclick={more}>{m.pj_act_more()}</Button></div>
		{/if}
	{/if}
</section>

<style>
	.muted {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.error {
		color: var(--md-sys-color-error);
		font-size: 0.875rem;
	}
	.notice {
		color: var(--md-sys-color-on-surface);
		font-size: 0.875rem;
	}
	.list {
		margin: 0;
		padding: 0 16px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-lowest);
		list-style: none;
	}
	.row {
		display: grid;
		grid-template-columns: 1fr auto auto;
		gap: 12px;
		align-items: center;
		min-height: 56px;
		padding: 10px 0;
		border-top: 1px solid var(--md-sys-color-surface-container-low);
	}
	.row:first-child {
		border-top: 0;
	}
	.row__text {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.row__title {
		color: var(--md-sys-color-on-surface);
		font-size: 0.875rem;
		font-weight: 600;
	}
	.row__detail {
		overflow: hidden;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.row__when {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
		white-space: nowrap;
	}
	.row__current {
		color: var(--md-sys-color-primary);
		font-size: 0.8125rem;
		font-weight: 600;
	}
	.more {
		display: flex;
		justify-content: center;
		margin-top: 8px;
	}
</style>
