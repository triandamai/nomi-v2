import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import { ArrowRight, Users } from '@lucide/svelte';
import Button from './Button.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Design System / Nomi → Actions: height 30, radius-sm, 13px; one primary per view, Inverse for
// the main action inside panels, so mint stays rare.

const box = (element: Element) => getComputedStyle(element);

describe('Button', () => {
	it('is a 30px button with a 13px label, type=button unless it submits a form', async () => {
		await render(Button, { label: 'Start building' });
		const button = page.getByRole('button', { name: 'Start building' }).element();
		expect(button.getAttribute('type')).toBe('button');
		expect(box(button).height).toBe('30px');
		expect(box(button).borderTopLeftRadius).toBe('6px');
		expect(box(button).fontSize).toBe('13px');
		expect(box(button).paddingLeft).toBe('12px');
	});

	it('paints each variant from its tokens', async () => {
		const cases = [
			{ variant: 'primary', background: 'nm-accent', color: 'nm-accent-ink', weight: '500' },
			{ variant: 'inverse', background: 'nm-text', color: 'nm-bg', weight: '500' },
			{ variant: 'secondary', background: null, color: 'nm-text', weight: '400', border: 'nm-border-strong' },
			{ variant: 'ghost', background: null, color: 'nm-text-2', weight: '400' },
			{ variant: 'danger', background: 'nm-danger-bg', color: 'nm-danger', weight: '500', border: 'nm-danger-border' },
		] as const;
		for (const c of cases) {
			const { container } = await render(Button, { label: c.variant, variant: c.variant });
			const style = box(container.querySelector('button')!);
			expect(style.backgroundColor, c.variant).toBe(c.background ? tokenBackground(c.background) : 'rgba(0, 0, 0, 0)');
			expect(style.color, c.variant).toBe(tokenColor(c.color));
			expect(style.fontWeight, c.variant).toBe(c.weight);
			if ('border' in c) expect(style.borderTopColor, c.variant).toBe(tokenColor(c.border));
		}
	});

	it('puts a trailing icon after the label and a leading one before it, 14px each', async () => {
		const { container } = await render(Button, { label: 'Start building', iconEnd: ArrowRight });
		const button = container.querySelector('button')!;
		const parts = [...button.children];
		expect(parts.map((part) => (part.querySelector('svg') ? 'icon' : part.textContent))).toEqual(['Start building', 'icon']);
		expect(button.querySelector('svg')?.getAttribute('width')).toBe('14');

		const ghost = await render(Button, { label: 'Team and invites', variant: 'ghost', iconStart: Users });
		const ghostParts = [...ghost.container.querySelector('button')!.children];
		expect(ghostParts.map((part) => (part.querySelector('svg') ? 'icon' : part.textContent))).toEqual(['icon', 'Team and invites']);
	});

	it('does what it says when clicked or pressed', async () => {
		const onclick = vi.fn();
		await render(Button, { label: 'Open workspace', onclick });
		await page.getByRole('button', { name: 'Open workspace' }).click();
		await userEvent.keyboard('{Enter}');
		expect(onclick).toHaveBeenCalledTimes(2);
	});

	it('shows the focus ring when reached by keyboard', async () => {
		await render(Button, { label: 'Start building' });
		await userEvent.tab();
		const button = page.getByRole('button', { name: 'Start building' }).element();
		expect(document.activeElement).toBe(button);
		expect(box(button).outlineColor).toBe(tokenColor('nm-focus'));
		expect(box(button).outlineStyle).toBe('solid');
	});

	it('darkens a secondary button on hover', async () => {
		await render(Button, { label: 'Add credits', variant: 'secondary' });
		const button = page.getByRole('button', { name: 'Add credits' });
		await userEvent.hover(button);
		expect(box(button.element()).backgroundColor).toBe(tokenBackground('nm-surface-2'));
	});

	it('is dimmed and does nothing while disabled', async () => {
		const onclick = vi.fn();
		await render(Button, { label: 'Start building', disabled: true, onclick });
		const button = page.getByRole('button', { name: 'Start building' }).element() as HTMLButtonElement;
		expect(button.disabled).toBe(true);
		expect(box(button).opacity).toBe('0.4');
		button.click();
		expect(onclick).not.toHaveBeenCalled();
	});

	it('shows real progress while loading: a spinner and the label it is given, and no second click', async () => {
		const onclick = vi.fn();
		const { container } = await render(Button, { label: 'Deploying · 00:14', variant: 'inverse', loading: true, onclick });
		const button = page.getByRole('button', { name: 'Deploying · 00:14' }).element() as HTMLButtonElement;
		expect(button.getAttribute('aria-busy')).toBe('true');
		expect(button.disabled).toBe(true);
		expect(container.querySelector('svg.lucide-loader-circle')).not.toBeNull();
		// Loading isn't disabled-looking: it's working.
		expect(box(button).opacity).toBe('1');
	});

	it('is a link when it goes somewhere', async () => {
		await render(Button, { label: 'Open workspace', href: '/projects/warung' });
		const link = page.getByRole('link', { name: 'Open workspace' }).element();
		expect(link.getAttribute('href')).toBe('/projects/warung');
		expect(box(link).height).toBe('30px');
	});

	it('can be shown hovered or focused for the gallery without a pointer', async () => {
		await render(Button, { label: 'Add credits', variant: 'secondary', 'data-hover': true });
		expect(box(page.getByRole('button').element()).backgroundColor).toBe(tokenBackground('nm-surface-2'));
	});
});
