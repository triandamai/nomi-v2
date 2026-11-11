import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import { createRawSnippet } from 'svelte';
import { CalendarCheck, CircleCheck, Database, FileText, Globe, SearchX, X } from '@lucide/svelte';
import ProjectEmpty from './ProjectEmpty.svelte';
import FirstProject from './FirstProject.svelte';
import LoadError from './LoadError.svelte';
import GithubConnectDialog from './GithubConnectDialog.svelte';
import RepoPickerDialog from './RepoPickerDialog.svelte';
import ImportReview from './ImportReview.svelte';
import ProjectDetailPanel from './ProjectDetailPanel.svelte';
import AskField from './AskField.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Design System / Projects → Cards & overlays: the empty and error states (F4dhk9, eI37L, HJAkR,
// vkXCA), the import from GitHub (s3mLi, tzhsm, TcRMc) and the project detail (I1bGE, oZ6Le).

const style = (element: Element) => getComputedStyle(element);
const box = (element: Element) => element.getBoundingClientRect();

// Wide enough for the grid's two columns and the widest dialog, as on a desktop.
beforeEach(async () => {
	await page.viewport(1280, 800);
});

describe('ProjectEmpty (F4dhk9, eI37L)', () => {
	it('is 240 tall, centred, with a 300 wide line and the one thing to do', async () => {
		const onclick = vi.fn();
		await render(ProjectEmpty, {
			icon: SearchX,
			title: 'No projects match “kasir”',
			body: "Search looks at names, descriptions and URLs in Trian's space.",
			action: { label: 'Clear search', icon: X, onclick },
		});
		const title = page.getByText('No projects match “kasir”').element();
		const empty = title.parentElement!;
		expect(box(empty).height).toBe(240);
		expect(style(empty).outlineColor).toBe(tokenColor('nm-border'));
		const body = page.getByText(/Search looks at names/).element();
		expect(box(body).width).toBe(300);
		expect(style(body).textAlign).toBe('center');
		await page.getByRole('button', { name: 'Clear search' }).click();
		expect(onclick).toHaveBeenCalledOnce();
	});

	it('can be only words, for an empty tab', async () => {
		await render(ProjectEmpty, { icon: CircleCheck, title: 'Nothing needs attention', body: 'When a deploy fails or Koda is waiting on you, the project shows up here.' });
		await expect.element(page.getByRole('button')).not.toBeInTheDocument();
	});
});

describe('FirstProject (HJAkR)', () => {
	it('is Koda saying what to do, then starters 40 tall, 2 apart', async () => {
		const onpick = vi.fn();
		await render(FirstProject, {
			starters: [
				{ icon: CalendarCheck, label: 'Booking page for my barbershop', meta: 'web app · database', onpick },
				{ icon: FileText, label: 'One-page portfolio with a contact form', meta: 'site · no database', onpick },
			],
		});
		await expect.element(page.getByRole('heading', { name: 'No projects yet' })).toBeInTheDocument();
		const list = page.getByRole('list', { name: 'Ideas to start from' }).element();
		const rows = page.getByRole('button').elements();
		expect(box(rows[0]).height).toBe(40);
		expect(box(rows[1]).top - box(rows[0]).bottom).toBe(2);
		expect(box(list.parentElement!).width).toBe(820);
		await page.getByRole('button', { name: /Booking page for my barbershop/ }).click();
		expect(onpick).toHaveBeenCalledWith('Booking page for my barbershop');
	});
});

describe('LoadError (vkXCA)', () => {
	it('says only the list failed, quotes why, and retries', async () => {
		const onretry = vi.fn();
		await render(LoadError, { reason: '503 from api.nomi.app · 09:14', onretry });
		const alert = page.getByRole('alert').element();
		expect(box(alert).height).toBe(59);
		expect(style(alert).backgroundColor).toBe(tokenBackground('nm-danger-bg'));
		await expect.element(page.getByText('Your apps are still live; only this list failed. 503 from api.nomi.app · 09:14')).toBeInTheDocument();
		await page.getByRole('button', { name: 'Retry' }).click();
		expect(onretry).toHaveBeenCalledOnce();
	});
});

describe('GithubConnectDialog (s3mLi)', () => {
	it('is 420 × 277 and says what Nomi can and can’t do', async () => {
		const oncontinue = vi.fn();
		await render(GithubConnectDialog, { inline: true, oncontinue });
		const panel = page.getByTestId('dialog-panel').element();
		expect(box(panel).height).toBe(277);
		expect(page.getByRole('list', { name: 'What Nomi can do' }).element().children).toHaveLength(3);
		await page.getByRole('button', { name: 'Continue on GitHub' }).click();
		expect(oncontinue).toHaveBeenCalledOnce();
	});
});

