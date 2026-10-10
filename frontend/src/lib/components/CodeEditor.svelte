<!-- An editable file in VS Code's look (Dark Modern + Dark+ colours). Fills its container.
     Cmd/Ctrl+S saves; it reports unsaved changes and the cursor for the status bar. -->
<script lang="ts">
	import { onMount } from 'svelte';
	import { EditorState } from '@codemirror/state';
	import {
		EditorView,
		keymap,
		lineNumbers,
		highlightActiveLine,
		highlightActiveLineGutter,
		highlightSpecialChars,
		drawSelection,
		rectangularSelection,
		crosshairCursor
	} from '@codemirror/view';
	import { defaultKeymap, history, historyKeymap, indentWithTab } from '@codemirror/commands';
	import { bracketMatching, foldGutter, foldKeymap, indentOnInput } from '@codemirror/language';
	import { closeBrackets, closeBracketsKeymap } from '@codemirror/autocomplete';
	import { highlightSelectionMatches, searchKeymap } from '@codemirror/search';
	import { vscodeDark } from '$lib/code/vscodeTheme';
	import { languageFor } from '$lib/code/language';

	let {
		path,
		value,
		onSave,
		onDirty,
		onCursor
	}: {
		path: string;
		value: string;
		onSave: (content: string) => void;
		onDirty?: (dirty: boolean) => void;
		onCursor?: (line: number, column: number) => void;
	} = $props();

	let container: HTMLDivElement | undefined = $state();
	let view: EditorView | undefined;
	// svelte-ignore state_referenced_locally
	let saved = value;

	function save() {
		const content = view?.state.doc.toString() ?? saved;
		saved = content;
		onDirty?.(false);
		onSave(content);
		return true;
	}

	onMount(() => {
		if (!container) return;
		view = new EditorView({
			state: EditorState.create({
				doc: value,
				extensions: [
					lineNumbers(),
					highlightActiveLineGutter(),
					foldGutter({ openText: '⌄', closedText: '›' }),
					highlightSpecialChars(),
					history(),
					drawSelection(),
					indentOnInput(),
					bracketMatching(),
					closeBrackets(),
					rectangularSelection(),
					crosshairCursor(),
					highlightActiveLine(),
					highlightSelectionMatches(),
					keymap.of([
						{ key: 'Mod-s', run: save, preventDefault: true },
						indentWithTab,
						...closeBracketsKeymap,
						...defaultKeymap,
						...searchKeymap,
						...historyKeymap,
						...foldKeymap
					]),
					vscodeDark,
					...languageFor(path),
					EditorView.updateListener.of((update) => {
						if (update.docChanged) onDirty?.(update.state.doc.toString() !== saved);
						if (update.selectionSet || update.docChanged) {
							const head = update.state.selection.main.head;
							const line = update.state.doc.lineAt(head);
							onCursor?.(line.number, head - line.from + 1);
						}
					})
				]
			}),
			parent: container
		});
		onCursor?.(1, 1);
		return () => view?.destroy();
	});

	export function getContent(): string {
		return view?.state.doc.toString() ?? value;
	}

	export function saveNow() {
		save();
	}
</script>

<div bind:this={container} class="code-editor"></div>

<style>
	.code-editor {
		height: 100%;
		min-height: 0;
		background: #1f1f1f;
	}
	.code-editor :global(.cm-editor) {
		height: 100%;
	}
	.code-editor :global(.cm-scroller) {
		overflow: auto;
	}
</style>
