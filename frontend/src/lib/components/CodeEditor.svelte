<script lang="ts">
	import { onMount } from 'svelte';
	import { EditorView, basicSetup } from 'codemirror';
	import { javascript } from '@codemirror/lang-javascript';
	import { html } from '@codemirror/lang-html';
	import { css } from '@codemirror/lang-css';
	import { EditorState } from '@codemirror/state';

	let {
		path,
		value,
		onSave,
	}: {
		path: string;
		value: string;
		onSave: (content: string) => void;
	} = $props();

	let container: HTMLDivElement | undefined = $state();
	let view: EditorView | undefined;

	function languageFor(path: string) {
		if (path.endsWith('.html')) return html();
		if (path.endsWith('.css')) return css();
		if (path.endsWith('.js') || path.endsWith('.mjs')) return javascript();
		return [];
	}

	function createEditor(path: string, value: string) {
		view?.destroy();
		if (!container) return;
		view = new EditorView({
			state: EditorState.create({ doc: value, extensions: [basicSetup, languageFor(path)] }),
			parent: container,
		});
	}

	onMount(() => {
		createEditor(path, value);
		return () => view?.destroy();
	});

	export function getContent(): string {
		return view?.state.doc.toString() ?? value;
	}
</script>

<div class="flex h-full flex-col">
	<div bind:this={container} class="min-h-0 flex-1 overflow-auto" style="background: var(--md-sys-color-surface-container-lowest)"></div>
	<div class="flex justify-end p-2" style="border-top: 1px solid var(--md-sys-color-outline-variant)">
		<button
			type="button"
			class="md-label-large rounded-full px-4 py-2"
			style="background: var(--md-sys-color-primary); color: var(--md-sys-color-on-primary); border: none; cursor: pointer"
			onclick={() => onSave(getContent())}
		>
			Save
		</button>
	</div>
</div>
