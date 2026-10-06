import { agentLook, GRADIENT_STOPS, TONE_ACCENT } from '$lib/components/m3/shapes';
import type { HomeSummary } from '$lib/types';
import { getLocale } from '$lib/paraglide/runtime';
import { m } from '$lib/paraglide/messages';

export type OutItem = HomeSummary['while_you_were_out'][number];
export type TodayItem = HomeSummary['today'][number];
export type PlanItem = HomeSummary['plans'][number];

/** Home's three sections, by the path of their own page (/home/<section>). */
const HOME_SECTIONS = {
	updates: () => ({ title: m.home_out_title(), lede: m.home_out_lede() }),
	today: () => ({ title: m.home_today_title(), lede: m.home_today_lede() }),
	plans: () => ({ title: m.home_plans_title(), lede: m.home_plans_lede() }),
};
export type HomeSection = keyof typeof HOME_SECTIONS;

export function isHomeSection(value: string): value is HomeSection {
	return Object.hasOwn(HOME_SECTIONS, value);
}

/** A section's title and lede, in the reader's language. */
export function homeSectionCopy(section: HomeSection): { title: string; lede: string } {
	return HOME_SECTIONS[section]();
}

/** "14:05" in the user's timezone. */
export function clockTime(iso: string, timeZone: string): string {
	try {
		return new Intl.DateTimeFormat(getLocale(), { hour: '2-digit', minute: '2-digit', hourCycle: 'h23', timeZone }).format(new Date(iso));
	} catch {
		return new Date(iso).toTimeString().slice(0, 5);
	}
}

export function agentName(agent: string): string {
	if (agent === 'chitchat') return 'Nomi';
	return agent.charAt(0).toUpperCase() + agent.slice(1);
}

export function recurrenceLabel(recurrence: string): string {
	const labels: Record<string, () => string> = { daily: m.rem_daily, weekly: m.rem_weekly, monthly: m.rem_monthly };
	return labels[recurrence]?.() ?? recurrence;
}

export function agentTint(agent: string): string {
	return GRADIENT_STOPS[agentLook(agent).tone].at(-1) ?? '#5be08f';
}

export function agentAccent(agent: string): string {
	return TONE_ACCENT[agentLook(agent).tone];
}
