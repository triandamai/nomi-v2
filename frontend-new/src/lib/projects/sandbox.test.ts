import { describe, expect, it } from 'vitest';
import { safePreviewUrl, sandboxView, usesSandbox } from './sandbox';

const SECRET = 'https://boat.example/p/s3cr3t-token';
const status = (over: Partial<Parameters<typeof sandboxView>[0]> = {}) => ({ runtime: 'sandbox', available: true, state: 'running', phase: 'running' as const, preview_url: SECRET, error: null, ...over });

describe('a project’s sandbox', () => {
	it('frames only an https address', () => {
		expect(safePreviewUrl(SECRET)).toBe(SECRET);
		expect(safePreviewUrl('http://boat.example/p')).toBeNull();
		expect(safePreviewUrl('javascript:alert(1)')).toBeNull();
		expect(safePreviewUrl('not a url')).toBeNull();
		expect(safePreviewUrl(null)).toBeNull();
	});

	it('runs in the sandbox only when the project does and it’s available', () => {
		expect(usesSandbox(status())).toBe(true);
		expect(usesSandbox(status({ available: false }))).toBe(false);
		expect(usesSandbox(status({ runtime: 'browser' }))).toBe(false);
		expect(usesSandbox(null)).toBe(false);
	});

	it('says what the preview is doing in the backend’s phases', () => {
		expect(sandboxView(status({ phase: 'idle', preview_url: null }))).toEqual({ phase: 'waking', previewUrl: null, error: null });
		expect(sandboxView(status({ phase: 'waking', preview_url: null }))).toEqual({ phase: 'waking', previewUrl: null, error: null });
		expect(sandboxView(status({ phase: 'running', preview_url: null }))).toEqual({ phase: 'starting', previewUrl: null, error: null });
		expect(sandboxView(status())).toEqual({ phase: 'running', previewUrl: SECRET, error: null });
		expect(sandboxView(status({ phase: 'stopped', preview_url: null }))).toEqual({ phase: 'stopped', previewUrl: null, error: null });
		expect(sandboxView(status({ phase: 'error', preview_url: null, error: 'out of memory' }))).toEqual({ phase: 'failed', previewUrl: null, error: 'out of memory' });
	});

	it('shows why a running sandbox has no preview, and never frames an unsafe address', () => {
		expect(sandboxView(status({ preview_url: null, error: 'npm install failed' }))).toEqual({ phase: 'failed', previewUrl: null, error: 'npm install failed' });
		expect(sandboxView(status({ preview_url: 'http://boat.example/p' })).previewUrl).toBeNull();
	});
});
