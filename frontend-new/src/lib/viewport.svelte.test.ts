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

	it('resets window scroll on editable focusin when visualViewport is shrunken', () => {
		const scrollToSpy = vi.spyOn(window, 'scrollTo').mockImplementation(() => {});

		Object.defineProperty(window, 'scrollY', {
			value: 120,
			configurable: true,
			writable: true
		});
		Object.defineProperty(window, 'innerHeight', {
			value: 800,
			configurable: true,
			writable: true
		});
		Object.defineProperty(window, 'visualViewport', {
			value: { height: 450, offsetTop: 0, addEventListener: vi.fn(), removeEventListener: vi.fn() },
			configurable: true,
			writable: true
		});

		Object.defineProperty(document.documentElement, 'scrollHeight', {
			value: 600,
			configurable: true,
			writable: true
		});

		const cleanup = initViewport();

		const input = document.createElement('input');
		document.body.appendChild(input);

		input.dispatchEvent(new Event('focusin', { bubbles: true }));

		expect(scrollToSpy).toHaveBeenCalledWith(0, 0);

		scrollToSpy.mockClear();

		const button = document.createElement('button');
		document.body.appendChild(button);

		button.dispatchEvent(new Event('focusin', { bubbles: true }));

		expect(scrollToSpy).not.toHaveBeenCalled();

		document.body.removeChild(input);
		document.body.removeChild(button);
		cleanup();
		scrollToSpy.mockRestore();
	});

	it('resets window scroll on app shell layouts when window scrolls or visualViewport updates', () => {
		const scrollToSpy = vi.spyOn(window, 'scrollTo').mockImplementation(() => {});

		const shell = document.createElement('div');
		shell.setAttribute('data-app-shell', '');
		document.body.appendChild(shell);

		Object.defineProperty(window, 'scrollY', {
			value: 200,
			configurable: true,
			writable: true
		});

		const cleanup = initViewport();

		expect(scrollToSpy).toHaveBeenCalledWith(0, 0);

		scrollToSpy.mockClear();

		window.dispatchEvent(new Event('scroll'));
		expect(scrollToSpy).toHaveBeenCalledWith(0, 0);

		document.body.removeChild(shell);
		cleanup();
		scrollToSpy.mockRestore();
	});
});
