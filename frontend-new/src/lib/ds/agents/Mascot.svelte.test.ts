import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import Mascot from './Mascot.svelte';
import { CREW } from './crew';
import { CREW_LOOKS, FACE_COLOR, TONE_STOPS, shapePath } from './shapes';

// Each crew member wears frontend's agent shape: their own outline in their own gradient, with the
// two-dot face, at 96, 32, 20 or 16px.

describe('Mascot', () => {
	it('has one for each of the ten crew members', () => {
		expect(CREW.map((member) => member.id)).toEqual(['nomi', 'koda', 'dana', 'kala', 'rena', 'tara', 'maya', 'raga', 'personality', 'supervisor']);
		expect(CREW.find((member) => member.id === 'personality')?.name).toBe('Personality');
	});

	it('draws each member in their own shape and gradient, with the two-dot face', async () => {
		for (const member of CREW) {
			const look = CREW_LOOKS[member.id];
			const { container } = await render(Mascot, { agent: member.id });
			const svg = container.querySelector('svg')!;
			expect(svg.getAttribute('viewBox')).toBe('0 0 48 48');
			const body = svg.querySelector('[data-part="body"]')!;
			expect(body.getAttribute('d'), member.id).toBe(shapePath(look.shape));
			const gradient = svg.querySelector(body.getAttribute('fill')!.replace(/^url\((#.+)\)$/, '$1').replace(/:/g, '\\:'))!;
			expect([...gradient.querySelectorAll('stop')].map((stop) => stop.getAttribute('stop-color')), member.id).toEqual(TONE_STOPS[look.tone]);
			const eyes = svg.querySelectorAll('[data-part="eye"]');
			expect(eyes, member.id).toHaveLength(2);
			expect([...eyes].every((eye) => eye.getAttribute('fill') === FACE_COLOR)).toBe(true);
		}
	});

	it('wears frontend’s look for each member’s agent', () => {
		expect(CREW_LOOKS.nomi).toEqual({ shape: 'cookie9', tone: 'glow', motion: 'spin' });
		expect(CREW_LOOKS.koda.shape).toBe('cookie6');
		expect(CREW_LOOKS.dana.shape).toBe('sunny8');
		expect(CREW_LOOKS.rena.shape).toBe('clover4');
		expect(CREW_LOOKS.supervisor).toEqual({ shape: 'sunny12', tone: 'dusk', motion: 'orbit' });
	});

	it('keeps each gradient to its own mascot when several are on a page', async () => {
		const { container } = await render(Mascot, { agent: 'koda' });
		const second = await render(Mascot, { agent: 'dana' });
		const ids = [container, second.container].map((root) => root.querySelector('linearGradient')!.id);
		expect(new Set(ids).size).toBe(2);
	});

	it('comes in the design’s sizes', async () => {
		for (const size of [96, 32, 20, 16] as const) {
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
