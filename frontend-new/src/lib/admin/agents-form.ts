import { m } from '$lib/paraglide/messages';

// A custom agent as the admin form writes it, checked as the backend checks it
// (dynamic_agents.rs::validate_request), and the looks a custom agent's mascot can take
// (agents.rs: SHAPES, TONES, MOTIONS).

export const SHAPES = ['cookie9', 'sunny8', 'cookie6', 'clover4', 'flower5', 'circle', 'sunny12', 'clover3', 'puffy7', 'burst16', 'wave10', 'soft-square', 'soft-triangle', 'pentagon', 'pill'] as const;
export const TONES = ['glow', 'ember', 'tide', 'sky', 'bloom', 'dusk', 'citrus', 'slate'] as const;
export const MOTIONS = ['spin', 'wobble', 'bounce', 'pulse', 'orbit'] as const;

export type AgentBody = {
	name: string;
	system_prompt: string;
	intent_label: string;
	intent_description: string;
	granted_tools: string[];
	supports_todos: boolean;
	supports_plans: boolean;
	can_delegate: boolean;
	shape: string;
	tone: string;
	motion: string;
};

const text = (form: FormData, name: string): string => {
	const value = form.get(name);
	return typeof value === 'string' ? value.trim() : '';
};
const oneOf = (list: readonly string[], value: string) => list.includes(value);

/** The agent from the form (tools are the `tool_<name>` boxes ticked), or what's wrong with it. */
export function agentFromForm(form: FormData): { ok: true; body: AgentBody } | { ok: false; message: string } {
	const body: AgentBody = {
		name: text(form, 'name'),
		system_prompt: text(form, 'system_prompt'),
		intent_label: text(form, 'intent_label'),
		intent_description: text(form, 'intent_description'),
		granted_tools: [...form.keys()]
			.filter((key) => key.startsWith('tool_') && form.get(key) === 'on')
			.map((key) => key.slice('tool_'.length))
			.sort(),
		supports_todos: form.get('supports_todos') === 'on',
		supports_plans: form.get('supports_plans') === 'on',
		can_delegate: form.get('can_delegate') === 'on',
		shape: text(form, 'shape'),
		tone: text(form, 'tone'),
		motion: text(form, 'motion'),
	};
	if (!body.name || !body.system_prompt || !body.intent_label || !body.intent_description) return { ok: false, message: m.admin_agent_required() };
	if (!oneOf(SHAPES, body.shape) || !oneOf(TONES, body.tone) || !oneOf(MOTIONS, body.motion)) return { ok: false, message: m.admin_agent_look_invalid() };
	return { ok: true, body };
}
