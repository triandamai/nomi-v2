import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import Toggle from './inputs/Toggle.svelte';
import Toast from './overlays/Toast.svelte';
import TooltipFixture from './overlays/TooltipFixture.svelte';
import MenuFixture from './overlays/MenuFixture.svelte';
import Thought from './chat/Thought.svelte';
import ActivityBlock from './conversation/ActivityBlock.svelte';
import ProjectRow from './lists/ProjectRow.svelte';
import DraftPlanCard from './chat/DraftPlanCard.svelte';
import SidebarFixture from './chrome/SidebarFixture.svelte';
import BuildPlanCard from './mobile/BuildPlanCard.svelte';
import Tag from './status/Tag.svelte';
import ChoiceOption from './chat/ChoiceOption.svelte';
import BuildPlan from './build/BuildPlan.svelte';
import { createRawSnippet } from 'svelte';
import { userEvent } from 'vitest/browser';
import { tokenColor } from './test-utils';

// Values read off the pen file (node ids in each test) after the 1:1 audit of the Nomi sheet.
// Each one was different in the first build.

const style = (element: Element) => getComputedStyle(element);
const box = (element: Element) => element.getBoundingClientRect();

describe('matches the pen', () => {
	it('Toggle (D1S92S / CAJtO): the knob sits 2px in from the track', async () => {
		await render(Toggle, { label: 'On', checked: true });
		const track = page.getByRole('switch').element();
		const knob = track.firstElementChild!;
		expect(box(track).right - box(knob).right).toBe(2);
	});

	it('Toast (iDGqw): the close is a bare 14px icon, so the title keeps its width', async () => {
		await render(Toast, { kind: 'error', title: 'Deploy failed', ondismiss: () => {} });
		const close = page.getByRole('button', { name: /Dismiss/ }).element();
		const taken = box(close).width + parseFloat(style(close).marginLeft) + parseFloat(style(close).marginRight);
		expect(Math.round(taken)).toBe(14);
	});

	it('Tooltip (EC8NV): flat, no shadow', async () => {
		await render(TooltipFixture);
		await userEvent.hover(page.getByRole('button').first());
		await expect.element(page.getByRole('tooltip')).toBeVisible();
		expect(style(page.getByRole('tooltip').element()).boxShadow).toBe('none');
	});

	it('Menu (y7iGfG): the title sits straight in the 4px padding, dividers 2px from the items', async () => {
		await render(MenuFixture, { onselect: () => {} });
		await page.getByRole('button', { name: 'Project actions' }).click();
		const title = page.getByText('salah-daily-tracker').element();
		expect(style(title).paddingTop).toBe('0px');
		expect(style(title).paddingLeft).toBe('0px');
		expect(style(page.getByRole('menu').element().querySelector('hr')!).marginTop).toBe('0px');
	});

	it('Thought (zTG0e): italic', async () => {
		await render(Thought, { text: 'Leaving last year out.' });
		expect(style(page.getByText('Leaving last year out.').element()).fontStyle).toBe('italic');
	});

	it('Activity Block (zFhiY): the list has no extra vertical padding', async () => {
		const { container } = await render(ActivityBlock, { title: 'Koda changed 2 files', files: [{ path: 'a.ts' }, { path: 'b.ts' }] });
		expect(style(container.querySelector('ul')!).paddingTop).toBe('0px');
	});

	it('Project Row (c3HSpk): a long URL wraps instead of being cut', async () => {
		await render(ProjectRow, { href: '#', name: 'warung', tone: 'koda', updated: '2m', url: 'warung-kopi-menu-3f9a12.trian.space', status: { kind: 'live', label: 'Live' } });
		expect(style(page.getByText('warung-kopi-menu-3f9a12.trian.space').element()).whiteSpace).not.toBe('nowrap');
	});

	it('Draft Plan (SgiLe): "Add a step" has an 8px gap', async () => {
		await render(DraftPlanCard, { props: { intro: 'Plan', steps: [{ label: 'One', owner: { agent: 'dana', name: 'Dana' } }], estimate: '~1 min', onadd: () => {} } });
		expect(style(page.getByRole('button', { name: 'Add a step' }).element()).columnGap).toBe('8px');
	});

	it('Sidebar (CKZsb): rows 10px in with text-2 icons; collapse is a bare 16px icon', async () => {
		await render(SidebarFixture, { oncollapse: () => {} });
		const row = page.getByRole('link', { name: /Inbox/ }).element();
		expect(style(row).paddingLeft).toBe('10px');
		expect(style(row.querySelector('span')!).color).toBe(tokenColor('nm-text-2'));
		const collapse = page.getByRole('button', { name: 'Collapse sidebar' }).element();
		expect(Math.round(box(collapse).width)).toBe(16);
	});

	it('Mobile / Build Plan (uYQxq): strokes sit inside the frame and lines round to whole pixels', async () => {
		const { container } = await render(BuildPlanCard, {
			project: 'p',
			summary: 'Menu and pre-order page',
			steps: ['Set up the project'],
			outcome: 'A live link.',
			cost: 'About 5 min',
			onbuild: () => {},
			onaskfirst: () => {},
			onkeep: () => {},
		});
		const card = container.querySelector('article')!;
		expect(box(card.firstElementChild!).top - box(card).top).toBe(16);
		expect(style(page.getByText('Set up the project').element()).lineHeight).toBe('20px');
	});

	it('Tag (a5VCNY): 2px above and below the label inside its stroke, 18 tall', async () => {
		await render(Tag, { kind: 'go', label: 'Recommended' });
		expect(box(page.getByText('Recommended').element()).height).toBe(18);
	});

	it('Plan option (gG2Es, Uk9gE): 10px in from its stroke; a field sits 8px under the title row', async () => {
		await render(ChoiceOption, { type: 'radio', name: 'run', checked: true, title: 'Build it', detail: 'Koda builds it.', tag: 'Recommended', key: 1, onchange: () => {} });
		expect(box(document.querySelector('[data-choice]')!).height).toBe(56);
		document.body.innerHTML = '';
		const children = createRawSnippet(() => ({ render: () => '<input class="h-8 w-full" />' }));
		await render(ChoiceOption, { type: 'radio', name: 'keep', checked: false, title: 'Keep planning', key: 3, onchange: () => {}, children });
		expect(box(document.querySelector('[data-choice]')!).height).toBe(80);
	});

	it('Chat / Build Plan (NmfBg): header 48 and footer 50 tall, their dividers inside', async () => {
		const { container } = await render(BuildPlan, {
			project: 'p',
			summary: 'Menu page',
			steps: ['One'],
			outcome: 'A link.',
			options: [{ title: 'Build it', recommended: true }],
			cost: 'About 5 min',
			onrun: () => {},
		});
		const card = container.querySelector('section')!;
		expect(box(card.querySelector('header')!).bottom - box(card).top).toBe(48);
		expect(box(card).bottom - box(card.querySelector('footer')!).top).toBe(50);
	});
});
