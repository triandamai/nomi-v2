import { MAILPIT_URL } from './env';

type Message = { ID: string; Subject: string; To: { Address: string }[]; Created: string };

/** The emails sent to `address`, newest first. */
export async function emailsTo(address: string): Promise<Message[]> {
	const response = await fetch(`${MAILPIT_URL}/api/v1/search?query=${encodeURIComponent(`to:${address}`)}`);
	if (!response.ok) throw new Error(`Mailpit search failed: ${response.status}`);
	return ((await response.json()) as { messages: Message[] }).messages;
}

/**
 * The 6-digit code in the newest email to `address` whose subject matches, waiting for it to
 * arrive (the backend sends it right after answering).
 */
export async function codeFor(address: string, subject: RegExp = /code|confirm/i, { after = 0, timeoutMs = 10_000 } = {}): Promise<string> {
	const deadline = Date.now() + timeoutMs;
	while (Date.now() < deadline) {
		const message = (await emailsTo(address)).find((m) => subject.test(m.Subject) && Date.parse(m.Created) >= after);
		const code = message?.Subject.match(/\b(\d{6})\b/)?.[1];
		if (code) return code;
		await new Promise((resolve) => setTimeout(resolve, 250));
	}
	throw new Error(`No code email to ${address} within ${timeoutMs}ms`);
}

/** The link in the newest email to `address` whose body has one starting with `prefix`. */
export async function linkFor(address: string, prefix: string, { timeoutMs = 10_000 } = {}): Promise<string> {
	const deadline = Date.now() + timeoutMs;
	while (Date.now() < deadline) {
		for (const message of await emailsTo(address)) {
			const body = (await (await fetch(`${MAILPIT_URL}/api/v1/message/${message.ID}`)).json()) as { Text: string; HTML: string };
			const link = `${body.Text}\n${body.HTML}`.match(new RegExp(`${prefix.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}[^\\s"'<>]*`))?.[0];
			if (link) return link;
		}
		await new Promise((resolve) => setTimeout(resolve, 250));
	}
	throw new Error(`No email to ${address} linking to ${prefix} within ${timeoutMs}ms`);
}
