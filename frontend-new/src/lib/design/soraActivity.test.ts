import { describe, expect, it } from 'vitest';
import { applyChatEventToSora, createSoraTracker, extractStreamTarget } from './soraActivity';

describe('soraActivity', () => {
	it('activates Sora on AgentSessionStarted and tracks streaming save_preview elements', () => {
		let tracker = createSoraTracker();
		tracker = applyChatEventToSora(tracker, {
			kind: 'AgentSessionStarted',
			agent_session_id: 'a1',
			agent_type: 'design',
			agent_display_name: 'Sora',
		});
		expect(tracker.state.active).toBe(true);

		tracker = applyChatEventToSora(tracker, {
			kind: 'Delta',
			turn_job_id: 'j1',
			event: { block: { ToolUse: { name: 'save_preview' } } },
		});
		expect(tracker.state.tool).toBe('save_preview');
		expect(tracker.state.label).toBe('Drawing canvas…');

		tracker = applyChatEventToSora(tracker, {
			kind: 'Delta',
			turn_job_id: 'j1',
			event: { ToolInputDelta: { partial_json: '{"files":[{"path":"canvas.pen","content":"{\\"id\\":\\"hero-section\\"}"}]}' } },
		});
		expect(tracker.state.targetId).toBe('hero-section');
		expect(tracker.state.label).toBe('Drawing hero-section');

		tracker = applyChatEventToSora(tracker, { kind: 'TurnCompleted', turn_job_id: 'j1', message_id: 'm1' });
		expect(tracker.state.active).toBe(false);
		expect(tracker.shouldRefresh).toBe(true);
	});

	it('tracks screenshot_preview inspection target without forcing a refresh', () => {
		let tracker = createSoraTracker();
		tracker = applyChatEventToSora(tracker, {
			kind: 'AgentPhaseChanged',
			agent_session_id: 'a1',
			phase: 'calling_tool',
			detail: 'screenshot_preview',
			agent_display_name: 'Sora',
		});
		expect(tracker.state.active).toBe(true);
		expect(tracker.state.label).toBe('Inspecting preview…');

		tracker = applyChatEventToSora(tracker, {
			kind: 'Delta',
			turn_job_id: 'j1',
			event: { ToolInputDelta: { partial_json: '{"target":"menu-phone"}' } },
		});
		expect(tracker.state.targetId).toBe('menu-phone');
		expect(tracker.state.label).toBe('Inspecting menu-phone');

		tracker = applyChatEventToSora(tracker, { kind: 'TurnCompleted', turn_job_id: 'j1', message_id: 'm2' });
		expect(tracker.state.active).toBe(false);
		expect(tracker.shouldRefresh).toBe(false);
	});

	it('ignores target "all" for screenshot_preview', () => {
		expect(extractStreamTarget('screenshot_preview', '{"target":"all"}')).toBeNull();
		expect(extractStreamTarget('screenshot_preview', '{"target":"nav-bar"}')).toBe('nav-bar');
	});
});
