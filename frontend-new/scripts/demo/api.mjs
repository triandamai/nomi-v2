// A stand-in backend for `npm run dev:demo`: enough of the API for Home to look lived in, without a
// real backend or an account. Any access token is accepted. Nothing is saved: sending a message or
// opening a chat answers 404, as a backend that isn't there would.
import { createServer } from 'node:http';

const now = () => new Date().toISOString();
const member = (agent_type, name, role, state = 'idle') => ({ agent_type, name, role, is_dynamic: false, state, status: '' });
const MODEL = { key: 'nomi:gemini-flash', label: 'Gemini Flash 3.8', model_id: 'gemini-flash-3.8', provider_id: 'google', provider_label: 'Google', has_logo: false, chat_only: false };

const routes = {
	'/api/maintenance': () => ({ enabled: false, message: null, until: null }),
	'/api/profile': () => ({ display_name: 'Budi Santoso', username: 'budi', email: 'budi@example.com', avatar_url: null }),
	'/api/preferences': () => ({ theme: process.env.DEMO_THEME ?? 'system', timezone: 'Asia/Jakarta', language: process.env.DEMO_LANGUAGE ?? 'en' }),
	'/api/agents': () => ({
		agents: [
			member('chitchat', 'Nomi', 'Talks with you and routes the work'),
			member('coding', 'Koda', 'Builds and edits project files', 'working'),
			member('planning', 'Rena', 'Plans, to-dos and reminders', 'waiting'),
			member('money', 'Dana', 'Transactions, budgets, subscriptions'),
			member('reminders', 'Kala', 'Reminds you on time, and keeps your reminders'),
			member('files', 'Maya', 'Reads what you attach and passes it to the right agent'),
			member('workspace', 'Tara', 'Works in your own Google account'),
			member('health', 'Raga', 'Health, fitness and training'),
			member('personality', 'Personality', 'Keeps Nomi sounding like you want'),
			member('supervisor', 'Supervisor', 'Keeps track of the crew, and stops it when you ask'),
			member('iris', 'Iris', 'Looks into a project’s published app'),
			member('design', 'Sora', 'Design work on a site or app'),
		],
	}),
	'/api/notifications/unread': () => ({ unread: 1 }),
	'/api/sessions': () => ({
		sessions: [
			{ id: 'demo-1', title: 'Landing page Warung Kopi Budi', last_message: { content: 'Hero sudah siap.', created_at: now() }, created_at: now(), updated_at: now(), pinned: false },
			{ id: 'demo-2', title: 'Pengingat bayar listrik', last_message: { content: 'Diingatkan tiap tanggal 20.', created_at: now() }, created_at: now(), updated_at: now(), pinned: false },
		],
	}),
	'/api/orgs': () => ({ orgs: [], project_org_id: null }),
	'/api/llm/model-options': () => ({
		groups: [{ id: 'nomi', source: 'nomi', provider_id: 'google', has_logo: false, label: 'Nomi credits', connection_id: null, error: null, models: [{ key: MODEL.key, label: MODEL.label, model_id: MODEL.model_id, free: true, enabled: true }] }],
		current: MODEL,
	}),
	'/api/home': () => ({
		since: now(),
		timezone: 'Asia/Jakarta',
		while_you_were_out: [{ kind: 'reply', agent: 'Rena', title: 'Landing page Warung Kopi Budi', detail: '', session_id: 'demo-1', at: now() }],
		today: [],
		plans: [{ kind: 'plan', title: 'Landing page Warung Kopi Budi', agent: 'Rena', done: 3, total: 5, session_id: 'demo-1', updated_at: now() }],
		while_you_were_out_total: 1,
		today_total: 0,
		plans_total: 1,
	}),
};

export function startDemoApi(port) {
	return new Promise((resolve) => {
		const server = createServer((req, res) => {
			const path = (req.url ?? '/').split('?')[0];
			if (path === '/health') return res.writeHead(200).end('ok');
			const route = routes[path];
			if (!route) return res.writeHead(404).end();
			res.writeHead(200, { 'content-type': 'application/json' }).end(JSON.stringify(route()));
		});
		server.listen(port, '127.0.0.1', () => resolve(server));
	});
}
