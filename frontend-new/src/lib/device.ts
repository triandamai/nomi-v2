/**
 * This browser as one of the person's devices. Each browser keeps a device id for good (the
 * `device_id` cookie, made by the server on the first visit): every call to the backend names it
 * (`X-Device-Id`), and each tab's realtime connection is `<deviceId>-<tabId>` on the broker.
 */

export const DEVICE_COOKIE = 'device_id';
export const DEVICE_ID_HEADER = 'X-Device-Id';
const TAB_STORAGE_KEY = 'nomi.tab_id';
const TAB_ID_LENGTH = 8;
const RADIX_36 = 36;
/** Two years: a browser stays the same device until its cookies are cleared. */
export const DEVICE_COOKIE_MAX_AGE = 2 * 365 * 24 * 60 * 60;

/** What a browser tells the device list about itself. */
export interface BrowserDevice {
	name: string;
	brand: string | null;
	model: string | null;
	os_name: string | null;
	os_version: string | null;
}

/** Browsers by their user-agent token, most specific first (Edge and Opera also say Chrome). */
const BROWSERS: Array<{ name: string; brand: string; pattern: RegExp }> = [
	{ name: 'Edge', brand: 'Microsoft', pattern: /Edg(?:e|A|iOS)?\/([\d.]+)/ },
	{ name: 'Opera', brand: 'Opera', pattern: /(?:OPR|Opera)\/([\d.]+)/ },
	{ name: 'Samsung Internet', brand: 'Samsung', pattern: /SamsungBrowser\/([\d.]+)/ },
	{ name: 'Firefox', brand: 'Mozilla', pattern: /(?:Firefox|FxiOS)\/([\d.]+)/ },
	{ name: 'Chrome', brand: 'Google', pattern: /(?:Chrome|CriOS)\/([\d.]+)/ },
	{ name: 'Safari', brand: 'Apple', pattern: /Version\/([\d.]+).*Safari/ },
];

/** Operating systems by their user-agent token; the version's `_` become `.`. */
const SYSTEMS: Array<{ name: string; pattern: RegExp }> = [
	{ name: 'iOS', pattern: /(?:iPhone|iPad|iPod).*? OS ([\d_]+)/ },
	{ name: 'Android', pattern: /Android ([\d.]+)/ },
	{ name: 'Windows', pattern: /Windows NT ([\d.]+)/ },
	{ name: 'macOS', pattern: /Mac OS X ([\d_.]+)/ },
	{ name: 'ChromeOS', pattern: /CrOS \S+ ([\d.]+)/ },
	{ name: 'Linux', pattern: /Linux()/ },
];

/** "Chrome on macOS", with the browser as brand and model, from a user agent. */
export function describeBrowser(userAgent: string): BrowserDevice {
	const browser = BROWSERS.map((b) => ({ ...b, match: userAgent.match(b.pattern) })).find((b) => b.match);
	const system = SYSTEMS.map((s) => ({ ...s, match: userAgent.match(s.pattern) })).find((s) => s.match);
	const browserName = browser?.name ?? 'Browser';
	const major = browser?.match?.[1]?.split('.')[0];
	return {
		name: system ? `${browserName} on ${system.name}` : browserName,
		brand: browser?.brand ?? null,
		model: browser ? `${browserName}${major ? ` ${major}` : ''}` : null,
		os_name: system?.name ?? null,
		os_version: system?.match?.[1]?.replaceAll('_', '.') || null,
	};
}

/** This tab's own id, kept while the tab lives (a reload keeps it, a new tab gets another). */
export function tabId(): string {
	try {
		const saved = sessionStorage.getItem(TAB_STORAGE_KEY);
		if (saved) return saved;
		const made = newTabId();
		sessionStorage.setItem(TAB_STORAGE_KEY, made);
		return made;
	} catch {
		// Storage blocked: a fresh id per page load still keeps tabs apart.
		return newTabId();
	}
}

function newTabId(): string {
	return Math.random().toString(RADIX_36).slice(2, 2 + TAB_ID_LENGTH).padEnd(TAB_ID_LENGTH, '0');
}