describe('RepoPickerDialog (tzhsm)', () => {
	const repos = [
		{ id: 'r1', name: 'mario/kasir-toko', meta: 'SvelteKit · private · updated 3d ago' },
		{ id: 'r2', name: 'mario/portfolio-2024', meta: 'Astro · public · 2mo ago' },
		{ id: 'r3', name: 'kopikita/web', meta: 'not shared with Nomi', grantHref: 'https://github.com/settings/installations' },
	];

	it('is 460 wide (383 with the pen’s four repos): search, the repos 44 tall, a repo Nomi can’t see offers access', async () => {
		await render(RepoPickerDialog, { inline: true, workspace: "Trian's space", owner: 'mario', repos, value: 'r1', onimport: () => {} });
		const panel = page.getByTestId('dialog-panel').element();
		expect(box(panel).width).toBe(460);
		expect(box(panel).height).toBe(383 - 46);
		await expect.element(page.getByRole('searchbox', { name: 'Search mario’s repos' })).toBeInTheDocument();
		const chosen = page.getByRole('radio', { name: /mario\/kasir-toko/ }).element();
		expect(box(chosen).height).toBe(44);
		expect(chosen.getAttribute('aria-checked')).toBe('true');
		expect(page.getByRole('radio').elements()).toHaveLength(2);
		await expect.element(page.getByRole('link', { name: /Grant access/ })).toHaveAttribute('href', 'https://github.com/settings/installations');
		await expect.element(page.getByRole('button', { name: 'Import kasir-toko' })).toBeInTheDocument();
	});

	it('imports the chosen repo', async () => {
		const onimport = vi.fn();
		await render(RepoPickerDialog, { inline: true, workspace: "Trian's space", owner: 'mario', repos, value: 'r1', onimport });
		await page.getByRole('radio', { name: /portfolio-2024/ }).click();
		await page.getByRole('button', { name: 'Import portfolio-2024' }).click();
		expect(onimport).toHaveBeenCalledWith('r2');
	});
});

describe('ImportReview (TcRMc)', () => {
	it('is 380 × 569: what Koda found, what it sets up, the missing secrets', async () => {
		const ondeploy = vi.fn();
		await render(ImportReview, {
			repo: 'kasir-toko',
			meta: '48 files · 00:41',
			found: { app: 'SvelteKit 2 · builds with npm run build', tests: '12 tests · all pass', data: 'Postgres via DATABASE_URL', secrets: "MIDTRANS_KEY and ADMIN_PIN aren't in the repo" },
			setup: [
				{ icon: Database, text: 'An empty Postgres database, migrated from /migrations' },
				{ icon: Globe, text: 'kasir-toko-3e9a.trian.space' },
			],
			secrets: ['MIDTRANS_KEY', 'ADMIN_PIN'],
			ondeploy,
			onlater: () => {},
		});
		const review = page.getByRole('region', { name: 'Koda read kasir-toko' }).element();
		expect(box(review).width).toBe(380);
		expect(style(review).backgroundColor).toBe(tokenBackground('nm-bg'));
		expect(style(page.getByText("MIDTRANS_KEY and ADMIN_PIN aren't in the repo").element()).color).toBe(tokenColor('nm-warning'));
		const secret = page.getByRole('textbox', { name: 'MIDTRANS_KEY' });
		expect(box(secret.element().parentElement!).height).toBe(34);
		await secret.fill('sk-123');
		await page.getByRole('button', { name: 'Set up & deploy' }).click();
		expect(ondeploy).toHaveBeenCalledWith({ MIDTRANS_KEY: 'sk-123', ADMIN_PIN: '' });
	});
});

describe('ProjectDetailPanel (I1bGE)', () => {
	it('is 380 × 641: header, what’s in it, its chats', async () => {
		const onclose = vi.fn();
		const onask = vi.fn();
		const preview = createRawSnippet(() => ({ render: () => '<div>Dhuhr</div>' }));
		await render(ProjectDetailPanel, {
			name: 'salah-daily-tracker',
			tone: 'nomi',
			live: 'live · deploy #14 · 2d ago',
			preview,
			workspaceHref: '/projects/p1',
			url: 'salah-daily-tracker-0ab8f8.trian.space',
			parts: { code: '23 files · main@a3f9c21', url: '…0ab8f8.trian.space', data: 'localStorage only · no server', deploys: '14 · last built in 41s', members: 'Trian (owner), you' },
			onexport: () => {},
			chats: [
				{ title: 'Build a daily salah tracker', age: '2d', href: '/chats/c1' },
				{ title: 'Add Hijri offset setting', age: '2d', href: '/chats/c2' },
			],
			onask,
			onclose,
		});
		const panel = page.getByRole('complementary', { name: 'salah-daily-tracker' }).element();
		expect(box(panel).width).toBe(380);
		expect(box(panel).height).toBe(641);
		await expect.element(page.getByRole('link', { name: /Open workspace/ })).toHaveAttribute('href', '/projects/p1');
		expect(box(page.getByRole('link', { name: /Open workspace/ }).element()).height).toBe(32);
		await expect.element(page.getByRole('link', { name: /Visit/ })).toHaveAttribute('href', 'https://salah-daily-tracker-0ab8f8.trian.space');
		const parts = page.getByRole('term').elements();
		expect(parts.map((part) => part.textContent?.trim())).toEqual(['Code', 'URL', 'Data', 'Deploys', 'Members']);
		expect(box(parts[0].parentElement!).height).toBe(31);
		await expect.element(page.getByRole('link', { name: /Add Hijri offset setting/ })).toHaveAttribute('href', '/chats/c2');
		await page.getByRole('button', { name: 'Ask Koda to change something' }).click();
		expect(onask).toHaveBeenCalledOnce();
		await page.getByRole('button', { name: 'Close' }).click();
		expect(onclose).toHaveBeenCalledOnce();
	});
});

describe('AskField (oZ6Le)', () => {
	it('is 340 × 78, accent while it holds a question, and sends it', async () => {
		const onask = vi.fn();
		await render(AskField, { project: 'salah-daily-tracker', value: 'Add a Maghrib reminder 10 minutes before', onask });
		const form = page.getByRole('form', { name: 'Ask Koda about salah-daily-tracker' }).element();
		expect(box(form).width).toBe(340);
		expect(box(form).height).toBe(78);
		expect(style(form).outlineColor).toBe(tokenColor('nm-accent'));
		await expect.element(page.getByText('new chat in salah-daily-tracker')).toBeInTheDocument();
		await page.getByRole('button', { name: 'Send' }).click();
		expect(onask).toHaveBeenCalledWith('Add a Maghrib reminder 10 minutes before');
	});
});
