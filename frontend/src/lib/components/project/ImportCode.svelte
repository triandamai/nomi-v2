<script lang="ts">
	// Brings existing code into the project: a .zip, or a public GitHub repository.
	import { m } from '$lib/paraglide/messages';
	import Button from '$lib/components/m3/Button.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import { importGithub, importZip, type ImportResult } from '$lib/projectImport';

	let {
		projectId,
		prepare,
		ondone,
	}: {
		projectId?: string;
		/** Makes the project to import into (when there's none yet), returning its id. */
		prepare?: () => Promise<string>;
		ondone?: (result: ImportResult) => void;
	} = $props();

	let mode = $state<'zip' | 'github'>('zip');
	let file = $state<File | null>(null);
	let repo = $state('');
	let ref = $state('');
	let busy = $state(false);
	let progress = $state<string | null>(null);
	let result = $state<ImportResult | null>(null);
	let error = $state<string | null>(null);

	async function run(event: SubmitEvent) {
		event.preventDefault();
		busy = true;
		error = null;
		result = null;
		try {
			const target = projectId ?? (await prepare?.());
			if (!target) return;
			if (mode === 'zip') {
				if (!file) return;
				result = await importZip(target, file, (sent, total) => (progress = m.import_progress({ sent, total })));
			} else {
				progress = m.import_downloading();
				result = await importGithub(target, repo.trim(), ref.trim());
			}
			ondone?.(result);
		} catch (e) {
			const message = (e as Error).message;
			error = message === 'empty' ? m.import_empty() : message || m.import_failed();
		} finally {
			busy = false;
			progress = null;
		}
	}
</script>

<form class="import" onsubmit={run}>
	<div class="modes" role="tablist" aria-label={m.import_title()}>
		<button type="button" role="tab" class="mode" aria-selected={mode === 'zip'} onclick={() => (mode = 'zip')}>{m.import_zip()}</button>
		<button type="button" role="tab" class="mode" aria-selected={mode === 'github'} onclick={() => (mode = 'github')}>{m.import_github()}</button>
	</div>
	{#if mode === 'zip'}
		<label class="file">
			<span class="file__label">{file ? file.name : m.import_pick_zip()}</span>
			<input type="file" accept=".zip,application/zip" onchange={(e) => (file = (e.currentTarget as HTMLInputElement).files?.[0] ?? null)} />
		</label>
	{:else}
		<TextField name="repo" label={m.import_repo()} placeholder="owner/repo" bind:value={repo} />
		<TextField name="ref" label={m.import_ref()} bind:value={ref} />
	{/if}
	<p class="hint">{m.import_hint()}</p>
	{#if progress}<p class="hint" role="status">{progress}</p>{/if}
	{#if error}<p class="error" role="alert">{error}</p>{/if}
	{#if result}<p class="ok" role="status">{m.import_done({ imported: result.imported, skipped: result.skipped })}</p>{/if}
	<div>
		<Button type="submit" variant="filled" size="s" disabled={busy || (mode === 'zip' ? !file : !repo.trim())}>{busy ? m.import_importing() : m.import_button()}</Button>
	</div>
</form>

<style>
	.import {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}
	.modes {
		display: inline-flex;
		align-self: flex-start;
		gap: 2px;
		padding: 3px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-high);
	}
	.mode {
		min-height: 36px;
		padding: 0 14px;
		border: none;
		border-radius: var(--md-sys-shape-corner-full);
		background: none;
		color: var(--md-sys-color-on-surface-variant);
		font: inherit;
		font-size: 0.875rem;
		font-weight: 600;
		cursor: pointer;
	}
	.mode[aria-selected='true'] {
		background: var(--md-sys-color-surface);
		color: var(--md-sys-color-on-surface);
	}
	.mode:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.file {
		position: relative;
		display: flex;
		align-items: center;
		min-height: 56px;
		padding: 0 16px;
		border: 1.5px dashed var(--md-sys-color-outline);
		border-radius: var(--md-sys-shape-corner-medium);
		cursor: pointer;
	}
	.file:hover,
	.file:focus-within {
		border-color: var(--md-sys-color-primary);
	}
	.file input {
		position: absolute;
		inset: 0;
		opacity: 0;
		cursor: pointer;
	}
	.file__label {
		overflow: hidden;
		color: var(--md-sys-color-on-surface-variant);
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.hint {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.error {
		margin: 0;
		color: var(--md-sys-color-error);
	}
	.ok {
		margin: 0;
		color: var(--md-sys-color-primary);
		font-weight: 600;
	}
</style>
