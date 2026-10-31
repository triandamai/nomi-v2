import ts from 'typescript';

// A component's API as its source declares it, for the docs' props tables. Runs at build time
// (the ds-api Vite plugin), so the docs never drift from the code: the props come from the type
// given to $props(), the notes from the comments written above it.

export type PropDoc = { name: string; type: string; required: boolean; default?: string; bindable: boolean; doc?: string };
export type ComponentApi = {
	/** The comments between the imports and $props(), one paragraph per comment. */
	notes: string;
	props: PropDoc[];
	/** Passes other props on to its element (`...rest`). */
	rest: boolean;
	/** Attribute families it accepts through index signatures, e.g. `data-*`. */
	attributes: string[];
	/** The component's own types that its props mention (`Option` → `{ value: string; … }`). */
	types: Record<string, string>;
};

const instanceScript = (source: string) =>
	[...source.matchAll(/<script\b([^>]*)>([\s\S]*?)<\/script>/g)].find(([, attributes]) => !/\bmodule\b|context=["']module["']/.test(attributes))?.[2] ?? '';

const oneLine = (text: string) => text.replace(/\s+/g, ' ').replace(/^\| /, '').trim();

function commentText(code: string, ranges: ts.CommentRange[] | undefined): string[] {
	const paragraphs: string[] = [];
	let previousEnd = -1;
	for (const range of ranges ?? []) {
		const raw = code.slice(range.pos, range.end);
		const text = raw.startsWith('//')
			? raw.replace(/^\/\/\s?/, '')
			: raw
					.replace(/^\/\*\*?/, '')
					.replace(/\*\/$/, '')
					.split('\n')
					.map((line) => line.replace(/^\s*\*\s?/, ''))
					.join(' ');
		// Consecutive // lines are one comment; a blank line starts the next paragraph.
		const joined = previousEnd >= 0 && raw.startsWith('//') && !/\n\s*\n/.test(code.slice(previousEnd, range.pos));
		if (joined) paragraphs[paragraphs.length - 1] += ` ${text}`;
		else paragraphs.push(text);
		previousEnd = range.end;
	}
	return paragraphs.map(oneLine).filter(Boolean);
}

function typeMembers(node: ts.TypeNode | undefined, file: ts.SourceFile): ts.TypeElement[] {
	if (!node) return [];
	if (ts.isTypeLiteralNode(node)) return [...node.members];
	if (ts.isIntersectionTypeNode(node)) return node.types.flatMap((type) => typeMembers(type, file));
	if (ts.isParenthesizedTypeNode(node)) return typeMembers(node.type, file);
	if (ts.isTypeReferenceNode(node) && ts.isIdentifier(node.typeName)) {
		const name = node.typeName.text;
		for (const statement of file.statements) {
			if (ts.isTypeAliasDeclaration(statement) && statement.name.text === name) return typeMembers(statement.type, file);
			if (ts.isInterfaceDeclaration(statement) && statement.name.text === name) {
				const inherited = (statement.heritageClauses ?? []).flatMap((clause) =>
					clause.types.flatMap((type) => (ts.isIdentifier(type.expression) ? typeMembers(ts.factory.createTypeReferenceNode(type.expression.text), file) : [])),
				);
				return [...inherited, ...statement.members];
			}
		}
	}
	return [];
}

const isPropsCall = (node: ts.Expression | undefined) => !!node && ts.isCallExpression(node) && ts.isIdentifier(node.expression) && node.expression.text === '$props';

export function extractApi(source: string): ComponentApi {
	const code = instanceScript(source);
	const file = ts.createSourceFile('component.ts', code, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS);
	const statements = [...file.statements];
	const propsAt = statements.findIndex(
		(statement) => ts.isVariableStatement(statement) && statement.declarationList.declarations.some((declaration) => isPropsCall(declaration.initializer)),
	);
	const firstAfterImports = statements.findIndex((statement) => !ts.isImportDeclaration(statement));

	// Without $props() the notes are whatever the script says (or a trailing comment, if that's all).
	const noteStatements = propsAt >= 0 ? statements.slice(firstAfterImports, propsAt + 1) : firstAfterImports >= 0 ? statements.slice(firstAfterImports) : [];
	const notes = noteStatements.flatMap((statement) => commentText(code, ts.getLeadingCommentRanges(code, statement.getFullStart())));
	if (!statements.length || firstAfterImports < 0) notes.push(...commentText(code, ts.getLeadingCommentRanges(code, file.endOfFileToken.getFullStart())));

	if (propsAt < 0) return { notes: notes.join('\n\n'), props: [], rest: false, attributes: [], types: {} };

	const declaration = (statements[propsAt] as ts.VariableStatement).declarationList.declarations.find((item) => isPropsCall(item.initializer))!;
	const members = typeMembers(declaration.type, file);
	const pattern = ts.isObjectBindingPattern(declaration.name) ? declaration.name : undefined;

	const bindings = new Map<string, { default?: string; bindable: boolean }>();
	let rest = false;
	for (const element of pattern?.elements ?? []) {
		if (element.dotDotDotToken) {
			rest = true;
			continue;
		}
		const name = (element.propertyName ?? element.name).getText(file);
		const initializer = element.initializer;
		const bindable = !!initializer && ts.isCallExpression(initializer) && ts.isIdentifier(initializer.expression) && initializer.expression.text === '$bindable';
		const fallback = bindable ? (initializer as ts.CallExpression).arguments[0] : initializer;
		bindings.set(name, { default: fallback ? oneLine(fallback.getText(file)) : undefined, bindable });
	}

	const attributes: string[] = [];
	const props: PropDoc[] = [];
	for (const member of members) {
		if (ts.isIndexSignatureDeclaration(member)) {
			const key = member.parameters[0]?.type?.getText(file) ?? '';
			const family = key.match(/^`([a-z]+-)\$\{string\}`$/);
			if (family) attributes.push(`${family[1]}*`);
			continue;
		}
		if (!ts.isPropertySignature(member) && !ts.isMethodSignature(member)) continue;
		const name = member.name.getText(file).replace(/^['"]|['"]$/g, '');
		const binding = bindings.get(name);
		const type = ts.isPropertySignature(member) ? oneLine(member.type?.getText(file) ?? 'unknown') : oneLine(member.getText(file).replace(/^[^(]*/, '').replace(/;$/, ''));
		const doc = commentText(code, ts.getLeadingCommentRanges(code, member.getFullStart())).join(' ');
		props.push({
			name,
			type,
			required: !member.questionToken,
			...(binding?.default !== undefined && { default: binding.default }),
			bindable: binding?.bindable ?? false,
			...(doc && { doc }),
		});
	}
	const types: Record<string, string> = {};
	const mentioned = props.map((prop) => prop.type).join(' ');
	for (const statement of statements) {
		const named = ts.isTypeAliasDeclaration(statement) || ts.isInterfaceDeclaration(statement) ? statement : undefined;
		if (!named || named === propsType(declaration.type, file) || !new RegExp(`\\b${named.name.text}\\b`).test(mentioned)) continue;
		types[named.name.text] = oneLine(ts.isTypeAliasDeclaration(named) ? named.type.getText(file) : `{ ${named.members.map((member) => member.getText(file)).join(' ')} }`);
	}
	return { notes: notes.join('\n\n'), props, rest, attributes, types };
}

/** The declaration behind `: Props`, so it isn't listed as one of the types. */
function propsType(node: ts.TypeNode | undefined, file: ts.SourceFile) {
	if (!node || !ts.isTypeReferenceNode(node) || !ts.isIdentifier(node.typeName)) return undefined;
	const name = node.typeName.text;
	return file.statements.find((statement) => (ts.isTypeAliasDeclaration(statement) || ts.isInterfaceDeclaration(statement)) && statement.name.text === name);
}
