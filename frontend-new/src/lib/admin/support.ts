import { m } from '$lib/paraglide/messages';

// A help article as the admin form writes it (support.rs::ArticleInput): a title of 1–200
// characters, its body, tags written with commas, and whether it's published.

export type ArticleBody = { title: string; body: string; tags: string[]; published: boolean };

const text = (form: FormData, name: string): string => {
	const value = form.get(name);
	return typeof value === 'string' ? value.trim() : '';
};

export function articleFromForm(form: FormData, { published }: { published?: boolean } = {}): { ok: true; body: ArticleBody } | { ok: false; message: string } {
	const title = text(form, 'title');
	if (!title || [...title].length > 200) return { ok: false, message: m.admin_article_title_invalid() };
	const tags = text(form, 'tags')
		.split(',')
		.map((tag) => tag.trim())
		.filter(Boolean);
	return { ok: true, body: { title, body: text(form, 'body'), tags, published: published ?? form.get('published') === 'on' } };
}
