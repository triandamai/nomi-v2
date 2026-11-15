import { m } from '$lib/paraglide/messages';
import type { DeviceItem, QuietHours } from '$lib/adapters/api';
import { ago } from '$lib/adapters/time';

// Settings › General (App / Settings / General m7lvY1): the email shown half-hidden, the person's
// devices in a line ("MacBook Air (this one) · iPhone 15 · 2h"), and the time zones to pick from.

/** "ko••••••••@gmail.com": the first two letters, dots for the rest (3 to 8), then the domain. */
export function maskEmail(email: string): string {
	const [local, domain] = email.split('@');
	return `${local.slice(0, 2)}${'•'.repeat(Math.min(8, Math.max(3, local.length - 2)))}@${domain ?? ''}`;
}

/** What a device calls itself: its name, else its brand and model, else its system, else its platform. */
export function deviceName(device: DeviceItem): string {
	return device.name || [device.brand, device.model].filter(Boolean).join(' ') || device.os_name || device.platform;
}

/** This device first, marked, then the others with how long ago each was last seen. */
export function devicesLine(devices: DeviceItem[], now: Date, timeZone?: string): string {
	const sorted = [...devices].sort((a, b) => Number(b.current) - Number(a.current) || Date.parse(b.last_seen_at) - Date.parse(a.last_seen_at));
	return sorted.map((device) => (device.current ? m.general_this_device({ name: deviceName(device) }) : `${deviceName(device)} · ${ago(device.last_seen_at, now, timeZone)}`)).join(' · ');
}

/** Every zone this runtime knows, each "Asia/Jakarta · WIB" (the short name as it is today). */
export function zoneOptions(now: Date): { value: string; label: string }[] {
	return Intl.supportedValuesOf('timeZone').map((zone) => {
		const short = new Intl.DateTimeFormat('en-US', { timeZone: zone, timeZoneName: 'short' }).formatToParts(now).find((part) => part.type === 'timeZoneName')?.value;
		return { value: zone, label: short ? `${zone} · ${short}` : zone };
	});
}

/** The nights quiet hours can cover: the pen's "22:00–06:00" and its neighbours. */
const QUIET_NIGHTS: QuietHours[] = [
	{ start: '21:00', end: '06:00' },
	{ start: '22:00', end: '06:00' },
	{ start: '22:00', end: '07:00' },
	{ start: '23:00', end: '07:00' },
	{ start: '00:00', end: '08:00' },
];

/** A range as the select keeps it: "22:00-06:00". */
export const quietValue = (hours: QuietHours): string => `${hours.start}-${hours.end}`;
const quietLabel = (hours: QuietHours): string => `${hours.start}–${hours.end}`;

/** The choices for quiet hours; a saved range that isn't one of them comes first, so it stays. */
export function quietOptions(current: QuietHours | null): { value: string; label: string }[] {
	const nights = current && !QUIET_NIGHTS.some((night) => quietValue(night) === quietValue(current)) ? [current, ...QUIET_NIGHTS] : QUIET_NIGHTS;
	return nights.map((night) => ({ value: quietValue(night), label: quietLabel(night) }));
}

const TIME = /^([01]\d|2[0-3]):[0-5]\d$/;

/** A chosen range back into the backend's start and end; null for anything that isn't two times. */
export function parseQuiet(value: string): QuietHours | null {
	const [start, end, ...rest] = value.split('-');
	if (rest.length || !start || !end || !TIME.test(start) || !TIME.test(end) || start === end) return null;
	return { start, end };
}
