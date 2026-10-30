import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import Mascot from './Mascot.svelte';
import { CREW } from './crew';
import { tokenColor } from '../test-utils';

// Mascots are identity, not decoration: one per agent, flat colour, drawn from the pen file's
// own geometry at 96, 32 or 20px.

describe('Mascot', () => {
	it('has one for each of the ten crew members', () => {
		expect(CREW.map((member) => member.id)).toEqual(['nomi', 'koda', 'dana', 'kala', 'rena', 'tara', 'maya', 'raga', 'personality', 'supervisor']);
		expect(CREW.find((member) => member.id === 'personality')?.name).toBe('Personality');
	});

	it('draws each member in their own crew colour with two eyes', async () => {
		for (const member of CREW) {
			const { container } = await render(Mascot, { agent: member.id });
			const svg = container.querySelector('svg')!;
			expect(svg.getAttribute('viewBox')).toBe('0 0 96 96');
			const body = svg.querySelector('[data-part="body"]')!;
			expect(getComputedStyle(body).fill, member.id).toBe(tokenColor(`crew-${member.id}`));
			expect(svg.querySelectorAll('[data-part="eye"]'), member.id).toHaveLength(2);
		}
	});

	it('comes in the design’s three sizes', async () => {
		for (const size of [96, 32, 20] as const) {
			const { container } = await render(Mascot, { agent: 'koda', size });
			const svg = container.querySelector('svg')!;
			expect([svg.getAttribute('width'), svg.getAttribute('height')]).toEqual([String(size), String(size)]);
		}
	});

	it('is decorative next to a name, and named when it stands alone', async () => {
		const quiet = await render(Mascot, { agent: 'dana', size: 20 });
		expect(quiet.container.querySelector('svg')!.getAttribute('aria-hidden')).toBe('true');
		await render(Mascot, { agent: 'dana', size: 32, label: 'Dana' });
		await expect.element(page.getByRole('img', { name: 'Dana' })).toBeInTheDocument();
	});
});
