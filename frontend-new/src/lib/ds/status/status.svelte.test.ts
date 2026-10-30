import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import Status from './Status.svelte';
import Tag from './Tag.svelte';
import SectionLabel from './SectionLabel.svelte';
import Meter from './Meter.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Status is a 6px dot plus a word: colour never carries meaning alone.

describe('Status', () => {
	const cases = [
		{ kind: 'live', label: 'Live', dot: 'nm-accent', text: 'nm-text', mono: false },
		{ kind: 'ready', label: 'ready', dot: 'nm-accent', text: 'nm-text-2', mono: true },
		{ kind: 'idle', label: 'idle', dot: 'nm-text-3', text: 'nm-text-3', mono: true },
		{ kind: 'failed', label: 'Deploy failed', dot: 'nm-danger', text: 'nm-danger', mono: false },
		{ kind: 'attention', label: 'Needs attention', dot: 'nm-warning', text: 'nm-warning', mono: false },
	] as const;

	for (const c of cases) {
		it(`shows ${c.kind} as a dot and a word`, async () => {
			const { container } = await render(Status, { kind: c.kind, label: c.label });
			const dot = container.querySelector<HTMLElement>('[data-dot]')!;
			expect([getComputedStyle(dot).width, getComputedStyle(dot).height]).toEqual(['6px', '6px']);
			expect(getComputedStyle(dot).backgroundColor).toBe(tokenBackground(c.dot));
			const word = page.getByText(c.label, { exact: true }).element();
			expect(getComputedStyle(word).color).toBe(tokenColor(c.text));
			expect(getComputedStyle(word).fontFamily.includes('Geist Mono')).toBe(c.mono);
		});
	}

	it('shows who is working with their mascot instead of a dot', async () => {
		const { container } = await render(Status, { kind: 'working', label: 'Koda is building', agent: 'koda' });
		expect(container.querySelector('[data-dot]')).toBeNull();
		expect(container.querySelector('svg')?.getAttribute('width')).toBe('20');
		await expect.element(page.getByText('Koda is building')).toBeInTheDocument();
	});
});

describe('Tag', () => {
	it('says go, no-go or standby in small mono capitals', async () => {
		for (const [kind, label, text, background] of [
			['no-go', 'No-go', 'nm-danger', 'nm-danger-bg'],
			['go', 'Go', 'nm-accent', 'nm-accent-bg'],
			['neutral', 'Stby', 'nm-text-2', 'nm-surface-2'],
		] as const) {
			await render(Tag, { kind, label });
			const word = page.getByText(label, { exact: true }).element();
			const style = getComputedStyle(word);
			expect(style.textTransform).toBe('uppercase');
			expect(style.color).toBe(tokenColor(text));
			expect(getComputedStyle(word).backgroundColor).toBe(tokenBackground(background));
			expect(style.borderTopLeftRadius).toBe('4px');
		}
	});
});

describe('SectionLabel', () => {
	it('names a section in mono capitals, with a count', async () => {
		await render(SectionLabel, { label: 'Recents', count: 4 });
		expect(getComputedStyle(page.getByText('Recents').element()).textTransform).toBe('uppercase');
		await expect.element(page.getByText('4')).toBeInTheDocument();
	});
});

describe('Meter', () => {
	it('shows how much is used, as a bar and in words', async () => {
		const { container } = await render(Meter, { label: 'Free plan', value: 0.02, caption: '21.8K / 1M tokens this month' });
		const meter = page.getByRole('meter', { name: 'Free plan' });
		await expect.element(meter).toHaveAttribute('aria-valuenow', '2');
		await expect.element(page.getByText('2%')).toBeInTheDocument();
		await expect.element(page.getByText('21.8K / 1M tokens this month')).toBeInTheDocument();
		const track = container.querySelector<HTMLElement>('[data-track]')!;
		const fill = container.querySelector<HTMLElement>('[data-fill]')!;
		expect(getComputedStyle(track).height).toBe('4px');
		expect(getComputedStyle(fill).backgroundColor).toBe(tokenBackground('nm-text-2'));
		expect(fill.getBoundingClientRect().width / track.getBoundingClientRect().width).toBeCloseTo(0.02, 2);
	});

	it('never shows more than full', async () => {
		await render(Meter, { label: 'Free plan', value: 1.4, caption: '1.4M / 1M tokens this month' });
		await expect.element(page.getByText('100%')).toBeInTheDocument();
	});
});
