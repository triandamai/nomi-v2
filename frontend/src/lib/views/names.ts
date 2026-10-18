import type { ViewName } from './types';

/** Every view this build draws. `registry.ts` must have a component for each (its type says so),
 * and examples.test.ts checks this list against the backend's examples. */
export const VIEW_NAMES = [
	'list',
	'table',
	'comparison',
	'stepper',
	'timeline',
	'card',
	'chart',
	'answer',
	'suggestions',
	'decision',
	'form',
	'link',
	'task',
	'shell',
] as const satisfies readonly ViewName[];

// Fails to compile if a view in `View` is missing from the list above.
type Missing = Exclude<ViewName, (typeof VIEW_NAMES)[number]>;
export const everyViewListed: Missing extends never ? true : Missing = true;
