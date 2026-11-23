import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import Prose from './Prose.svelte';
import PhoneMessageBubble from './PhoneMessageBubble.svelte';
import { forwardScrollTo } from '$lib/chat/scroll';

// A phone's thread is 358 wide (390 less its 16 sides) and scrolls up and down only: nothing in a
// message may push it wider, or the whole thread slides sideways.
const PHONE_COLUMN = 358;
const LONG_URL = 'https://warung-kopi-budi.nomi.build/menu/pesanan?id=8f3c2a91b7e64d0fa5c1e9b2d4f6a8c0&ref=whatsapp-broadcast-jakarta-selatan';
const LONG_WORD = 'Rp249.000'.repeat(12);

function inPhoneColumn(container: HTMLElement) {
	container.style.width = `${PHONE_COLUMN}px`;
	container.style.overflowY = 'auto';
	return container;
}

describe('a phone thread never scrolls sideways', () => {
	it('wraps a long link and an unbroken word in a reply', async () => {
		const { container } = await render(Prose, { phone: true, html: `<p>Buka <a href="${LONG_URL}">${LONG_URL}</a> lalu cek ${LONG_WORD}</p>` });
		const column = inPhoneColumn(container);
		expect(column.scrollWidth).toBeLessThanOrEqual(column.clientWidth);
	});

	it('fits a wide picture to the column', async () => {
		const { container } = await render(Prose, { phone: true, html: '<p><img alt="Menu" width="1200" height="600" src="data:image/gif;base64,R0lGODlhAQABAAAAACw="></p>' });
		const column = inPhoneColumn(container);
		expect(column.scrollWidth).toBeLessThanOrEqual(column.clientWidth);
	});

	it('scrolls a long code line inside its own block, not the thread', async () => {
		const { container } = await render(Prose, { phone: true, html: `<pre><code>const url = "${LONG_URL}";</code></pre>` });
		const column = inPhoneColumn(container);
		expect(column.scrollWidth).toBeLessThanOrEqual(column.clientWidth);
	});

	it('wraps a long link in your own message', async () => {
		const { container } = await render(PhoneMessageBubble, { text: `Tolong cek ${LONG_URL} ya` });
		const column = inPhoneColumn(container);
		expect(column.scrollWidth).toBeLessThanOrEqual(column.clientWidth);
	});
});

describe('a phone chat scroller contains vertical scrolling', () => {
	it('confines scrolling to the chat scroller without overflowing the parent workspace', () => {
		const parent = document.createElement('div');
		parent.style.height = '600px';
		parent.style.display = 'flex';
		parent.style.flexDirection = 'column';
		parent.style.overflow = 'hidden';

		const chatWrapper = document.createElement('div');
		chatWrapper.style.minHeight = '0';
		chatWrapper.style.flex = '1 1 0%';
		chatWrapper.style.display = 'flex';
		chatWrapper.style.flexDirection = 'column';
		chatWrapper.style.overflow = 'hidden';

		const scroller = document.createElement('div');
		scroller.style.minHeight = '0';
		scroller.style.flex = '1 1 0%';
		scroller.style.overflowY = 'auto';
		scroller.style.overscrollBehaviorY = 'contain';

		const tallContent = document.createElement('div');
		tallContent.style.height = '2000px';

		scroller.appendChild(tallContent);
		chatWrapper.appendChild(scroller);
		parent.appendChild(chatWrapper);
		document.body.appendChild(parent);

		expect(parent.scrollHeight).toBeLessThanOrEqual(parent.clientHeight);
		expect(scroller.scrollHeight).toBeGreaterThan(scroller.clientHeight);

		document.body.removeChild(parent);
	});
});

describe('forwardScrollTo swiping from compose view', () => {
	it('forwards vertical swipe gestures from compose view to chat scroller and prevents parent scroll', () => {
		const composerNode = document.createElement('div');
		const scroller = document.createElement('div');
		scroller.style.height = '100px';
		scroller.style.overflowY = 'auto';
		const inner = document.createElement('div');
		inner.style.height = '1000px';
		scroller.appendChild(inner);
		document.body.appendChild(scroller);
		scroller.scrollTop = 100;

		const cleanup = forwardScrollTo(composerNode, () => scroller);

		const touchStart = new TouchEvent('touchstart', {
			touches: [new Touch({ identifier: 1, target: composerNode, clientX: 50, clientY: 200 })],
			bubbles: true,
		});
		composerNode.dispatchEvent(touchStart);

		const touchMove = new TouchEvent('touchmove', {
			touches: [new Touch({ identifier: 1, target: composerNode, clientX: 50, clientY: 150 })],
			cancelable: true,
			bubbles: true,
		});
		composerNode.dispatchEvent(touchMove);

		expect(scroller.scrollTop).toBe(150);
		expect(touchMove.defaultPrevented).toBe(true);

		cleanup.destroy();
		document.body.removeChild(scroller);
	});

	it('forwards wheel scroll events from compose view to chat scroller', () => {
		const composerNode = document.createElement('div');
		const scroller = document.createElement('div');
		scroller.style.height = '100px';
		scroller.style.overflowY = 'auto';
		const inner = document.createElement('div');
		inner.style.height = '1000px';
		scroller.appendChild(inner);
		document.body.appendChild(scroller);
		scroller.scrollTop = 100;

		const cleanup = forwardScrollTo(composerNode, () => scroller);

		const wheel = new WheelEvent('wheel', {
			deltaY: 40,
			cancelable: true,
			bubbles: true,
		});
		composerNode.dispatchEvent(wheel);

		expect(scroller.scrollTop).toBe(140);
		expect(wheel.defaultPrevented).toBe(true);

		cleanup.destroy();
		document.body.removeChild(scroller);
	});
});
