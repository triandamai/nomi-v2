// @vitest-environment jsdom
import { describe, expect, it } from 'vitest';
import { render } from '../../test-stubs/render';
import DesignStudioLink from './DesignStudioLink.svelte';

describe('DesignStudioLink', () => {
	it('opens the chat’s Design Studio once it has a preview', () => {
		const root = render(DesignStudioLink as never, { sessionId: 's1', versions: 2 });
		const link = root.querySelector('a') as HTMLAnchorElement;
		expect(link.getAttribute('href')).toBe('/chat/s1/design');
		expect(link.getAttribute('aria-label')).toBe('Open the Design Studio');
	});

	it('is not there before Sora has made a preview', () => {
		const root = render(DesignStudioLink as never, { sessionId: 's1', versions: 0 });
		expect(root.querySelector('a')).toBeNull();
	});
});
