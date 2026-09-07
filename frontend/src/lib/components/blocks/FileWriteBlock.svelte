<!-- frontend/src/lib/components/blocks/FileWriteBlock.svelte -->
<script lang="ts">
	import { EditorState } from '@codemirror/state';
	import { EditorView, basicSetup } from 'codemirror';
	import { MergeView } from '@codemirror/merge';
	import { css } from '@codemirror/lang-css';
	import { html } from '@codemirror/lang-html';
	import { javascript } from '@codemirror/lang-javascript';
	import type { ContentBlock } from '$lib/types';

	let { block }: { block: Extract<ContentBlock, { kind: 'file_write' }> } = $props();

	let container: HTMLDivElement | undefined = $state();

	function languageExtension(path: string) {
		const ext = path.split('.').pop()?.toLowerCase();
		if (ext === 'html' || ext === 'htm') return html();
		if (ext === 'css') return css();
		if (ext === 'js' || ext === 'mjs' || ext === 'jsx' || ext === 'ts' || ext === 'tsx') return javascript();
		return null;
	}

	$effect(() => {
		if (!container) return;
		const lang = languageExtension(block.path);
		const readOnlyExtensions = [basicSetup, EditorView.editable.of(false), ...(lang ? [lang] : [])];

		let view: MergeView | EditorView;
		if (block.previous_content !== null) {
			view = new MergeView({
				a: { doc: block.previous_content, extensions: readOnlyExtensions },
				b: { doc: block.content, extensions: readOnlyExtensions },
				parent: container,
			});
		} else {
			view = new EditorView({ state: EditorState.create({ doc: block.content, extensions: readOnlyExtensions }), parent: container });
		}

		return () => view.destroy();
	});
</script>

<div class="m3-block-card m3-block-card--write">
	<div class="m3-block-card__header">
		<span aria-hidden="true">{block.previous_content !== null ? '📝' : '✨'}</span>
		<span class="md-body-medium">{block.previous_content !== null ? 'Updated' : 'Created'} <code>{block.path}</code></span>
	</div>
	<div class="m3-block-card__editor" bind:this={container}></div>
</div>

<style>
	.m3-block-card {
		border-radius: var(--md-sys-shape-corner-medium);
		border: 1px solid var(--md-sys-color-outline-variant);
		overflow: hidden;
	}
	.m3-block-card--write {
		background: var(--md-sys-color-surface-container-low);
	}
	.m3-block-card__header {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 14px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.m3-block-card__editor {
		max-height: 320px;
		overflow: auto;
		font-size: 0.85em;
	}
</style>
