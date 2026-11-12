import { isRedirect } from '@sveltejs/kit';
import type { DeviceItem, EmailPreferences, Preferences, SignInMethods } from '$lib/adapters/api';
import { apiFetch, type ApiEvent } from './api';
import { readApiError } from './apiError';
import type { Outcome } from './chat';

// Settings › General's calls to the backend, as the person: their email, how they sign in (a
// password, Google), their signed-in devices, their preferences and which emails they get; adding
// and removing Google, and signing other devices out. Logic and API use from the old app
// (frontend/src/routes/(app)/account).

export type GeneralData = {
	email: string;
	methods: SignInMethods;
	/** Null when they couldn't be listed; the rest of the page still shows. */
	devices: DeviceItem[] | null;
	preferences: Preferences;
	emails: EmailPreferences | null;
};

/** A backend that can't be reached answers like one that's down. */
async function call(event: ApiEvent, path: string, init?: RequestInit): Promise<Response> {
	try {
		return await apiFetch(event, path, init);
	} catch (error) {
		if (isRedirect(error)) throw error;
		return new Response('', { status: 503 });
	}
}

async function read<T>(event: ApiEvent, path: string): Promise<T | null> {
	const response = await call(event, path);
	return response.ok ? ((await response.json()) as T) : null;
}

async function send<T>(event: ApiEvent, path: string, method: string, parse = false): Promise<Outcome<T>> {
	const response = await call(event, path, { method });
	if (!response.ok) return { ok: false, status: response.status, message: (await readApiError(response)).message };
	return { ok: true, data: (parse ? await response.json() : undefined) as T };
}

export async function loadGeneral(event: ApiEvent): Promise<GeneralData | null> {
	const [profile, methods, devices, preferences, emails] = await Promise.all([
		read<{ email: string }>(event, '/api/profile'),
		read<SignInMethods>(event, '/api/auth/google'),
		read<DeviceItem[]>(event, '/api/devices'),
		read<Preferences>(event, '/api/preferences'),
		read<EmailPreferences>(event, '/api/notifications/preferences'),
	]);
	if (!profile || !methods || !preferences) return null;
	return { email: profile.email, methods, devices, preferences, emails };
}

/** Where Google asks the person to add their account; it comes back through /auth/google/callback. */
export const startGoogleLink = (event: ApiEvent) => send<{ url: string }>(event, '/api/auth/google/link', 'POST', true);

export const unlinkGoogle = (event: ApiEvent) => send<void>(event, '/api/auth/google', 'DELETE');

/** Ends the sign-ins of these devices; each is told to leave. Stops at the first that fails. */
export async function signOutDevices(event: ApiEvent, ids: string[]): Promise<Outcome<void>> {
	for (const id of ids) {
		const done = await send<void>(event, `/api/devices/${encodeURIComponent(id)}`, 'DELETE');
		if (!done.ok) return done;
	}
	return { ok: true, data: undefined };
}
