import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import doc from '$lib/ds/tokens/tokens.json';
import ColourSection from './ColourSection.svelte';
import CrewColourSection from './CrewColourSection.svelte';
import TypeSection from './TypeSection.svelte';
import SpaceSection from './SpaceSection.svelte';

// The gallery rebuilds the design sheet "Design System / Nomi" from the same tokens the app uses,
// so it can be compared with the pen file side by side.

const tokens = doc.tokens as Record<string, { type: string; light?: string; dark?: string; value?: string | number }>;

describe('Colour section', () => {
	it('shows every themed colour token in a dark and a light column', async () => {
		await render(ColourSection);
		const themed = Object.entries(tokens).filter(([name, t]) => name.startsWith('nm-') && t.type === 'color');
		expect(themed.length).toBeGreaterThan(20);
		for (const [name] of themed) {
			expect(page.getByText(name, { exact: true }).elements(), name).toHaveLength(2);
		}
	});

	it('paints each swatch in its own theme', async () => {
		const { container } = await render(ColourSection);
		const swatch = (theme: string) =>
			container.querySelector<HTMLElement>(`[data-theme="${theme}"] [data-token="nm-accent"]`)!;
		expect(getComputedStyle(swatch('dark')).backgroundColor).toBe('rgb(123, 227, 164)');
		expect(getComputedStyle(swatch('light')).backgroundColor).toBe('rgb(30, 154, 88)');
	});
});

describe('Crew colour section', () => {
	it('gives each of the ten agents its colour, background and ink', async () => {
		await render(CrewColourSection);
		for (const agent of ['Nomi', 'Koda', 'Dana', 'Kala', 'Rena', 'Tara', 'Maya', 'Raga', 'Personality', 'Supervisor']) {
			expect(page.getByText(agent, { exact: true }).elements().length, agent).toBeGreaterThan(0);
			const key = agent.toLowerCase();
			for (const token of [`crew-${key}`, `crew-${key}-bg`, `crew-${key}-ink`]) {
				expect(tokens[token], token).toBeDefined();
			}
		}
	});
});

describe('Type section', () => {
	it('shows the five sizes with their use', async () => {
		await render(TypeSection);
		for (const [name, px] of [
			['nm-size-display', 28],
			['nm-size-body', 15],
			['nm-size-ui', 13],
			['nm-size-caption', 12],
			['nm-size-label', 11],
		] as const) {
			expect(tokens[name].value).toBe(px);
			expect(page.getByText(name, { exact: true }).elements().length, name).toBeGreaterThan(0);
		}
		const greeting = page.getByText('Morning, Mario.').element();
		expect(getComputedStyle(greeting).fontSize).toBe('28px');
		expect(getComputedStyle(greeting).fontWeight).toBe('500');
		const label = page.getByText("What's in this project").element();
		expect(getComputedStyle(label).textTransform).toBe('uppercase');
	});
});

describe('Space, radius and icons section', () => {
	it('shows each spacing step, both radii, the floating shadow and the icon sizes', async () => {
		await render(SpaceSection);
		for (const step of ['2xs', 'xs', 'sm', 'md', 'lg', 'xl', '2xl', '3xl', '4xl', '5xl']) {
			expect(page.getByText(`space-${step}`, { exact: true }).elements().length, step).toBe(1);
		}
		expect(page.getByText('radius-sm · 6px').elements()).toHaveLength(1);
		expect(page.getByText('radius-md · 10px').elements()).toHaveLength(1);
		expect(page.getByText('Floating', { exact: true }).elements()).toHaveLength(1);
		for (const size of ['16px', '14px', '12px']) {
			expect(page.getByText(size, { exact: true }).elements().length, size).toBeGreaterThan(0);
		}
	});
});
