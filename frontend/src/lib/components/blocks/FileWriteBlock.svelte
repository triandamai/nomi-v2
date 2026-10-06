<!-- frontend/src/lib/components/blocks/FileWriteBlock.svelte -->
<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { EditorState } from '@codemirror/state';
	import { EditorView, basicSetup } from 'codemirror';
	import { MergeView } from '@codemirror/merge';
	import { css } from '@codemirror/lang-css';
	import { html } from '@codemirror/lang-html';
	import { javascript } from '@codemirror/lang-javascript';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
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
		<AgentShape agent="coding" size={22} face />
		<span class="md-body-medium">{block.previous_content !== null ? m.file_updated() : m.file_created()} <code>{block.path}</code></span>
	</div>
	<div class="m3-block-card__editor" bind:this={container}></div>
</div>

<style>
	.m3-block-card {
		border-radius: var(--md-sys-shape-corner-large-increased);
		overflow: hidden;
	}
	.m3-block-card--write {
		background: var(--md-sys-color-surface-container-lowest);
	}
	.m3-block-card__header code {
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.85em;
	}
	.m3-block-card__header {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 12px 16px;
		background: color-mix(in srgb, #3fb8c8 14%, var(--md-sys-color-surface-container-lowest));
	}
	.m3-block-card__editor {
		max-height: 320px;
		overflow: auto;
		font-size: 0.85em;
	}
</style>
