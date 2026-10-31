import type { Component } from 'svelte';
import { slugify } from './slug';

// The Chat responses and Build sheets, section by section, are the docs' patterns: how components
// combine for one job. Each page shows the section itself, exactly as the sheet draws it.

export type Pattern = { sheet: 'chat' | 'build'; title: string; id: string; load: () => Promise<{ default: Component }> };

const define = (sheet: Pattern['sheet'], title: string, load: Pattern['load']): Pattern => ({ sheet, title, id: slugify(title), load });

export const SHEETS = { chat: 'Chat responses', build: 'Build' } as const;

export const PATTERNS: Pattern[] = [
	define('chat', 'Thinking & planning', () => import('$lib/gallery/chat/ThinkingSection.svelte')),
	define('chat', 'Rich content', () => import('$lib/gallery/chat/RichSection.svelte')),
	define('chat', 'Actions', () => import('$lib/gallery/chat/ActionsSection.svelte')),
	define('chat', 'Response feedback', () => import('$lib/gallery/chat/FeedbackSection.svelte')),
	define('chat', 'States & errors', () => import('$lib/gallery/chat/StatesSection.svelte')),
	define('chat', 'Message controls & composer', () => import('$lib/gallery/chat/ControlsSection.svelte')),
	define('chat', 'Chat management & long threads', () => import('$lib/gallery/chat/ThreadsSection.svelte')),
	define('chat', 'Crew behaviour', () => import('$lib/gallery/chat/CrewSection.svelte')),
	define('chat', 'Mobile chat', () => import('$lib/gallery/chat/MobileSection.svelte')),
	define('build', 'Build estimate', () => import('$lib/gallery/build/EstimateSection.svelte')),
	define('build', 'Building', () => import('$lib/gallery/build/BuildingSection.svelte')),
	define('build', 'Share & export', () => import('$lib/gallery/build/ShareSection.svelte')),
	define('build', 'First run', () => import('$lib/gallery/build/FirstRunSection.svelte')),
	define('build', 'Workspace', () => import('$lib/gallery/build/WorkspaceSection.svelte')),
	define('build', 'Code, terminal & phone', () => import('$lib/gallery/build/CodeSection.svelte')),
	define('build', 'Build together', () => import('$lib/gallery/build/TogetherSection.svelte')),
	define('build', 'Build on mobile', () => import('$lib/gallery/build/MobileBuildSection.svelte')),
	define('build', 'Story sheet', () => import('$lib/gallery/build/StorySection.svelte')),
	define('build', 'Build types', () => import('$lib/gallery/build/TypesSection.svelte')),
];
