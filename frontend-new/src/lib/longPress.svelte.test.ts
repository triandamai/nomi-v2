import { describe, expect, it, vi } from 'vitest';
import { longPress } from './longPress';

const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));
const press = (node: Element, type: string, x = 10, y = 10) => node.dispatchEvent(new PointerEvent(type, { bubbles: true, clientX: x, clientY: y }));

describe('a long press', () => {
	it('fires after half a second held, and the click that ends it doesn’t also act', async () => {
		const node = document.body.appendChild(document.createElement('div'));
		const onpress = vi.fn();
		const onclick = vi.fn();
		node.addEventListener('click', onclick);
		longPress(node, onpress);
		press(node, 'pointerdown');
		await wait(550);
		press(node, 'pointerup');
		node.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }));
		expect([onpress.mock.calls.length, onclick.mock.calls.length]).toEqual([1, 0]);
		node.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }));
		expect(onclick).toHaveBeenCalledOnce();
		node.remove();
	});

	it('isn’t one when let go early or moved (a scroll), and the context menu opens it too', async () => {
		const node = document.body.appendChild(document.createElement('div'));
		const onpress = vi.fn();
		longPress(node, onpress);
		press(node, 'pointerdown');
		await wait(200);
		press(node, 'pointerup');
		press(node, 'pointerdown');
		press(node, 'pointermove', 10, 60);
		await wait(550);
		expect(onpress).not.toHaveBeenCalled();
		const menu = new MouseEvent('contextmenu', { bubbles: true, cancelable: true });
		node.dispatchEvent(menu);
		expect([onpress.mock.calls.length, menu.defaultPrevented]).toEqual([1, true]);
		node.remove();
	});
});
