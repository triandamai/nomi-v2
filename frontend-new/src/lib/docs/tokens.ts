// The Tokens page's rows: each token in tokens.json (exported from the pen), its CSS variable,
// the Tailwind classes that reach it (read from app.css's @theme), and its value in each theme.

type Token = { type: string; light?: string; dark?: string; value?: string | number };
export type TokenRow = { name: string; cssVar: string; classes: string[]; light?: string; dark?: string; value?: string };
export type TokenGroup = { title: string; rows: TokenRow[] };

const GROUPS: [title: string, test: (name: string) => boolean][] = [
	['Surfaces & text', (name) => /^nm-(bg|surface|border|text|overlay|shadow|focus)/.test(name)],
	['Accent & signals', (name) => /^nm-(accent|danger|warning)/.test(name)],
	['Crew', (name) => name.startsWith('crew-')],
	['Code & diff', (name) => /^nm-(code|diff)-/.test(name)],
	['Peers', (name) => name.startsWith('nm-peer-')],
	['Heat map & keyboard', (name) => /^nm-(heat|kb)-/.test(name)],
	['Type', (name) => /^nm-(sans|mono|size-)/.test(name)],
	['Space', (name) => name.startsWith('nm-space-')],
	['Radius', (name) => name.startsWith('nm-radius-')],
];

export function tokenGroups(tokens: Record<string, Token>, css: string): TokenGroup[] {
	const mapped = new Map<string, string[]>();
	for (const [, kind, key, token] of css.matchAll(/--(color|font|text|radius)-([a-z0-9-]+):\s*var\(--((?:nm|crew)-[a-z0-9-]+)\)/g)) {
		const classes = kind === 'color' ? [`bg-${key}`, `text-${key}`, `border-${key}`] : [`${{ font: 'font', text: 'text', radius: 'rounded' }[kind]}-${key}`];
		mapped.set(token, classes);
	}
	const row = (name: string, token: Token): TokenRow => {
		const classes = mapped.get(name) ?? (name.startsWith('nm-space-') ? [`p-${Number(token.value) / 4}`, `gap-${Number(token.value) / 4}`] : []);
		if (token.type === 'color') return { name, cssVar: `--${name}`, classes, light: token.light, dark: token.dark };
		return { name, cssVar: `--${name}`, classes, value: token.type === 'number' ? `${token.value}px` : String(token.value) };
	};
	return GROUPS.map(([title, test]) => ({
		title,
		rows: Object.entries(tokens)
			.filter(([name]) => test(name))
			.map(([name, token]) => row(name, token)),
	})).filter((group) => group.rows.length);
}
