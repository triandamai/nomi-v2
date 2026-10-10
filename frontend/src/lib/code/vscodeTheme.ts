// A read-only CodeMirror look modelled on VS Code's "Dark Modern" theme and its Dark+ token
// colours, used for the file cards Koda posts in chat (and anywhere else code is shown).

import { EditorView } from '@codemirror/view';
import { HighlightStyle, syntaxHighlighting } from '@codemirror/language';
import { tags as t } from '@lezer/highlight';

export const VSCODE = {
	editor: '#1f1f1f',
	tabBar: '#181818',
	border: '#2b2b2b',
	text: '#cccccc',
	muted: '#9d9d9d',
	lineNumber: '#6e7681',
	activeLineNumber: '#cccccc',
	selection: '#264f78',
	focus: '#0078d4',
	inserted: '#2ea0431f',
	insertedText: '#2ea04340',
	insertedGutter: '#2ea043',
	removed: '#f851491f',
	removedText: '#f8514940',
	removedGutter: '#f85149'
} as const;

const theme = EditorView.theme(
	{
		'&': { backgroundColor: VSCODE.editor, color: '#d4d4d4', fontSize: '13px' },
		'.cm-scroller': {
			fontFamily: "'Cascadia Code', 'JetBrains Mono', Menlo, Consolas, 'Courier New', monospace",
			lineHeight: '19px'
		},
		'.cm-content': { padding: '6px 0', caretColor: '#aeafad' },
		'.cm-cursor, .cm-dropCursor': { borderLeftColor: '#aeafad', borderLeftWidth: '2px' },
		'.cm-activeLine': { backgroundColor: 'transparent', boxShadow: 'inset 0 0 0 1px #282828' },
		'.cm-activeLineGutter': { backgroundColor: 'transparent', color: VSCODE.activeLineNumber },
		'.cm-foldGutter .cm-gutterElement': { color: '#c5c5c5', padding: '0 4px', cursor: 'pointer' },
		'.cm-matchingBracket, &.cm-focused .cm-matchingBracket': { backgroundColor: '#0064001a', outline: '1px solid #888888' },
		'.cm-selectionMatch': { backgroundColor: '#3a3d41' },
		'.cm-foldPlaceholder': { backgroundColor: '#3a3d41', border: 'none', color: '#cccccc' },
		'.cm-panels': { backgroundColor: VSCODE.tabBar, color: VSCODE.text },
		'.cm-panels.cm-panels-top': { borderBottom: `1px solid ${VSCODE.border}` },
		'.cm-searchMatch': { backgroundColor: '#623315', outline: '1px solid #9e6a03' },
		'.cm-searchMatch.cm-searchMatch-selected': { backgroundColor: '#9e6a03' },
		'.cm-line': { padding: '0 16px 0 8px' },
		'.cm-gutters': { backgroundColor: VSCODE.editor, color: VSCODE.lineNumber, border: 'none' },
		'.cm-lineNumbers .cm-gutterElement': { padding: '0 4px 0 16px', minWidth: '40px' },
		'&.cm-focused': { outline: 'none' },
		'.cm-selectionBackground, &.cm-focused .cm-selectionBackground, ::selection': { backgroundColor: `${VSCODE.selection} !important` },
		// Diffs, as VS Code's inline diff editor shows them.
		'.cm-insertedLine, .cm-changedLine': { backgroundColor: VSCODE.inserted },
		'.cm-changedText': { background: `${VSCODE.insertedText} !important` },
		'.cm-insertedLine .cm-changedText': { background: `${VSCODE.insertedText} !important` },
		'.cm-deletedChunk': { backgroundColor: VSCODE.removed, paddingLeft: '8px' },
		'.cm-deletedChunk .cm-deletedLine': { backgroundColor: 'transparent' },
		'.cm-deletedChunk .cm-changedText, .cm-deletedText': { background: `${VSCODE.removedText} !important` },
		'.cm-deletedChunk del': { textDecoration: 'none' },
		'.cm-changeGutter': { width: '3px', paddingLeft: '0' },
		'.cm-changedLineGutter, .cm-insertedLineGutter': { background: VSCODE.insertedGutter },
		'.cm-deletedLineGutter': { background: VSCODE.removedGutter },
		'.cm-collapsedLines': {
			background: VSCODE.tabBar,
			color: VSCODE.muted,
			fontFamily: 'inherit',
			fontSize: '12px',
			padding: '4px 16px',
			borderBlock: `1px solid ${VSCODE.border}`,
			cursor: 'pointer'
		},
		'.cm-collapsedLines:before, .cm-collapsedLines:after': { display: 'none' }
	},
	{ dark: true }
);

/** VS Code Dark+ token colours. */
const highlight = HighlightStyle.define([
	{ tag: [t.keyword, t.controlKeyword, t.moduleKeyword, t.operatorKeyword], color: '#c586c0' },
	{ tag: [t.definitionKeyword, t.modifier, t.bool, t.null, t.self, t.atom], color: '#569cd6' },
	{ tag: [t.typeName, t.className, t.namespace, t.standard(t.typeName)], color: '#4ec9b0' },
	{ tag: [t.function(t.variableName), t.function(t.propertyName), t.macroName], color: '#dcdcaa' },
	{ tag: [t.variableName, t.propertyName, t.attributeName, t.labelName], color: '#9cdcfe' },
	{ tag: [t.definition(t.variableName), t.local(t.variableName)], color: '#9cdcfe' },
	{ tag: [t.constant(t.variableName), t.standard(t.variableName)], color: '#4fc1ff' },
	{ tag: [t.string, t.special(t.string), t.character, t.attributeValue], color: '#ce9178' },
	{ tag: [t.regexp, t.escape], color: '#d16969' },
	{ tag: [t.number, t.unit], color: '#b5cea8' },
	{ tag: [t.comment, t.lineComment, t.blockComment, t.docComment], color: '#6a9955' },
	{ tag: [t.tagName, t.angleBracket], color: '#569cd6' },
	{ tag: [t.heading, t.strong], color: '#569cd6', fontWeight: 'bold' },
	{ tag: t.emphasis, fontStyle: 'italic' },
	{ tag: t.link, color: '#4fc1ff', textDecoration: 'underline' },
	{ tag: [t.operator, t.punctuation, t.separator, t.bracket], color: '#d4d4d4' },
	{ tag: t.meta, color: '#9cdcfe' },
	{ tag: t.invalid, color: '#f44747' }
]);

export const vscodeDark = [theme, syntaxHighlighting(highlight)];
