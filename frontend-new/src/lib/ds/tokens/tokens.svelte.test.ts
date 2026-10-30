import { afterEach, describe, expect, it } from 'vitest';
import doc from './tokens.json';

// The utilities a component uses resolve to the design's tokens, in dark and in light, and
// nothing else is available: Tailwind's own palette, radii and shadows are switched off.

type Token = { light?: string; dark?: string; value?: string | number };
const tokens = doc.tokens as Record<string, Token>;

/** `#RRGGBB` as the browser reports a computed colour. */
function rgb(hex: string): string {
	const [r, g, b] = [1, 3, 5].map((at) => parseInt(hex.slice(at, at + 2), 16));
	return `rgb(${r}, ${g}, ${b})`;
}

function probe(theme: 'dark' | 'light', classes: string): HTMLElement {
	const wrapper = document.createElement('div');
	wrapper.dataset.theme = theme;
	wrapper.innerHTML = `<p class="${classes}">probe</p>`;
	document.body.append(wrapper);
	return wrapper.firstElementChild as HTMLElement;
}

afterEach(() => {
	document.body.innerHTML = '';
});

describe('Tailwind theme', () => {
	for (const theme of ['dark', 'light'] as const) {
		it(`maps colour utilities to the ${theme} tokens`, () => {
			const element = probe(theme, 'bg-surface text-text border border-border-strong outline-accent');
			const style = getComputedStyle(element);
			expect(style.backgroundColor).toBe(rgb(tokens['nm-surface'][theme]!));
			expect(style.color).toBe(rgb(tokens['nm-text'][theme]!));
			expect(style.borderTopColor).toBe(rgb(tokens['nm-border-strong'][theme]!));
			expect(style.outlineColor).toBe(rgb(tokens['nm-accent'][theme]!));
		});

		it(`gives each agent its colour in ${theme}`, () => {
			const element = probe(theme, 'bg-crew-koda-bg text-crew-koda-ink');
			const style = getComputedStyle(element);
			expect(style.backgroundColor).toBe(rgb(tokens['crew-koda-bg'][theme]!));
			expect(style.color).toBe(rgb(tokens['crew-koda-ink'][theme]!));
		});
	}

	it('uses the design’s type, radii and single floating shadow', () => {
		const element = probe('dark', 'font-sans text-ui rounded-sm shadow-float');
		const style = getComputedStyle(element);
		expect(style.fontFamily).toContain('Geist');
		expect(style.fontSize).toBe('13px');
		expect(style.borderTopLeftRadius).toBe('6px');
		expect(style.boxShadow).toContain('0px 8px 24px');

		const mono = getComputedStyle(probe('dark', 'font-mono text-label rounded-md'));
		expect(mono.fontFamily).toContain('Geist Mono');
		expect(mono.fontSize).toBe('11px');
		expect(mono.borderTopLeftRadius).toBe('10px');
	});

	it('has no colours, radii or shadows besides the design’s', () => {
		const style = getComputedStyle(probe('dark', 'bg-red-500 text-blue-600 rounded-4xl shadow-lg'));
		expect(style.backgroundColor).toBe('rgba(0, 0, 0, 0)');
		expect(style.color).toBe(rgb(tokens['nm-text'].dark!));
		expect(style.borderTopLeftRadius).toBe('0px');
		expect(style.boxShadow).toBe('none');
	});

	it('has the phone radii the Mobile sheet draws: buttons 12, options 14, toasts 16, bubbles 18', () => {
		const radius = (name: string) => getComputedStyle(probe('dark', name)).borderTopLeftRadius;
		expect([radius('rounded-lg'), radius('rounded-xl'), radius('rounded-2xl'), radius('rounded-3xl')]).toEqual(['12px', '14px', '16px', '18px']);
	});

	it('sets the page ground and text from the tokens', () => {
		const page = getComputedStyle(document.body);
		expect(page.fontFamily).toContain('Geist');
		expect(page.fontSize).toBe('13px');
	});
});
