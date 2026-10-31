import { error } from '@sveltejs/kit';
import { api, components } from '$lib/docs/components';
import { PATTERNS } from '$lib/docs/patterns';
import { patternDocs } from '$lib/docs/pattern-docs';
import type { EntryGenerator, PageLoad } from './$types';

export const entries: EntryGenerator = () => PATTERNS.map((pattern) => ({ sheet: pattern.sheet, id: pattern.id }));

export const load: PageLoad = async ({ params }) => {
	const pattern = PATTERNS.find((candidate) => candidate.sheet === params.sheet && candidate.id === params.id);
	if (!pattern) error(404, 'No pattern by that name');
	const used = components.filter((item) => api[item.file].sections.some((section) => section.sheet === pattern.sheet && section.id === pattern.id));
	return { pattern: { sheet: pattern.sheet, title: pattern.title, id: pattern.id }, Section: (await pattern.load()).default, doc: patternDocs[`${pattern.sheet}/${pattern.id}`], used };
};
