import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import ShellsSection from './ShellsSection.svelte';
import RulesSection from './RulesSection.svelte';

const box = (element: Element) => element.getBoundingClientRect();
const style = (element: Element) => getComputedStyle(element);

describe('Layout shells section', () => {
	it('draws the three shells with their fixed columns', async () => {
		await render(ShellsSection);
		for (const text of ['App shell', 'Workspace shell', 'Focus shell', 'Sidebar 248', 'Agent chat 440', 'Column 480–720']) {
			expect(page.getByText(text, { exact: true }).elements(), text).toHaveLength(1);
		}
	});

	it('draws the diagrams as the pen does: 220 tall, border-strong, square regions at the pen’s sizes', async () => {
		await page.viewport(1512, 900);
		const { container } = await render(ShellsSection);
		const diagrams = [...container.querySelectorAll<HTMLElement>('[data-diagram]')];
		expect(diagrams).toHaveLength(3);
		for (const diagram of diagrams) {
			expect(box(diagram).height).toBe(220);
			expect(diagram.classList).toContain('border-border-strong');
			expect(style(diagram).padding).toBe('8px');
			// Diagram, name and use stack at 10px.
			expect(style(diagram.parentElement!).rowGap).toBe('10px');
		}
		const region = (label: string) => page.getByText(label, { exact: true }).element().parentElement!;
		for (const label of ['Sidebar 248', 'Top bar 52', 'Content · pad 32–56', 'Panel 300–380', 'Project bar 52', 'Agent chat 440', 'Preview / Code / Terminal', 'Column 480–720']) {
			const element = region(label);
			expect(element.classList, label).toContain('border-border');
			expect(element.classList, label).toContain('bg-surface');
			expect(style(element).borderTopLeftRadius, label).toBe('0px');
			expect(style(element).padding, label).toBe('8px');
			// The label sits top-left, in text-2.
			const text = page.getByText(label, { exact: true }).element();
			expect(text.classList, label).toContain('text-text-2');
			expect(box(text).top - box(element).top, label).toBe(9);
			expect(box(text).left - box(element).left, label).toBe(9);
		}
		expect(box(region('Sidebar 248')).width).toBe(62);
		expect(box(region('Panel 300–380')).width).toBe(90);
		expect(box(region('Top bar 52')).height).toBe(22);
		expect(box(region('Project bar 52')).height).toBe(22);
		expect(box(region('Agent chat 440')).width).toBe(110);
		expect(box(region('Column 480–720')).width).toBe(150);
		// No gaps between the regions of one body; the workspace's bar sits 6px above its body.
		expect(box(region('Top bar 52')).left).toBe(box(region('Sidebar 248')).right);
		expect(box(region('Content · pad 32–56')).top).toBe(box(region('Top bar 52')).bottom);
		expect(box(region('Panel 300–380')).left).toBe(box(region('Content · pad 32–56')).right);
		expect(box(region('Preview / Code / Terminal')).left).toBe(box(region('Agent chat 440')).right);
		expect(box(region('Agent chat 440')).top - box(region('Project bar 52')).bottom).toBe(6);
	});
});

describe('Rules section', () => {
	it('lists what to do and what not to, from the design sheet', async () => {
		await render(RulesSection);
		await expect.element(page.getByText('Use nm-* tokens for every colour, size, gap and radius')).toBeInTheDocument();
		await expect.element(page.getByText('Pure #000 or #FFF surfaces')).toBeInTheDocument();
	});
});
