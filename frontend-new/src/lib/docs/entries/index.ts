import type { ComponentDoc } from '../types';

// Every file next to this one exports the docs of one category's components.
const modules = import.meta.glob<ComponentDoc[]>(['./*.ts', '!./index.ts', '!./*.test.ts'], { eager: true, import: 'default' });

export const componentDocs: Record<string, ComponentDoc> = Object.fromEntries(
	Object.values(modules)
		.flat()
		.map((doc) => [doc.file, doc]),
);
