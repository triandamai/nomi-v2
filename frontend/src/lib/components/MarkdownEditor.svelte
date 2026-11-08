<script lang="ts">
	// Writing that's stored as Markdown but edited the way Notion does it: what you see is the
	// formatted text. Markdown shortcuts work as you type ("# ", "- ", "1. ", "[] ", "> ", "```"),
	// "/" opens a menu of blocks, and selecting text shows bold / italic / strikethrough / code.
	// With `editable={false}` it only shows the Markdown, formatted.
	import { onMount } from 'svelte';
	import { Editor, wrappingInputRule } from '@tiptap/core';
	import StarterKit from '@tiptap/starter-kit';
	import { TaskItem, TaskList } from '@tiptap/extension-list';
	import { Placeholder } from '@tiptap/extensions';
	import { Markdown } from '@tiptap/markdown';
	import {
		Bold,
		Code,
		Heading1,
		Heading2,
		Heading3,
		Italic,
		List,
		ListOrdered,
		ListTodo,
		Minus,
		Pilcrow,
		Quote,
		SquareCode,
		Strikethrough,
	} from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import { slashQuery, slashResults, type SlashBlock } from '$lib/markdownEditor';

	let {
		value = $bindable(''),
		label,
		placeholder = '',
		editable = true,
		id,
	}: {
		/** The text, as Markdown. */
		value?: string;
		/** What the field is called, for screen readers. */
		label: string;
		placeholder?: string;
		editable?: boolean;
		id?: string;
	} = $props();

	// "[] " starts a to-do, as in Notion (Tiptap itself only knows "[ ] " and "[x] ").
	const NotionTaskList = TaskList.extend({
		addInputRules() {
			return [wrappingInputRule({ find: /^\s*\[\]\s$/, type: this.type })];
		},
	});

	/** How far above the selection the formatting bar sits, in px. */
	const BUBBLE_GAP = 8;
	/** How far below the "/" the block menu opens, in px. */
	const MENU_GAP = 6;
	/** The block menu's size, in px (it flips above the "/" when there's no room below). */
	const MENU_WIDTH = 220;
	const MENU_MAX_HEIGHT = 300;
	/** The closest the menus come to the screen's edges, in px. */
	const EDGE = 8;

	const BLOCK_LABEL: Record<SlashBlock, () => string> = {
		text: m.md_block_text,
		h1: m.md_block_h1,
		h2: m.md_block_h2,
		h3: m.md_block_h3,
		bullets: m.md_block_bullets,
		numbers: m.md_block_numbers,
		todo: m.md_block_todo,
		quote: m.md_block_quote,
		code: m.md_block_code,
		divider: m.md_block_divider,
	};
	// Other words people type after "/" for the same block.
	const BLOCK_ALIASES: Record<SlashBlock, string[]> = {
		text: ['paragraph', 'p'],
		h1: ['heading', 'title', 'h1'],
		h2: ['heading', 'h2'],
		h3: ['heading', 'h3'],
		bullets: ['list', 'ul', 'bullet'],
		numbers: ['list', 'ol', 'numbered'],
		todo: ['task', 'checkbox', 'check'],
		quote: ['blockquote'],
		code: ['codeblock', 'pre'],
		divider: ['hr', 'line', 'separator'],
	};
	const BLOCK_ICON = {
		text: Pilcrow,
		h1: Heading1,
		h2: Heading2,
		h3: Heading3,
		bullets: List,
		numbers: ListOrdered,
		todo: ListTodo,
		quote: Quote,
		code: SquareCode,
		divider: Minus,
	};

	let host: HTMLDivElement | undefined = $state();
	let editor: Editor | null = null;
	// The Markdown the editor last reported, so a change from outside (a different issue opened)
	// can be told apart from the person's own typing.
	let lastEmitted = '';

	// Where the menu opens: below the "/" (top), or above it (bottom) when there's no room below.
	let slash = $state<{ from: number; to: number; query: string; left: number; top: number | null; bottom: number | null; height: number } | null>(null);
	let highlighted = $state(0);
	const results = $derived(slash ? slashResults(slash.query, (block) => [BLOCK_LABEL[block](), ...BLOCK_ALIASES[block]]) : []);

	let bubble = $state<{ left: number; top: number } | null>(null);
	let active = $state({ bold: false, italic: false, strike: false, code: false });

	function refresh() {
		if (!editor || !editable) return;
		const { state, view } = editor;
		const { empty, from, to } = state.selection;
		const cursor = state.selection.$from;
		const inCode = cursor.parent.type.spec.code === true;

		// The "/" menu: only while typing in plain text right after a "/".
		const query = empty && !inCode ? slashQuery(cursor.parent.textBetween(0, cursor.parentOffset, undefined, '￼')) : null;
		if (query === null) {
			slash = null;
		} else {
			const start = cursor.pos - query.length - 1;
			const at = view.coordsAtPos(start);
			if (!slash || slash.from !== start) highlighted = 0;
			const below = at.bottom + MENU_GAP;
			const flip = below + MENU_MAX_HEIGHT > window.innerHeight - EDGE && at.top - MENU_GAP > MENU_MAX_HEIGHT / 2;
			const left = Math.max(EDGE, Math.min(at.left, window.innerWidth - MENU_WIDTH - EDGE));
			slash = { from: start, to: cursor.pos, query, left, top: flip ? null : below, bottom: flip ? window.innerHeight - at.top + MENU_GAP : null, height: Math.min(MENU_MAX_HEIGHT, flip ? at.top - MENU_GAP - EDGE : window.innerHeight - below - EDGE) };
		}

		// The formatting bar: over a selection of text.
		if (empty || inCode || !view.hasFocus()) {
			bubble = null;
		} else {
			const a = view.coordsAtPos(from);
			const b = view.coordsAtPos(to);
			bubble = { left: (a.left + b.right) / 2, top: Math.min(a.top, b.top) - BUBBLE_GAP };
			active = {
				bold: editor.isActive('bold'),
				italic: editor.isActive('italic'),
				strike: editor.isActive('strike'),
				code: editor.isActive('code'),
			};
		}
	}

	function apply(block: SlashBlock) {
		if (!editor || !slash) return;
		const chain = editor.chain().focus().deleteRange({ from: slash.from, to: slash.to });
		if (block === 'text') chain.setParagraph();
		else if (block === 'h1') chain.setHeading({ level: 1 });
		else if (block === 'h2') chain.setHeading({ level: 2 });
		else if (block === 'h3') chain.setHeading({ level: 3 });
		else if (block === 'bullets') chain.toggleBulletList();
		else if (block === 'numbers') chain.toggleOrderedList();
		else if (block === 'todo') chain.toggleTaskList();
		else if (block === 'quote') chain.toggleBlockquote();
		else if (block === 'code') chain.toggleCodeBlock();
		else chain.setHorizontalRule();
		chain.run();
		slash = null;
	}

	function format(mark: 'bold' | 'italic' | 'strike' | 'code') {
		if (!editor) return;
		const chain = editor.chain().focus();
		if (mark === 'bold') chain.toggleBold();
		else if (mark === 'italic') chain.toggleItalic();
		else if (mark === 'strike') chain.toggleStrike();
		else chain.toggleCode();
		chain.run();
		refresh();
	}

	// The "/" menu takes the arrow keys, Enter and Escape while it's open.
	function handleKeyDown(event: KeyboardEvent): boolean {
		if (!slash || results.length === 0) return false;
		if (event.key === 'ArrowDown') {
			highlighted = (highlighted + 1) % results.length;
		} else if (event.key === 'ArrowUp') {
			highlighted = (highlighted - 1 + results.length) % results.length;
		} else if (event.key === 'Enter' || event.key === 'Tab') {
			apply(results[Math.min(highlighted, results.length - 1)]);
		} else if (event.key === 'Escape') {
			slash = null;
		} else {
			return false;
		}
		event.preventDefault();
		event.stopPropagation();
		return true;
	}

	onMount(() => {
		if (!host) return;
		lastEmitted = value;
		editor = new Editor({
			element: host,
			editable,
			extensions: [
				StarterKit.configure({ link: { openOnClick: !editable, autolink: true } }),
				NotionTaskList,
				TaskItem.configure({ nested: true }),
				Placeholder.configure({ placeholder }),
				Markdown,
			],
			content: value,
			contentType: 'markdown',
			editorProps: {
				attributes: {
					class: 'md-editor__doc',
					...(editable ? { role: 'textbox', 'aria-multiline': 'true', 'aria-label': label } : { 'aria-label': label }),
					...(id ? { id } : {}),
				},
				handleKeyDown: (_view, event) => handleKeyDown(event),
			},
			onUpdate: ({ editor: e }) => {
				lastEmitted = e.getMarkdown();
				value = lastEmitted;
				refresh();
			},
			onSelectionUpdate: refresh,
			onFocus: refresh,
			onBlur: () => {
				bubble = null;
				slash = null;
			},
		});
		// The sheet scrolling moves the text out from under the menus.
		const onScroll = () => {
			if (slash || bubble) refresh();
		};
		window.addEventListener('scroll', onScroll, true);
		return () => {
			window.removeEventListener('scroll', onScroll, true);
			editor?.destroy();
			editor = null;
		};
	});

	// A different text from outside (another issue opened in the same editor).
	$effect(() => {
		const next = value;
		if (editor && next !== lastEmitted) {
			lastEmitted = next;
			editor.commands.setContent(next, { contentType: 'markdown', emitUpdate: false });
		}
	});
