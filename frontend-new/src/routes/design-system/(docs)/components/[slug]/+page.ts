import { error } from '@sveltejs/kit';
import { api, components } from '$lib/docs/components';
import { componentDocs } from '$lib/docs/entries';
import { examplesFor } from '$lib/docs/examples';
import type { EntryGenerator, PageLoad } from './$types';

export const entries: EntryGenerator = () => components.map((item) => ({ slug: item.slug }));

export const load: PageLoad = async ({ params }) => {
	const item = components.find((candidate) => candidate.slug === params.slug);
	if (!item) error(404, 'No component by that name');
	return { item, api: api[item.file], doc: componentDocs[item.file], examples: examplesFor(item.slug) };
};
