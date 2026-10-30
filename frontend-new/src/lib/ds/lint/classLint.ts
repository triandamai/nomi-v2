// Checks a component's classes against the design system's rules (see CLAUDE.md, "Don't").
// It reads markup only: static class attributes, string literals in class expressions and
// class: directives. Script and style blocks are skipped.

const HEX = /#[0-9a-fA-F]{3,8}\b/;
// shadow-float for what floats; shadow-focus is a focused field's glow, not elevation.
const ALLOWED_SHADOWS = new Set(['shadow-float', 'shadow-focus', 'shadow-none']);

function markup(source: string): string {
	return source.replace(/<script[\s\S]*?<\/script>/g, '').replace(/<style[\s\S]*?<\/style>/g, '');
}

/** String literals in an expression, except those it only compares against (`variant === 'ghost'`). */
function literals(expression: string): string[] {
	const found: string[] = [];
	for (const match of expression.matchAll(/'([^']*)'|"([^"]*)"|`([^`$]*)`/g)) {
		const before = expression.slice(0, match.index).trimEnd();
		const after = expression.slice(match.index + match[0].length).trimStart();
		if (/[=!]==?$/.test(before) || /^[=!]==?/.test(after)) continue;
		found.push(match[1] ?? match[2] ?? match[3] ?? '');
	}
	return found;
}

/** Classes the component styles itself, in its own <style> block. */
function scopedClasses(source: string): Set<string> {
	const styles = [...source.matchAll(/<style[^>]*>([\s\S]*?)<\/style>/g)].map((match) => match[1]).join('\n');
	return new Set([...styles.matchAll(/\.(-?[_a-zA-Z][\w-]*)/g)].map((match) => match[1]));
}

/** The class names a component's markup uses. */
export function classNames(source: string): string[] {
	const html = markup(source);
	const names: string[] = [];
	for (const match of html.matchAll(/\bclass\s*=\s*(?:"([^"]*)"|'([^']*)'|\{((?:[^{}]|\{[^{}]*\})*)\})/g)) {
		const quoted = match[1] ?? match[2];
		if (quoted !== undefined) {
			// Static text plus any string literals inside {…} interpolations.
			const expressions = [...quoted.matchAll(/\{([^}]*)\}/g)].flatMap((inner) => literals(inner[1]));
			names.push(...quoted.replace(/\{[^}]*\}/g, ' ').split(/\s+/), ...expressions.flatMap((text) => text.split(/\s+/)));
		} else {
			names.push(...literals(match[3]).flatMap((text) => text.split(/\s+/)));
		}
	}
	for (const match of html.matchAll(/\bclass:([^\s=>/]+)/g)) names.push(match[1]);
	return [...new Set(names.filter((name) => name && !/[{}'"`$]/.test(name)))];
}

/** Everything in `source` that breaks the design system's class rules, as readable lines. */
export function findClassProblems(source: string, isKnown: (name: string) => boolean): string[] {
	const problems: string[] = [];
	const scoped = scopedClasses(source);
	for (const name of classNames(source)) {
		if (scoped.has(name)) continue;
		const utility = name.split(':').pop() ?? name;
		if (HEX.test(name)) problems.push(`raw colour: ${name}`);
		else if (name.startsWith('dark:') || name.includes(':dark:')) problems.push(`dark: variant: ${name}`);
		else if (/^-?(inset-)?shadow-/.test(utility) && !ALLOWED_SHADOWS.has(utility)) problems.push(`shadow on a flat element: ${name}`);
		else if (name.includes('[')) problems.push(`arbitrary value: ${name}`);
		else if (!isKnown(name)) problems.push(`unknown class: ${name}`);
	}
	for (const match of markup(source).matchAll(/\bstyle\s*=\s*(?:"([^"]*)"|'([^']*)')/g)) {
		const hex = (match[1] ?? match[2]).match(HEX);
		if (hex) problems.push(`raw colour in style: ${hex[0]}`);
	}
	return problems;
}