</script>

<div class="md-editor" class:md-editor--readonly={!editable}>
	<div bind:this={host} class="md-editor__host"></div>

	{#if slash && results.length > 0}
		<div class="md-menu" role="listbox" aria-label={m.md_blocks()} style="left: {slash.left}px; {slash.top !== null ? `top: ${slash.top}px` : `bottom: ${slash.bottom}px`}; max-height: {slash.height}px">
			{#each results as block, index (block)}
				{@const Icon = BLOCK_ICON[block]}
				<button
					type="button"
					role="option"
					class="md-menu__item"
					aria-selected={index === highlighted}
					onmousedown={(event) => event.preventDefault()}
					onmouseenter={() => (highlighted = index)}
					onclick={() => apply(block)}
				>
					<span class="md-menu__icon"><Icon size={18} /></span>{BLOCK_LABEL[block]()}
				</button>
			{/each}
		</div>
	{/if}

	{#if bubble}
		<div class="md-bubble" role="toolbar" aria-label={m.md_format()} style="left: {bubble.left}px; top: {bubble.top}px">
			<button type="button" class="md-bubble__btn" aria-label={m.md_bold()} title={m.md_bold()} aria-pressed={active.bold} onmousedown={(event) => event.preventDefault()} onclick={() => format('bold')}><Bold size={16} /></button>
			<button type="button" class="md-bubble__btn" aria-label={m.md_italic()} title={m.md_italic()} aria-pressed={active.italic} onmousedown={(event) => event.preventDefault()} onclick={() => format('italic')}><Italic size={16} /></button>
			<button type="button" class="md-bubble__btn" aria-label={m.md_strike()} title={m.md_strike()} aria-pressed={active.strike} onmousedown={(event) => event.preventDefault()} onclick={() => format('strike')}><Strikethrough size={16} /></button>
			<button type="button" class="md-bubble__btn" aria-label={m.md_code()} title={m.md_code()} aria-pressed={active.code} onmousedown={(event) => event.preventDefault()} onclick={() => format('code')}><Code size={16} /></button>
		</div>
	{/if}
</div>

<style>
	.md-editor {
		position: relative;
		min-width: 0;
	}
	.md-editor__host :global(.md-editor__doc) {
		min-height: 160px;
		padding: 4px 0;
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-sys-typescale-body-large-font);
		font-size: var(--md-sys-typescale-body-large-size);
		line-height: 1.6;
		overflow-wrap: anywhere;
		outline: none;
	}
	.md-editor--readonly .md-editor__host :global(.md-editor__doc) {
		min-height: 0;
		padding: 0;
		font-size: var(--md-sys-typescale-body-medium-size);
	}
	.md-editor__host :global(.md-editor__doc > :first-child) {
		margin-top: 0;
	}
	.md-editor__host :global(.md-editor__doc > :last-child) {
		margin-bottom: 0;
	}
	.md-editor__host :global(p) {
		margin: 0 0 6px;
	}
	.md-editor__host :global(h1),
	.md-editor__host :global(h2),
	.md-editor__host :global(h3) {
		margin: 18px 0 6px;
		font-family: var(--md-ref-typeface-brand);
		font-weight: 700;
		line-height: 1.25;
		letter-spacing: -0.01em;
	}
	.md-editor__host :global(h1) {
		font-size: 1.625rem;
	}
	.md-editor__host :global(h2) {
		font-size: 1.3125rem;
	}
	.md-editor__host :global(h3) {
		font-size: 1.0625rem;
	}
	.md-editor__host :global(ul),
	.md-editor__host :global(ol) {
		margin: 0 0 6px;
		padding-left: 1.5em;
	}
	.md-editor__host :global(ul[data-type='taskList']) {
		padding-left: 0.25em;
		list-style: none;
	}
	.md-editor__host :global(ul[data-type='taskList'] li) {
		display: flex;
		align-items: flex-start;
		gap: 8px;
	}
	.md-editor__host :global(ul[data-type='taskList'] li > label) {
		flex: none;
		margin-top: 0.3em;
	}
	.md-editor__host :global(ul[data-type='taskList'] li > div) {
		flex: 1;
		min-width: 0;
	}
	.md-editor__host :global(ul[data-type='taskList'] li[data-checked='true'] > div) {
		color: var(--md-sys-color-on-surface-variant);
		text-decoration: line-through;
	}
	.md-editor__host :global(input[type='checkbox']) {
		width: 16px;
		height: 16px;
		accent-color: var(--md-sys-color-primary);
	}
	.md-editor__host :global(blockquote) {
		margin: 0 0 6px;
		padding: 2px 0 2px 14px;
		border-left: 3px solid var(--md-sys-color-outline-variant);
		color: var(--md-sys-color-on-surface-variant);
	}
	.md-editor__host :global(code) {
		padding: 1px 5px;
		border-radius: 4px;
		background: var(--md-sys-color-surface-container-highest);
		font-family: var(--md-ref-typeface-mono, ui-monospace, monospace);
		font-size: 0.875em;
	}
	.md-editor__host :global(pre) {
		margin: 0 0 6px;
		padding: 12px 14px;
		border-radius: 8px;
		background: var(--md-sys-color-surface-container-highest);
		overflow-x: auto;
	}
	.md-editor__host :global(pre code) {
		padding: 0;
		background: none;
	}
	.md-editor__host :global(hr) {
		margin: 14px 0;
		border: none;
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}
	.md-editor__host :global(a) {
		color: var(--md-sys-color-primary);
	}
	.md-editor__host :global(p.is-editor-empty:first-child::before) {
		content: attr(data-placeholder);
		float: left;
		height: 0;
		color: var(--md-sys-color-on-surface-variant);
		pointer-events: none;
	}

	.md-menu,
	.md-bubble {
		position: fixed;
		z-index: 10;
		border: 1px solid var(--md-sys-color-outline-variant);
		background: var(--md-sys-color-surface-container);
		box-shadow: var(--md-sys-elevation-shadow-level2);
	}
	.md-menu {
		display: flex;
		flex-direction: column;
		width: 220px; /* MENU_WIDTH */
		max-height: 300px; /* MENU_MAX_HEIGHT */
		padding: 4px;
		border-radius: 10px;
		overflow-y: auto;
	}
	.md-menu__item {
		display: flex;
		align-items: center;
		gap: 10px;
		min-height: 40px;
		padding: 0 10px;
		border: none;
		border-radius: 6px;
		background: none;
		color: var(--md-sys-color-on-surface);
		font: inherit;
		font-size: 0.875rem;
		text-align: left;
		cursor: pointer;
	}
	.md-menu__item[aria-selected='true'],
	.md-menu__item:hover {
		background: var(--md-sys-color-surface-container-highest);
	}
	.md-menu__item:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: -2px;
	}
	.md-menu__icon {
		display: inline-flex;
		color: var(--md-sys-color-on-surface-variant);
	}
	.md-bubble {
		display: flex;
		gap: 2px;
		padding: 3px;
		border-radius: 8px;
		transform: translate(-50%, -100%);
	}
	.md-bubble__btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 36px;
		height: 36px;
		border: none;
		border-radius: 6px;
		background: none;
		color: var(--md-sys-color-on-surface);
		cursor: pointer;
	}
	.md-bubble__btn:hover {
		background: var(--md-sys-color-surface-container-highest);
	}
	.md-bubble__btn[aria-pressed='true'] {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.md-bubble__btn:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: -2px;
	}
	@media (pointer: coarse) {
		.md-bubble__btn {
			width: 44px;
			height: 44px;
		}
		.md-menu__item {
			min-height: 44px;
		}
	}
</style>
