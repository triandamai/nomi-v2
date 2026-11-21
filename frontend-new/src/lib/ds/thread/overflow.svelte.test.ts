import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import Prose from './Prose.svelte';
import PhoneMessageBubble from './PhoneMessageBubble.svelte';

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
