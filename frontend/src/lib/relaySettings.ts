// Admin → Integrations → Relay: turning the settings form into what Relay takes, and Relay's
// refusal back into each check that failed.

export interface RelayCheck {
	name: string;
	ok: boolean;
	message?: string;
}

/** An SMTP server, as Relay answers it (never with its password). */
export interface RelaySmtp {
	host: string;
	port: number;
	user: string;
	password: string;
	from: string;
	security: string;
}

/** Relay's settings as it answers them: database URLs without their passwords. */
export interface RelaySettings {
	configured: boolean;
	public_domain: string;
	shared_dev_url: string;
	shared_production_url: string;
	postgrest_dev_url: string;
	postgrest_production_url: string;
	postgrest_role: string;
	preview_origins: string[];
	shared_smtp: RelaySmtp | null;
	checks: RelayCheck[];
}

export interface RelayConnection {
	source: 'admin' | 'environment' | 'none';
	control_url: string;
	key_set: boolean;
}

export interface RelayOverview {
	connection: RelayConnection;
	relay: RelaySettings | null;
	relay_error: string | null;
	default_preview_origins: string[];
}

/** The form: database URLs start empty (passwords are never shown back), origins are one text box. */
export interface RelaySettingsForm {
	public_domain: string;
	shared_dev_url: string;
	shared_production_url: string;
	postgrest_dev_url: string;
	postgrest_production_url: string;
	postgrest_role: string;
	preview_origins: string;
	/** The shared sender for sign-in codes; the password starts empty and stays as stored. */
	smtp_host: string;
	smtp_port: string;
	smtp_security: string;
	smtp_user: string;
	smtp_password: string;
	smtp_from: string;
}

/** Port 587 with STARTTLS: what most providers (Resend, Postmark, SES) take. */
export const DEFAULT_SMTP_PORT = 587;

export function originsFromText(text: string): string[] {
	return text
		.split(/[\n,]/)
		.map((o) => o.trim())
		.filter(Boolean);
}

/**
 * What `PUT /api/admin/relay/settings` takes: database URLs only when typed, so stored ones stay;
 * the shared sender when a host is typed, or with an empty host to remove one that was stored.
 */
export function settingsBody(form: RelaySettingsForm, hadSender: boolean): Record<string, unknown> {
	const body: Record<string, unknown> = {
		public_domain: form.public_domain.trim(),
		postgrest_dev_url: form.postgrest_dev_url.trim(),
		postgrest_production_url: form.postgrest_production_url.trim(),
		postgrest_role: form.postgrest_role.trim(),
		preview_origins: originsFromText(form.preview_origins),
	};
	if (form.shared_dev_url.trim()) body.shared_dev_url = form.shared_dev_url.trim();
	if (form.shared_production_url.trim()) body.shared_production_url = form.shared_production_url.trim();
	if (form.smtp_host.trim() || hadSender) {
		body.shared_smtp = {
			host: form.smtp_host.trim(),
			port: Number(form.smtp_port) || DEFAULT_SMTP_PORT,
			security: form.smtp_security,
			user: form.smtp_user.trim(),
			password: form.smtp_password,
			from: form.smtp_from.trim(),
		};
	}
	return body;
}

/** Relay's 422 `{ message, checks }`, or any other failure as its text. */
export function readRefusal(error: unknown): { message: string; checks: RelayCheck[] } {
	const text = error instanceof Error ? error.message : String(error);
	try {
		const parsed = JSON.parse(text) as { message?: string; checks?: RelayCheck[] };
		if (parsed && typeof parsed.message === 'string') return { message: parsed.message, checks: Array.isArray(parsed.checks) ? parsed.checks : [] };
	} catch {
		// Not JSON: a plain message.
	}
	return { message: text, checks: [] };
}
