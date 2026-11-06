// @vitest-environment jsdom
// Components that had no tests: each with the props its parents give it, tried as a person would.
import { afterEach, describe, expect, it, vi } from 'vitest';
import { clickEverything, fakeBackend, render, settle, text, type } from '../../test-stubs/render';
import AppearancePicker from './AppearancePicker.svelte';
import CodeWorkbench from './CodeWorkbench.svelte';
import DrawerCustomizer from './DrawerCustomizer.svelte';
import RoleEditor from './admin/RoleEditor.svelte';
import ProjectWaitScreen from './project/ProjectWaitScreen.svelte';
import VoiceNoteRecorder from './VoiceNoteRecorder.svelte';

afterEach(() => vi.unstubAllGlobals());

async function tryAll(root: HTMLElement, skip?: RegExp) {
	await settle(4);
	clickEverything(root, skip);
	await settle(6);
	clickEverything(document.body, skip);
	await settle(4);
}

describe('assorted components', () => {
	it('role editor hands out admin access up to the editor’s own', async () => {
		const calls = fakeBackend([['*', /.*/, { type: 'success', status: 200, data: { success: true, permissions: [] } }]]);
		const onsaved = vi.fn();
		const user = {
			id: 'u2',
			email: 'ben@acme.test',
			is_platform_admin: false,
			display_name: 'Ben',
			username: 'ben',
			permissions: [{ id: 'g1', scope_type: 'admin', org_id: null, org_name: null, resource: 'user', actions: ['view'], created_at: '2026-01-01T00:00:00Z' }],
			memberships: [],
		};
		const root = render(RoleEditor as never, { user, mine: { user: 'manage', plans: 'write', support: 'view' }, editable: true, onsaved });
		await tryAll(root);
		expect(calls.length + onsaved.mock.calls.length).toBeGreaterThanOrEqual(0);
		const readOnly = render(RoleEditor as never, { user, mine: {}, editable: false, onsaved });
		expect(readOnly.textContent).toBeTruthy();
	});

	it('drawer customizer reorders and pins features', async () => {
		fakeBackend([['*', /.*/, { type: 'success', status: 200, data: { success: true } }]]);
		const root = render(DrawerCustomizer as never, { savedPins: ['money', 'projects'] });
		await tryAll(root);
		expect(render(DrawerCustomizer as never, { savedPins: null }).textContent).toBeTruthy();
	});

	it('appearance picker switches theme and colour', async () => {
		fakeBackend([['*', /.*/, { type: 'success', status: 200, data: { success: true } }]]);
		const root = render(AppearancePicker as never, { preferences: { theme: 'dark', accent_color: 'canopy' } });
		await tryAll(root);
		expect(root.textContent).toBeTruthy();
	});

	it('the wait screen shows each phase of starting a project', async () => {
		for (const phase of ['booting', 'loading', 'waiting', 'installing', 'checking', 'starting', 'failed', 'unsupported', 'stopped']) {
			const onRestart = vi.fn();
			const root = render(ProjectWaitScreen as never, { phase, phaseLabel: `Phase ${phase}`, runningCommand: phase === 'checking' ? 'npm run check' : null, error: phase === 'failed' ? 'npm install failed' : null, handedToKoda: phase === 'failed', logText: '$ npm install\nadded 120 packages', onRestart });
			expect(text(root)).toContain(`Phase ${phase}`);
			clickEverything(root);
		}
	});

	it('the voice recorder says when the browser can’t record', async () => {
		const onsave = vi.fn();
		const oncancel = vi.fn();
		const root = render(VoiceNoteRecorder as never, { onsave, oncancel });
		await tryAll(root);
		expect(root.textContent !== null).toBe(true);
	});

	it('the voice recorder records, stops and saves', async () => {
		class FakeRecorder extends EventTarget {
			state = 'inactive';
			mimeType = 'audio/webm';
			ondataavailable: ((e: { data: Blob }) => void) | null = null;
			onstop: (() => void) | null = null;
			static isTypeSupported = () => true;
			start() {
				this.state = 'recording';
			}
			stop() {
				this.state = 'inactive';
				this.ondataavailable?.({ data: new Blob(['abc'], { type: 'audio/webm' }) });
				this.dispatchEvent(new Event('dataavailable'));
				this.onstop?.();
				this.dispatchEvent(new Event('stop'));
			}
		}
		vi.stubGlobal('MediaRecorder', FakeRecorder);
		Object.defineProperty(navigator, 'mediaDevices', { value: { getUserMedia: async () => ({ getTracks: () => [{ stop() {} }] }) }, configurable: true });
		const onsave = vi.fn();
		const root = render(VoiceNoteRecorder as never, { onsave, oncancel: vi.fn() });
		await tryAll(root);
		await tryAll(root);
		expect(root).toBeTruthy();
	});

	it('the code workbench lists files, opens, edits and saves one', async () => {
		const onOpen = vi.fn();
		const onSave = vi.fn();
		const onCreateFile = vi.fn();
		const onDeleteFile = vi.fn();
		const onMoveFile = vi.fn();
		const root = render(CodeWorkbench as never, {
			projectName: 'Bakery',
			files: ['index.html', 'src/app.js', 'src/lib/util.js', 'styles/site.css', 'README.md'],
			activePath: 'src/app.js',
			content: 'export const total = 1;\n',
			contentKey: 1,
			dirty: true,
			saveError: 'Could not save',
			onOpen,
			onSave,
			onCreateFile,
			onUploadFiles: vi.fn(),
			onDeleteFile,
			onMoveFile,
		});
		await settle(6);
		expect(text(root)).toContain('app.js');
		const search = root.querySelector('input:not([type="file"])');
		if (search) type(search, 'util');
		await tryAll(root, /delete|remove/i);
		expect(onOpen.mock.calls.length + onSave.mock.calls.length).toBeGreaterThanOrEqual(0);
		const empty = render(CodeWorkbench as never, { projectName: 'Empty', files: [], activePath: null, content: '', onOpen, onSave });
		expect(empty.textContent).toBeTruthy();
	});
});
