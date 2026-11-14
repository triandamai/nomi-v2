import type { Plan } from '$lib/types';

export { MB_PER_GB, formatStorage } from '$lib/storage';

export const SANDBOX_SIZES = ['small', 'default', 'large'] as const;
export type SandboxSize = (typeof SANDBOX_SIZES)[number];

export type PlanLimits = Pick<
	Plan,
	'max_projects' | 'max_orgs' | 'max_org_members' | 'max_custom_crew' | 'max_storage_mb' | 'custom_domain' | 'sandbox_size' | 'all_models' | 'model_ids'
>;

/** A limit typed in: empty (or not a whole number of zero or more) means unlimited. */
export function parseLimit(raw: FormDataEntryValue | null): number | null {
	if (typeof raw !== 'string' || raw.trim() === '') return null;
	const value = Number(raw);
	return Number.isInteger(value) && value >= 0 ? value : null;
}

function parseSandboxSize(raw: FormDataEntryValue | null): SandboxSize | null {
	return (SANDBOX_SIZES as readonly string[]).includes(raw as string) ? (raw as SandboxSize) : null;
}

/** The limits half of the plan form, as the backend takes them. */
export function readPlanLimits(data: FormData): PlanLimits {
	return {
		max_projects: parseLimit(data.get('max_projects')),
		max_orgs: parseLimit(data.get('max_orgs')),
		max_org_members: parseLimit(data.get('max_org_members')),
		max_custom_crew: parseLimit(data.get('max_custom_crew')),
		max_storage_mb: parseLimit(data.get('max_storage_mb')),
		custom_domain: data.get('custom_domain') === 'true',
		sandbox_size: parseSandboxSize(data.get('sandbox_size')),
		all_models: data.get('all_models') === 'true',
		model_ids: data.getAll('model_ids').filter((id): id is string => typeof id === 'string' && id.length > 0),
	};
}

/** A price typed in: a whole number of zero or more, or null when empty or not one. */
export function parseAmount(raw: string): number | null {
	const text = raw.trim();
	if (text === '') return null;
	const value = Number(text);
	return Number.isInteger(value) && value >= 0 ? value : null;
}
