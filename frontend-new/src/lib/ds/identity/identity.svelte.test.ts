import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import Avatar from './Avatar.svelte';
import AgentByline from './AgentByline.svelte';
import CrewRow from './CrewRow.svelte';
import { tokenColor } from '../test-utils';

// People and agents.

describe('Avatar', () => {
	it('shows a person’s initials in mono on a 28px tile', async () => {
		await render(Avatar, { name: 'Mario Prasetyo' });
		const tile = page.getByText('MP').element();
		expect(getComputedStyle(tile).fontFamily).toContain('Geist Mono');
		const box = getComputedStyle(tile.closest('[data-avatar]')!);
		expect([box.width, box.height]).toEqual(['28px', '28px']);
		await expect.element(page.getByRole('img', { name: 'Mario Prasetyo' })).toBeInTheDocument();
	});

	it('uses their picture when they have one', async () => {
		const { container } = await render(Avatar, { name: 'Sari', src: '/avatars/sari.png' });
		expect(container.querySelector('img')?.getAttribute('alt')).toBe('Sari');
	});
});

describe('AgentByline', () => {
	it('says who is speaking, with their mascot and when, in mono', async () => {
		const { container } = await render(AgentByline, { agent: 'koda', name: 'Koda', meta: '15:37 · 4m 12s' });
		expect(container.querySelector('svg')?.getAttribute('width')).toBe('20');
		expect(getComputedStyle(page.getByText('Koda').element()).fontWeight).toBe('500');
		expect(getComputedStyle(page.getByText('15:37 · 4m 12s').element()).fontFamily).toContain('Geist Mono');
	});
});

describe('CrewRow', () => {
	it('introduces a crew member with what they do and what they are doing', async () => {
		const { container } = await render(CrewRow, { agent: 'dana', name: 'Dana', role: 'Transactions, budgets, subscriptions', state: 'idle' });
		expect(container.querySelector('svg')?.getAttribute('width')).toBe('32');
		expect(getComputedStyle(page.getByText('Transactions, budgets, subscriptions').element()).color).toBe(tokenColor('nm-text-3'));
		expect(getComputedStyle(page.getByText('idle').element()).fontFamily).toContain('Geist Mono');
	});
});
