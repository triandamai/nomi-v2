// A project's own domains (shop.example.com): what's left to do for one, and how to name it.
import type { CustomDomain } from '$lib/types';

/** How many of the domain's records the last look found, of how many. */
export function foundCount(domain: CustomDomain): { found: number; total: number } {
	return { found: domain.records.filter((r) => r.status === 'found').length, total: domain.records.length };
}

/** Whether it can be made the app's primary address: every record found, and not already. */
export function canMakePrimary(domain: CustomDomain): boolean {
	return !domain.primary && domain.verified_at !== null;
}

/** What's typed, as the backend reads a hostname (no scheme, path or trailing dot). */
export function typedHostname(raw: string): string {
	return raw
		.trim()
		.toLowerCase()
		.replace(/^https?:\/\//, '')
		.split(/[/?#]/)[0]
		.replace(/\.$/, '');
}

/** The record's name as a registrar's form takes it: `@` for the domain itself, else the part
 * before the zone (`_nomi-verify` for `_nomi-verify.example.com` under `example.com`). */
export function recordHost(name: string, hostname: string): string {
	if (name === hostname) return '@';
	return name.endsWith(`.${hostname}`) ? name.slice(0, -(hostname.length + 1)) : name;
}
