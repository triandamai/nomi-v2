// The sources a reply cites: the numbered links `[1](https://…)` the crew writes after a claim
// (see the web_search tool's description on the server), listed under the reply by $lib/richReply.

export type Cited = { n: number; url: string; title: string; host: string };

/** A citation is a markdown link whose text is just its number: `[2](https://…)`. */
const CITATION = /\[(\d{1,2})\]\((https?:\/\/[^\s)]+)\)/g;

export function hostOf(url: string): string {
	try {
		return new URL(url).hostname.replace(/^www\./, '');
	} catch {
		return url;
	}
}

/**
 * The sources a reply cites, in citation order, one per number: the title from the pages the crew
 * searched or read (`known`), else the site's name.
 */
export function citedSources(markdown: string, known: { title: string | null; url: string }[] = []): Cited[] {
	const byNumber = new Map<number, string>();
	for (const match of markdown.matchAll(CITATION)) {
		const n = Number(match[1]);
		if (n > 0 && !byNumber.has(n)) byNumber.set(n, match[2]);
	}
	return [...byNumber.entries()]
		.sort(([a], [b]) => a - b)
		.map(([n, url]) => {
			const title = known.find((source) => source.url === url || source.url.replace(/\/$/, '') === url.replace(/\/$/, ''))?.title;
			return { n, url, title: title?.trim() || hostOf(url), host: hostOf(url) };
		});
}
