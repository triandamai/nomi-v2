// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { flushSync } from 'svelte';

vi.mock('$lib/notifications.svelte', () => ({ notificationArrived: () => {} }));
vi.mock('$lib/snackbars.svelte', () => ({ showSnackbar: () => {} }));

const { useUserActivity } = await import('$lib/userActivityFeed.svelte');

class FakeSocket {
	static all: FakeSocket[] = [];
	static OPEN = 1;
	readyState = 0;
	closed = false;
	constructor(public url: string) {
		FakeSocket.all.push(this);
	}
	addEventListener() {}
	close() {
		this.closed = true;
	}
}

beforeEach(() => {
	FakeSocket.all = [];
	vi.stubGlobal('WebSocket', FakeSocket);
});

afterEach(() => vi.unstubAllGlobals());

describe("the app's live feed", () => {
	it('keeps its socket when the layout reloads its data (the same device)', () => {
		let data = $state({ deviceId: 'd1', unread: 0 });
		const stop = $effect.root(() => useUserActivity(() => data.deviceId));
		flushSync();
		expect(FakeSocket.all).toHaveLength(1);

		data = { deviceId: 'd1', unread: 3 };
		flushSync();
		expect(FakeSocket.all).toHaveLength(1);
		expect(FakeSocket.all[0].closed).toBe(false);
		stop();
	});

	it('reconnects for another device, and closes when the app goes', () => {
		let data = $state({ deviceId: 'd1' });
		const stop = $effect.root(() => useUserActivity(() => data.deviceId));
		flushSync();
		data = { deviceId: 'd2' };
		flushSync();
		expect(FakeSocket.all).toHaveLength(2);
		expect(FakeSocket.all[0].closed).toBe(true);
		stop();
		expect(FakeSocket.all[1].closed).toBe(true);
	});
});
