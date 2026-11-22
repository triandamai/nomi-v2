import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { initViewport } from './viewport';

describe('initViewport browser', () => {
	let originalVisualViewport: VisualViewport | null = null;

	beforeEach(() => {
		originalVisualViewport = window.visualViewport;
		document.documentElement.style.removeProperty('--viewport-height');
		document.documentElement.style.removeProperty('--viewport-top');
		window.scrollY = 0;
		window.scrollX = 0;
	});

	afterEach(() => {
		if (originalVisualViewport) {
			Object.defineProperty(window, 'visualViewport', {
				value: originalVisualViewport,
				configurable: true,
				writable: true
			});
		}
	});

	it('sets CSS variables from visualViewport when available and updates on resize', () => {
		const listeners = new Map<string, () => void>();
		const mockVV = {
			height: 520.4,
			offsetTop: 15.2,
			addEventListener: vi.fn((event: string, cb: () => void) => {
				listeners.set(event, cb);
			}),
			removeEventListener: vi.fn((event: string) => {
				listeners.delete(event);
			})
		};

		Object.defineProperty(window, 'visualViewport', {
			value: mockVV,
			configurable: true,
			writable: true
		});

		const cleanup = initViewport();

		expect(document.documentElement.style.getPropertyValue('--viewport-height')).toBe('520px');
		expect(document.documentElement.style.getPropertyValue('--viewport-top')).toBe('15px');

		// Simulate keyboard dismissal/resizing
		mockVV.height = 844;
		mockVV.offsetTop = 0;
		listeners.get('resize')?.();

		expect(document.documentElement.style.getPropertyValue('--viewport-height')).toBe('844px');
		expect(document.documentElement.style.getPropertyValue('--viewport-top')).toBe('0px');

		cleanup();
		expect(mockVV.removeEventListener).toHaveBeenCalledWith('resize', expect.any(Function));
		expect(mockVV.removeEventListener).toHaveBeenCalledWith('scroll', expect.any(Function));
	});

	it('resets window scroll if scrollY > 0', () => {
		const scrollToSpy = vi.spyOn(window, 'scrollTo').mockImplementation(() => {});

		Object.defineProperty(window, 'scrollY', {
			value: 120,
			configurable: true,
			writable: true
		});

		const cleanup = initViewport();

		expect(scrollToSpy).toHaveBeenCalledWith(0, 0);

		cleanup();
		scrollToSpy.mockRestore();
	});
});
