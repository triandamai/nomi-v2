import { m } from '$lib/paraglide/messages';
import { clock } from '$lib/chat/dates';

// A domain of the person's own (Flow / Build — Workspace tools U16wU, Screen / Domain J2hPJ and
// Domain connected Dd60u): the records to set at the registrar, looked up when it's added, on
// Check now and every 5 minutes. Waiting for DNS until every record is found, then connected, with
// HTTPS on once the hosting issued it. Only a connected domain can be made primary.

/** One of the project's own domains, as `GET /api/projects/:id/domains` gives it. */
export type OwnDomain = {
	hostname: string;
	records: { type: string; name: string; value: string; status: string }[];
	checked_at: string | null;
	verified_at: string | null;
	primary: boolean;
	https: string;
	created_at: string;
};

export type DomainView = {
	state: { label: string; tone: 'live' | 'waiting' };
	records: { type: string; name: string; value: string; found: boolean }[];
	checked: string;
	note: string;
	canMakePrimary: boolean;
};

export function domainView(domain: OwnDomain, timeZone?: string): DomainView {
	const records = domain.records.map((record) => ({ type: record.type, name: record.name, value: record.value, found: record.status === 'found' }));
	const connected = records.length > 0 && records.every((record) => record.found);
	const when = domain.checked_at ? clock(domain.checked_at, timeZone) : null;
	const secured = domain.https === 'issued';
	return {
		state: connected ? { label: secured ? m.domain_connected_https() : m.domain_connected_https_coming(), tone: 'live' } : { label: m.domain_waiting_dns(), tone: 'waiting' },
		records,
		checked: !when ? m.domain_not_checked() : connected ? m.domain_checked_all({ time: when }) : m.domain_checked_every({ time: when }),
		note: !connected ? m.domain_https_after_records() : secured ? m.domain_serves_both() : m.domain_https_coming_note(),
		canMakePrimary: connected && !domain.primary,
	};
}
