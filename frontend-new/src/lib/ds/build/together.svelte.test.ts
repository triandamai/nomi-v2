import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import PeerAvatar from './PeerAvatar.svelte';
import TeammateMessage from './TeammateMessage.svelte';
import Typing from './Typing.svelte';
import JoinedNote from './JoinedNote.svelte';
import Presence from './Presence.svelte';
import RequestQueue from './RequestQueue.svelte';
import PublishRequest from './PublishRequest.svelte';
import RoleMenu from './RoleMenu.svelte';
import BuildInvite from './BuildInvite.svelte';
import PeerCursor from './PeerCursor.svelte';
import PresenceMenu from './PresenceMenu.svelte';
import FollowBar from './FollowBar.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Build → Build together: invite people into a project's chat. Everyone prompts the same Koda; asks
// queue so changes never collide; roles decide who can publish.

const sari = { name: 'Sari Wulandari', initials: 'SW', peer: 1 as const };
const mario = { name: 'Mario Prasetyo', initials: 'MP' };

describe('People in the chat', () => {
	it('gives each teammate their own colour, and you a neutral one', async () => {
		const { container } = await render(PeerAvatar, { initials: 'SW', peer: 1 });
		expect(getComputedStyle(container.firstElementChild!).backgroundColor).toBe(tokenBackground('nm-peer-1-bg'));
		const you = await render(PeerAvatar, { initials: 'MP' });
		expect(getComputedStyle(you.container.firstElementChild!).backgroundColor).toBe(tokenBackground('nm-surface-2'));
	});

	it('shows a teammate’s message in their colour, who it went to and when', async () => {
		await render(TeammateMessage, { who: sari, time: '09:14', to: 'Koda', text: 'Can we add a Makanan section?' });
		expect(getComputedStyle(page.getByText('Sari Wulandari').element()).color).toBe(tokenColor('nm-peer-1'));
		await expect.element(page.getByText('to Koda')).toBeVisible();
		expect(getComputedStyle(page.getByText('Can we add a Makanan section?').element()).backgroundColor).toBe(tokenBackground('nm-peer-1-bg'));
	});

	it('says who is typing, who joined and what they can do', async () => {
		await render(Typing, { who: sari });
		await expect.element(page.getByText('Sari is typing')).toBeVisible();
		await render(JoinedNote, { name: 'Sari', role: 'prompt', publisher: 'Mario' });
		await expect.element(page.getByRole('separator', { name: 'Sari joined · can prompt · Mario publishes' })).toBeInTheDocument();
	});

	it('shows who is here, online in mint', async () => {
		const { container } = await render(Presence, { people: [mario, { ...sari, online: true }] });
		expect(container.querySelectorAll('[data-avatar]')).toHaveLength(2);
		expect(getComputedStyle(container.querySelector('[data-online]')!).backgroundColor).toBe(tokenBackground('nm-accent'));
	});
});

describe('Asks in line', () => {
	it('queues asks so changes never collide, and lets you reorder', async () => {
		const onfirst = vi.fn();
		await render(RequestQueue, {
			asks: [
				{ text: 'Make the Pre-order button green', who: mario, state: 'running', meta: 'Mario · running · 00:21' },
				{ text: 'Add a Makanan section with nasi uduk', who: sari, state: 'next', meta: 'Sari · next' },
			],
			onfirst,
		});
		await expect.element(page.getByText('2 asks in line')).toBeVisible();
		await expect.element(page.getByText('Koda is building')).toBeVisible();
		await expect.element(page.getByText('Next', { exact: true })).toBeVisible();
		await page.getByRole('button', { name: "Do Sari's first" }).click();
		expect(onfirst).toHaveBeenCalledWith(1);
	});

	it('asks the publisher to publish a teammate’s change', async () => {
		const onpublish = vi.fn();
		await render(PublishRequest, { who: sari, meta: 'Makanan section · 2 files · built and tested in the preview', onpublish, onreview: () => {} });
		await expect.element(page.getByText("Sari's change is ready to publish")).toBeVisible();
		await page.getByRole('button', { name: 'Publish' }).click();
		expect(onpublish).toHaveBeenCalledOnce();
	});
});

describe('Roles and invites', () => {
	it('picks what an invited person can do', async () => {
		const onchange = vi.fn();
		await render(RoleMenu, { value: 'build', onchange });
		await expect.element(page.getByRole('radio', { name: /Can build/ })).toBeChecked();
		await page.getByRole('radio', { name: /Can prompt/ }).click();
		expect(onchange).toHaveBeenCalledWith('prompt');
	});

	it('invites by email with the project and one clear button', async () => {
		await render(BuildInvite, {
			from: 'Mario Prasetyo',
			project: 'warung-kopi-menu',
			detail: 'Menu and pre-order page · live · deploy #2',
			role: 'prompt',
			href: 'https://nomi.app/invite/abc',
			sentTo: 'sari.w@gmail.com',
			expires: 'in 7 days',
		});
		const title = page.getByRole('heading', { name: 'Mario Prasetyo invited you to build warung-kopi-menu' });
		expect(getComputedStyle(title.element()).fontSize).toBe('20px');
		await expect.element(page.getByRole('link', { name: 'Join and build' })).toHaveAttribute('href', 'https://nomi.app/invite/abc');
		await expect.element(page.getByText('Sent to sari.w@gmail.com · expires in 7 days · not you? ignore this email')).toBeVisible();
	});
});

describe('Live sessions', () => {
	it('shows a teammate’s cursor and who is where, with follow', async () => {
		await render(PeerCursor, { name: 'Sari', peer: 1 });
		expect(getComputedStyle(page.getByText('Sari', { exact: true }).element()).backgroundColor).toBe(tokenBackground('nm-peer-1'));
		const onfollow = vi.fn();
		await render(PresenceMenu, {
			people: [
				{ ...sari, where: 'Viewing Code · +page.svelte · active now' },
				{ ...mario, where: 'you · Preview', you: true },
			],
			onfollow,
		});
		await expect.element(page.getByText('In this project · 2', { exact: false })).toBeVisible();
		expect(page.getByRole('button', { name: 'Follow' }).elements()).toHaveLength(1);
		await page.getByRole('button', { name: 'Follow' }).click();
		expect(onfollow).toHaveBeenCalledWith(0);
	});

	it('says whom you follow and stops', async () => {
		const onstop = vi.fn();
		await render(FollowBar, { name: 'Sari', peer: 1, where: 'Code · +page.svelte', onstop });
		const bar = page.getByRole('status');
		expect(getComputedStyle(bar.element()).backgroundColor).toBe(tokenBackground('nm-peer-1'));
		await expect.element(bar).toHaveTextContent('Following Sari · Code · +page.svelte');
		await page.getByRole('button', { name: 'Stop following' }).click();
		expect(onstop).toHaveBeenCalledOnce();
	});
});
