// One way to turn a title or a component name into a URL part, shared by the docs' pages and the
// sheets' section anchors: "Code, terminal & phone" → code-terminal-phone, KVRow → kv-row.
export const slugify = (text: string) =>
	text
		.replace(/([a-z0-9])([A-Z])/g, '$1-$2')
		.replace(/([A-Z]+)([A-Z][a-z])/g, '$1-$2')
		.toLowerCase()
		.replace(/[^a-z0-9]+/g, '-')
		.replace(/^-|-$/g, '');
