// A reply's text as the design system draws it (Chat responses → Rich content): the markdown is
// rendered (and sanitised) on the server, then the parts the design system has its own pieces for
// are swapped in here, on the real DOM:
// - fenced code → CodeBlock (numbered, nm-code-* colours, copy),
// - `[1](https://…)` → Citation, pointing at the reply's Sources list,
// - other links → Hyperlink,
// - "@Dana" → MentionCrew, in that member's tint.
// Only text and attributes are read from the message; nothing from it is ever set as HTML.
import { mount, unmount } from 'svelte';
import CodeBlock from '$lib/components/ds/chat/CodeBlock.svelte';
import Citation from '$lib/components/ds/chat/Citation.svelte';
import Hyperlink from '$lib/components/ds/chat/Hyperlink.svelte';
import MentionCrew from '$lib/components/ds/chat/MentionCrew.svelte';
import MentionFile from '$lib/components/ds/chat/MentionFile.svelte';
import { File, FileCode, FileImage, FileSpreadsheet, FileText } from '@lucide/svelte';
import { CREW, type CrewId } from '$lib/components/ds/agents/crew';
import { highlight } from '$lib/codeTokens';
import { hostOf, type Cited } from '$lib/citations';

export { citedSources, type Cited } from '$lib/citations';

const CREW_BY_NAME = new Map<string, CrewId>(
	CREW.filter((member) => member.id !== 'personality' && member.id !== 'supervisor').map((member) => [member.name.toLowerCase(), member.id]),
);
const MENTION_SOURCE = `@(${[...CREW_BY_NAME.keys()].join('|')})\\b`;
const MENTION = new RegExp(MENTION_SOURCE, 'gi');
const HAS_MENTION = new RegExp(MENTION_SOURCE, 'i');

/** A file's name written as inline code (`rent-2026.xlsx`), not a path or a sentence. */
const FILE_NAME = /^[\p{L}\p{N}_. -]{1,80}\.([A-Za-z0-9]{1,5})$/u;
const FILE_ICONS: Record<string, typeof File> = {
	xlsx: FileSpreadsheet,
	xls: FileSpreadsheet,
	csv: FileSpreadsheet,
	ods: FileSpreadsheet,
	numbers: FileSpreadsheet,
	pdf: FileText,
	doc: FileText,
	docx: FileText,
	txt: FileText,
	md: FileText,
	rtf: FileText,
	odt: FileText,
	png: FileImage,
	jpg: FileImage,
	jpeg: FileImage,
	gif: FileImage,
	webp: FileImage,
	svg: FileImage,
	heic: FileImage,
	ts: FileCode,
	js: FileCode,
	svelte: FileCode,
	py: FileCode,
	rs: FileCode,
	json: FileCode,
	html: FileCode,
	css: FileCode,
	sql: FileCode,
	go: FileCode,
};
/** Extensions that aren't files when they follow a dot in code (`user.name`, `Math.round`). */
const KNOWN = new Set(Object.keys(FILE_ICONS));

export function fileMention(text: string): { name: string; icon: typeof File } | null {
	const match = FILE_NAME.exec(text.trim());
	if (!match) return null;
	const ext = match[1].toLowerCase();
	if (!KNOWN.has(ext)) return null;
	return { name: text.trim(), icon: FILE_ICONS[ext] ?? File };
}

type Mounted = ReturnType<typeof mount>;

/** Puts a mount point where `node` was. */
function slot(node: Node, inline = true): HTMLElement {
	const target = document.createElement(inline ? 'span' : 'div');
	target.className = inline ? 'rich-inline' : 'rich-block';
	node.parentNode?.replaceChild(target, node);
	return target;
}

/**
 * Swaps the design system's pieces into a reply rendered from markdown. `sourcesId` is the id of
 * the reply's Sources list (citations link to its items). Returns what undoes it (the components
 * are unmounted; the DOM itself is Svelte's {@html} and is replaced with the message).
 */
export function enrichReply(root: HTMLElement, cited: Cited[], sourcesId: string): () => void {
	const mounted: Mounted[] = [];

	// Fenced code (the server marks each block's <pre> with its language).
	for (const pre of root.querySelectorAll<HTMLElement>('pre[data-lang]')) {
		const language = pre.dataset.lang && pre.dataset.lang !== 'text' ? pre.dataset.lang : undefined;
		const code = (pre.textContent ?? '').replace(/\n+$/, '');
		mounted.push(
			mount(CodeBlock, {
				target: slot(pre, false),
				props: { code, language, tokens: highlight(code, language) },
			}),
		);
	}

	// Links: citations and hyperlinks. Only links that are plain text are swapped; a link around
	// bold or code keeps its own markup.
	for (const link of root.querySelectorAll<HTMLAnchorElement>('a[href]')) {
		if (link.closest('.rich-inline, .rich-block')) continue;
		if (link.children.length > 0) continue;
		const href = link.getAttribute('href') ?? '';
		const label = (link.textContent ?? '').trim();
		if (!label) continue;
		const external = /^https?:\/\//i.test(href);
		if (/^\d{1,2}$/.test(label)) {
			const n = Number(label);
			const source = cited.find((item) => item.n === n);
			mounted.push(
				mount(Citation, {
					target: slot(link),
					props: {
						n,
						title: source?.title ?? hostOf(href),
						href: source ? `#${sourcesId}-${n}` : href,
					},
				}),
			);
		} else {
			mounted.push(
				mount(Hyperlink, {
					target: slot(link),
					props: { href, label, external },
				}),
			);
		}
	}

	// A file named in inline code, as a file mention (Chat responses → Mention / File).
	for (const code of root.querySelectorAll<HTMLElement>('code')) {
		if (code.closest('pre, a, .rich-inline, .rich-block') || code.children.length > 0) continue;
		const file = fileMention(code.textContent ?? '');
		if (file) mounted.push(mount(MentionFile, { target: slot(code), props: file }));
	}

	// @mentions of the crew, in prose only (not in code or links).
	const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, {
		acceptNode: (node) =>
			node.parentElement?.closest('pre, code, a, .rich-inline, .rich-block') || !HAS_MENTION.test(node.nodeValue ?? '')
				? NodeFilter.FILTER_REJECT
				: NodeFilter.FILTER_ACCEPT,
	});
	const texts: Text[] = [];
	for (let node = walker.nextNode(); node; node = walker.nextNode()) texts.push(node as Text);
	for (const text of texts) {
		const value = text.nodeValue ?? '';
		const parts = document.createDocumentFragment();
		const targets: { at: HTMLElement; agent: CrewId; name: string }[] = [];
		let last = 0;
		for (const match of value.matchAll(MENTION)) {
			const agent = CREW_BY_NAME.get(match[1].toLowerCase());
			if (!agent) continue;
			parts.append(value.slice(last, match.index));
			const at = document.createElement('span');
			at.className = 'rich-inline';
			parts.append(at);
			targets.push({
				at,
				agent,
				name: CREW.find((member) => member.id === agent)?.name ?? match[1],
			});
			last = (match.index ?? 0) + match[0].length;
		}
		if (targets.length === 0) continue;
		parts.append(value.slice(last));
		text.replaceWith(parts);
		for (const { at, agent, name } of targets) mounted.push(mount(MentionCrew, { target: at, props: { agent, name } }));
	}

	return () => {
		for (const instance of mounted) void unmount(instance);
	};
}
