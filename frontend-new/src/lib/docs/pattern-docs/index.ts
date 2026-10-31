import type { PatternDoc } from '../types';

const modules = import.meta.glob<PatternDoc[]>(['./*.ts', '!./index.ts', '!./*.test.ts'], { eager: true, import: 'default' });

export const patternDocs: Record<string, PatternDoc> = Object.fromEntries(
	Object.values(modules)
		.flat()
		.map((doc) => [`${doc.sheet}/${doc.id}`, doc]),
);
