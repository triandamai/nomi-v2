<!-- frontend/src/lib/components/blocks/FileChangeBlock.svelte -->
<!-- A file Koda created, edited or deleted, as a VS Code-style tab: collapsed to its path and
     line counts, opening onto the code (or the diff). It opens by itself when it first arrives
     so the person can watch the change land, and starts collapsed in history. -->
<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { EditorState, type Extension } from '@codemirror/state';
	import { EditorView, lineNumbers, highlightSpecialChars, drawSelection } from '@codemirror/view';
	import { unifiedMergeView } from '@codemirror/merge';
	import { css } from '@codemirror/lang-css';
	import { html } from '@codemirror/lang-html';
	import { javascript } from '@codemirror/lang-javascript';
	import { vscodeDark } from '$lib/code/vscodeTheme';
	import { diffStats, fileBadge } from '$lib/code/diffStats';
	import type { ContentBlock } from '$lib/types';

	let {
		block,
		fresh = false
	}: {
		block: Extract<ContentBlock, { kind: 'file_write' | 'file_delete' }>;
		/** Arrived just now (not loaded from history): start open. */
		fresh?: boolean;
	} = $props();

	// svelte-ignore state_referenced_locally
	let open = $state(fresh);
	let container: HTMLDivElement | undefined = $state();

	const deleted = $derived(block.kind === 'file_delete');
	const before = $derived(block.previous_content ?? null);
	const after = $derived(block.kind === 'file_write' ? block.content : null);
	const stats = $derived(diffStats(before, after));
	const action = $derived(deleted ? m.block_deleted() : before !== null ? m.file_updated() : m.file_created());
	const badge = $derived(fileBadge(block.path));
	const fileName = $derived(block.path.split('/').pop() ?? block.path);
	const folder = $derived(block.path.includes('/') ? block.path.slice(0, block.path.lastIndexOf('/')) : '');
	// A delete with no saved content (older messages) has nothing to open.
	const hasBody = $derived(!deleted || before !== null);

	function language(path: string): Extension[] {
		const ext = path.split('.').pop()?.toLowerCase();
		if (ext === 'html' || ext === 'htm' || ext === 'svelte' || ext === 'vue' || ext === 'astro') return [html()];
		if (ext === 'css') return [css()];
		if (ext === 'ts' || ext === 'mts' || ext === 'cts') return [javascript({ typescript: true })];
		if (ext === 'tsx') return [javascript({ typescript: true, jsx: true })];
		if (ext === 'jsx') return [javascript({ jsx: true })];
		if (ext === 'js' || ext === 'mjs' || ext === 'cjs' || ext === 'json') return [javascript()];
		return [];
	}

	$effect(() => {
		if (!open || !container) return;
		const base: Extension[] = [
			lineNumbers(),
			highlightSpecialChars(),
			drawSelection(),
			EditorState.readOnly.of(true),
			EditorView.editable.of(false),
			vscodeDark,
			...language(block.path)
		];
		const diff =
			block.kind === 'file_write' && before !== null
				? [unifiedMergeView({ original: before, mergeControls: false, highlightChanges: true, gutter: true, collapseUnchanged: { margin: 3, minSize: 6 } })]
				: [];
		const doc = after ?? before ?? '';
		const view = new EditorView({ state: EditorState.create({ doc, extensions: [...base, ...diff] }), parent: container });
		return () => view.destroy();
	});
</script>

<div class="file-card" class:file-card--deleted={deleted} style:--file-badge={badge.color}>
	<button type="button" class="file-card__tab" aria-expanded={hasBody ? open : undefined} disabled={!hasBody} onclick={() => (open = !open)}>
		{#if hasBody}
			<svg class="file-card__chevron" class:file-card__chevron--open={open} width="16" height="16" viewBox="0 0 16 16" aria-hidden="true">
				<path d="M6 4l4 4-4 4" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
			</svg>
		{/if}
		<span class="file-card__badge" aria-hidden="true">{badge.label}</span>
		<span class="file-card__path">
			<span class="file-card__name">{fileName}</span>
			{#if folder}<span class="file-card__folder">{folder}</span>{/if}
		</span>
		<span class="file-card__action">{action}</span>
		<span class="file-card__stats">
			{#if stats.added > 0}<span class="file-card__added">+{stats.added}</span>{/if}
			{#if stats.removed > 0}<span class="file-card__removed">−{stats.removed}</span>{/if}
		</span>
	</button>
	{#if open && hasBody}
		<div class="file-card__editor" class:file-card__editor--deleted={deleted} bind:this={container}></div>
	{/if}
</div>

<style>
	.file-card {
		border-radius: 4px;
		overflow: hidden;
		border: 1px solid #2b2b2b;
		background: #1f1f1f;
		color: #cccccc;
		width: 100%;
		min-width: 0;
	}
	.file-card__tab {
		display: flex;
		align-items: center;
		gap: 6px;
		width: 100%;
		min-height: 36px;
		padding: 0 10px 0 6px;
		background: #181818;
		border: none;
		border-top: 2px solid transparent;
		color: inherit;
		font: 13px/1 system-ui, -apple-system, 'Segoe UI', sans-serif;
		text-align: left;
		cursor: pointer;
	}
	.file-card__tab[aria-expanded='true'] {
		background: #1f1f1f;
		border-top-color: #0078d4;
		border-bottom: 1px solid #2b2b2b;
	}
	.file-card__tab:hover:not(:disabled) {
		background: #2a2d2e;
	}
	.file-card__tab:focus-visible {
		outline: 1px solid #0078d4;
		outline-offset: -1px;
	}
	.file-card__tab:disabled {
		cursor: default;
		padding-left: 10px;
	}
	.file-card__chevron {
		flex: none;
		color: #9d9d9d;
		transition: transform 120ms ease-out;
	}
	.file-card__chevron--open {
		transform: rotate(90deg);
	}
	.file-card__badge {
		flex: none;
		font: 600 10px/1 'Cascadia Code', Menlo, Consolas, monospace;
		color: var(--file-badge);
		min-width: 18px;
		text-align: center;
	}
	.file-card__path {
		display: flex;
		align-items: baseline;
		gap: 6px;
		min-width: 0;
		flex: 1;
		overflow: hidden;
		white-space: nowrap;
	}
	.file-card__name {
		color: #e7e7e7;
		flex: none;
	}
	.file-card--deleted .file-card__name {
		text-decoration: line-through;
		color: #9d9d9d;
	}
	.file-card__folder {
		color: #8b8b8b;
		font-size: 12px;
		overflow: hidden;
		text-overflow: ellipsis;
		direction: rtl;
		text-align: left;
	}
	.file-card__action {
		flex: none;
		color: #9d9d9d;
		font-size: 12px;
	}
	.file-card__stats {
		flex: none;
		display: flex;
		gap: 6px;
		font: 12px/1 'Cascadia Code', Menlo, Consolas, monospace;
		font-variant-numeric: tabular-nums;
	}
	.file-card__added {
		color: #3fb950;
	}
	.file-card__removed {
		color: #f85149;
	}
	.file-card__editor {
		max-height: 360px;
		overflow: auto;
	}
	.file-card__editor--deleted :global(.cm-line) {
		background: #f8514914;
	}
	@media (prefers-reduced-motion: reduce) {
		.file-card__chevron {
			transition: none;
		}
	}
</style>
