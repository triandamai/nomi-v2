import { m } from '$lib/paraglide/messages';

// What the configuration page calls the backend's sections and fields (nomi_settings::config:
// SECTIONS), in the person's language. One the backend adds later shows by its own name rather
// than not at all.

const messages = m as unknown as Record<string, (() => string) | undefined>;
const humanize = (name: string) => name.charAt(0).toUpperCase() + name.slice(1).replace(/_/g, ' ');

export function sectionLabel(section: string): { title: string; description: string | undefined } {
	return { title: messages[`admin_cfg_${section}`]?.() ?? humanize(section), description: messages[`admin_cfg_${section}_desc`]?.() };
}

export const fieldLabel = (section: string, field: string): string => messages[`admin_cfg_${section}_${field}`]?.() ?? humanize(field);

/** The sections that can be tested: storage with what's typed, Xendit with what's saved. */
export const TESTABLE = ['storage', 'xendit'];
