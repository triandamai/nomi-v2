import { agentLook, GRADIENT_STOPS, TONE_ACCENT } from '$lib/components/m3/shapes';
import type { HomeSummary } from '$lib/types';

export type OutItem = HomeSummary['while_you_were_out'][number];
export type TodayItem = HomeSummary['today'][number];
export type PlanItem = HomeSummary['plans'][number];

/** Home's three sections, by the path of their own page (/home/<section>). */
export const HOME_SECTIONS = {
	updates: { title: 'While you were out', lede: 'Everything the crew did since you last looked.' },
	today: { title: 'Today', lede: 'Everything scheduled for today.' },
	plans: { title: 'Plans in progress', lede: 'Every plan and to-do list with steps still open.' },
} as const;
export type HomeSection = keyof typeof HOME_SECTIONS;

export function isHomeSection(value: string): value is HomeSection {
	return Object.hasOwn(HOME_SECTIONS, value);
}

/** "14:05" in the user's timezone. */
export function clockTime(iso: string, timeZone: string): string {
	try {
		return new Intl.DateTimeFormat(undefined, { hour: '2-digit', minute: '2-digit', hourCycle: 'h23', timeZone }).format(new Date(iso));
	} catch {
		return new Date(iso).toTimeString().slice(0, 5);
	}
}

export function agentName(agent: string): string {
	if (agent === 'chitchat') return 'Nomi';
	return agent.charAt(0).toUpperCase() + agent.slice(1);
}

export const RECURRENCE: Record<string, string> = { daily: 'Daily', weekly: 'Weekly', monthly: 'Monthly' };

export function agentTint(agent: string): string {
	return GRADIENT_STOPS[agentLook(agent).tone].at(-1) ?? '#5be08f';
}

export function agentAccent(agent: string): string {
	return TONE_ACCENT[agentLook(agent).tone];
}
