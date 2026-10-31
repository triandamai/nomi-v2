import { slugify } from './slug';

// The components the docs list: every folder of src/lib/ds is a category, every component in it a
// page. Fixtures (test and gallery scaffolding) are left out before this is called.

export const CATEGORIES = [
	{ key: 'actions', title: 'Actions', blurb: 'Buttons, links and keys: how people start things.' },
	{ key: 'inputs', title: 'Inputs & selection', blurb: 'Fields, choices, tabs and the composer.' },
	{ key: 'navigation', title: 'Navigation', blurb: 'Sidebar rows, recents and group labels.' },
	{ key: 'identity', title: 'Identity', blurb: 'Who is speaking or working: avatars, bylines, the crew.' },
	{ key: 'status', title: 'Status', blurb: 'Verdicts, meters and section labels.' },
	{ key: 'lists', title: 'Lists', blurb: 'Project and suggestion rows, and what a list shows when empty or loading.' },
	{ key: 'conversation', title: 'Conversation & build', blurb: 'Messages, and what agents do in a thread.' },
	{ key: 'overlays', title: 'Overlays', blurb: 'Menus, dialogs, toasts and tooltips: the only things that float.' },
	{ key: 'shells', title: 'Layout shells', blurb: 'The frames every screen sits in.' },
	{ key: 'chrome', title: 'App chrome', blurb: 'The sidebar, the settings nav and what sits in them.' },
	{ key: 'agents', title: 'Agents', blurb: "The crew's mascots." },
	{ key: 'foundations', title: 'Foundations', blurb: 'Icons and the helpers everything else uses.' },
	{ key: 'chat', title: 'Chat responses', blurb: 'Everything a reply can hold, and the controls around it.' },
	{ key: 'build', title: 'Build', blurb: 'From describing an app to a live URL.' },
	{ key: 'mobile', title: 'Mobile', blurb: 'The phone versions, drawn at 390 pt.' },
] as const;

export type CategoryKey = (typeof CATEGORIES)[number]['key'];
export type CatalogItem = { file: string; name: string; slug: string; category: CategoryKey };

export function catalog(files: string[]): CatalogItem[] {
	const order = new Map<string, number>(CATEGORIES.map((category, index) => [category.key, index]));
	const items = files.map((file) => {
		const [folder, base] = file.split('/');
		if (!order.has(folder)) throw new Error(`No docs category for ${folder} (${file}): add it to CATEGORIES`);
		return { file, category: folder as CategoryKey, name: base.replace(/\.svelte$/, '') };
	});
	const named = new Map<string, number>();
	for (const item of items) named.set(item.name, (named.get(item.name) ?? 0) + 1);
	return items
		.map((item) => ({ ...item, slug: named.get(item.name)! > 1 ? `${item.category}-${slugify(item.name)}` : slugify(item.name) }))
		.sort((a, b) => order.get(a.category)! - order.get(b.category)! || a.name.localeCompare(b.name));
}
