import type { ChatEvent } from '$lib/chat/events';

/** Live state of what Sora is currently doing on the Design Studio canvas. */
export interface SoraCanvasActivity {
	active: boolean;
	/** Short human-readable status shown in Sora's multiplayer cursor pill. */
	label: string;
	/** Screen or element id Sora is creating, editing, or inspecting (if known). */
	targetId: string | null;
	/** Tool Sora is currently running (e.g. `save_preview`, `screenshot_preview`). */
	tool: string | null;
	/** Increments whenever Sora streams a new element or phase so the cursor glides smoothly. */
	tick: number;
}

export interface SoraActivityTracker {
	state: SoraCanvasActivity;
	/** Whether the latest event finished a design update and the canvas should reload previews. */
	shouldRefresh: boolean;
	/** Internal rolling buffer for parsing element ids/names from streaming tool input JSON. */
	streamBuffer: string;
	/** Whether Sora called a preview-mutating tool during the current run. */
	mutatedDesign: boolean;
}

const DESIGN_TOOLS = new Set([
	'save_preview',
	'screenshot_preview',
	'propose_directions',
	'read_preview',
	'read_design_skill',
	'approve_version',
	'create_handoff',
]);

const MUTATING_TOOLS = new Set(['save_preview', 'propose_directions', 'approve_version']);

export function createSoraTracker(): SoraActivityTracker {
	return {
		state: { active: false, label: '', targetId: null, tool: null, tick: 0 },
		shouldRefresh: false,
		streamBuffer: '',
		mutatedDesign: false,
	};
}

function labelForTool(tool: string, target: string | null): string {
	switch (tool) {
		case 'save_preview':
			return target ? `Drawing ${target}` : 'Drawing canvas…';
		case 'screenshot_preview':
			return target ? `Inspecting ${target}` : 'Inspecting preview…';
		case 'propose_directions':
			return 'Exploring directions…';
		case 'read_design_skill':
			return target ? `Reviewing ${target}` : 'Reviewing craft…';
		case 'read_preview':
			return 'Reading canvas…';
		case 'create_handoff':
			return 'Preparing handoff…';
		default:
			return 'Designing…';
	}
}

/** Extracts the most recently streamed element id/name/target from a partial tool input JSON chunk. */
export function extractStreamTarget(tool: string | null, buffer: string): string | null {
	if (!tool || !buffer) return null;
	if (tool === 'screenshot_preview') {
		const matches = [...buffer.matchAll(/"target"\s*:\s*"([^"\\]{1,64})"/g)];
		const last = matches.at(-1)?.[1]?.trim();
		return last && last !== 'all' ? last : null;
	}
	if (tool === 'read_design_skill') {
		const matches = [...buffer.matchAll(/"name"\s*:\s*"([^"\\]{1,64})"/g)];
		return matches.at(-1)?.[1]?.trim() ?? null;
	}
	if (tool === 'save_preview') {
		// Inside canvas.pen JSON strings (often escaped as \"id\": \"hero-section\"), match both raw and escaped quotes.
		const idMatches = [...buffer.matchAll(/\\?"id\\?"\s*:\s*\\?"([a-z0-9][a-z0-9-]{1,48})\\?"/g)];
		const lastId = idMatches.at(-1)?.[1];
		if (lastId) return lastId;
		const nameMatches = [...buffer.matchAll(/\\?"name\\?"\s*:\s*\\?"([^"\\]{2,40})\\?"/g)];
		return nameMatches.at(-1)?.[1]?.trim() ?? null;
	}
	return null;
}

function isSoraName(name: string | null | undefined): boolean {
	return typeof name === 'string' && name.trim().toLowerCase() === 'sora';
}

/** Applies one live chat event and returns the next tracker state. */
export function applyChatEventToSora(tracker: SoraActivityTracker, event: ChatEvent): SoraActivityTracker {
	const next: SoraActivityTracker = {
		state: { ...tracker.state },
		shouldRefresh: false,
		streamBuffer: tracker.streamBuffer,
		mutatedDesign: tracker.mutatedDesign,
	};

	switch (event.kind) {
		case 'AgentSessionStarted': {
			if (event.agent_type === 'design' || isSoraName(event.agent_display_name)) {
				next.state = { active: true, label: 'Designing…', targetId: null, tool: null, tick: tracker.state.tick + 1 };
				next.streamBuffer = '';
				next.mutatedDesign = false;
			}
			return next;
		}
		case 'AgentPhaseChanged': {
			const sora = isSoraName(event.agent_display_name) || (event.detail ? DESIGN_TOOLS.has(event.detail) : false) || tracker.state.active;
			if (!sora) return next;
			const phase = event.phase.toLowerCase();
			if (phase === 'waiting' || phase === 'idle' || phase === 'done') {
				next.state = { ...next.state, active: false };
				if (next.mutatedDesign) {
					next.shouldRefresh = true;
					next.mutatedDesign = false;
				}
				return next;
			}
			const tool = event.detail && DESIGN_TOOLS.has(event.detail) ? event.detail : next.state.tool;
			if (tool && MUTATING_TOOLS.has(tool)) next.mutatedDesign = true;
			const label = tool ? labelForTool(tool, next.state.targetId) : phase === 'thinking' ? 'Thinking…' : 'Designing…';
			next.state = { active: true, label, targetId: next.state.targetId, tool, tick: tracker.state.tick + 1 };
			return next;
		}
		case 'Delta': {
			const raw = event.event as Record<string, unknown> | null;
			if (!raw || typeof raw !== 'object') return next;
			// Handle StreamEvent::ContentBlockStart { block: { ToolUse: { name } } } or tagged { type: "tool_use", name }
			const block = (raw.block ?? raw.ContentBlockStart) as Record<string, unknown> | undefined;
			const toolUse = (block?.ToolUse ?? (raw.type === 'tool_use' ? raw : undefined)) as Record<string, unknown> | undefined;
			if (toolUse && typeof toolUse.name === 'string' && DESIGN_TOOLS.has(toolUse.name)) {
				const tool = toolUse.name;
				if (MUTATING_TOOLS.has(tool)) next.mutatedDesign = true;
				next.streamBuffer = '';
				next.state = { active: true, label: labelForTool(tool, null), targetId: null, tool, tick: tracker.state.tick + 1 };
				return next;
			}
			// Handle StreamEvent::ToolInputDelta { partial_json }
			const delta = (raw.ToolInputDelta ?? raw) as Record<string, unknown>;
			if (typeof delta.partial_json === 'string' && next.state.tool) {
				const combined = (next.streamBuffer + delta.partial_json).slice(-1024);
				next.streamBuffer = combined;
				const target = extractStreamTarget(next.state.tool, combined);
				if (target && target !== next.state.targetId) {
					next.state = {
						active: true,
						label: labelForTool(next.state.tool, target),
						targetId: target,
						tool: next.state.tool,
						tick: tracker.state.tick + 1,
					};
				}
			}
			return next;
		}
		case 'AgentSessionEnded':
		case 'TurnCompleted':
		case 'ChatSettled': {
			if (tracker.state.active || tracker.mutatedDesign) {
				next.state = { ...next.state, active: false };
				next.shouldRefresh = tracker.mutatedDesign;
				next.mutatedDesign = false;
				next.streamBuffer = '';
			}
			return next;
		}
		case 'TurnFailed': {
			next.state = { ...next.state, active: false };
			next.mutatedDesign = false;
			next.streamBuffer = '';
			return next;
		}
		default:
			return next;
	}
}
