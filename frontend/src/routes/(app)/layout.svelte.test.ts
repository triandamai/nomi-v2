// @vitest-environment jsdom
import { describe, expect, it } from 'vitest';
import { createRawSnippet } from 'svelte';
import { render, text } from '../../test-stubs/render';
import Layout from './+layout.svelte';

describe('(app) layout', () => {
	it('renders app navigation and children', () => {
		const children = createRawSnippet(() => ({
			render: () => '<main id="app-main-child">Child Content</main>',
		}));

		const data = {
			userEmail: 'ana@example.com',
			profile: { display_name: 'Ana', username: 'ana', email: 'ana@example.com', avatar_url: null },
			preferences: { theme: 'system', accent_color: 'canopy', timezone: 'UTC', language: 'en', has_stored_timezone: true },
			crew: [],
			usage: null,
			unread: 2,
			attachmentLimits: null,
			deviceId: 'dev-1',
		};

		const root = render(Layout, { data: data as never, children: children as never });
		expect(text(root)).toContain('Child Content');
	});
});
