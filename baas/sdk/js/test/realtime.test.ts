import { afterEach, describe, expect, it, vi } from 'vitest';
import { RealtimeClient, RECONNECT_BASE_MS, type Change } from '../src/realtime.js';

class FakeSocket {
  static instances: FakeSocket[] = [];
  readyState = 0;
  sent: Record<string, unknown>[] = [];
  onopen: (() => void) | null = null;
  onmessage: ((event: { data: string }) => void) | null = null;
  onclose: (() => void) | null = null;

  constructor(readonly url: string) {
    FakeSocket.instances.push(this);
  }
  send(data: string) {
    this.sent.push(JSON.parse(data));
  }
  close() {
    this.readyState = 3;
  }
  open() {
    this.readyState = 1;
    this.onopen?.();
  }
  receive(message: unknown) {
    this.onmessage?.({ data: JSON.stringify(message) });
  }
  drop() {
    this.readyState = 3;
    this.onclose?.();
  }
}

const flush = () => new Promise((resolve) => setTimeout(resolve, 0));

function realtime() {
  return new RealtimeClient('https://todo.api.nomi.test', 'anon', async () => 'token-1', {
    WebSocket: FakeSocket as unknown as typeof WebSocket
  });
}

afterEach(() => {
  FakeSocket.instances = [];
  vi.useRealTimers();
});

describe('realtime', () => {
  it('connects on first listener, authenticates and subscribes', async () => {
    const changes: Change[] = [];
    realtime().on('task', (change) => changes.push(change));
    const socket = FakeSocket.instances[0]!;
    expect(socket.url).toBe('wss://todo.api.nomi.test/realtime?apikey=anon');

    socket.open();
    await flush();
    expect(socket.sent).toEqual([
      { type: 'auth', token: 'token-1' },
      { type: 'subscribe', table: 'task' }
    ]);

    socket.receive({ type: 'change', table: 'task', event: 'insert', row: { id: 1 }, key: { id: 1 } });
    socket.receive({ type: 'change', table: 'other', event: 'insert', row: { id: 2 }, key: { id: 2 } });
    expect(changes).toEqual([{ event: 'insert', table: 'task', row: { id: 1 }, key: { id: 1 } }]);
  });

  it('filters by event and stops listening when unsubscribed', async () => {
    const client = realtime();
    const deletes: Change[] = [];
    const stop = client.on('task', (change) => deletes.push(change), { event: 'delete' });
    const socket = FakeSocket.instances[0]!;
    socket.open();
    await flush();

    socket.receive({ type: 'change', table: 'task', event: 'insert', row: {}, key: { id: 1 } });
    socket.receive({ type: 'change', table: 'task', event: 'delete', key: { id: 1 } });
    expect(deletes.map((change) => change.event)).toEqual(['delete']);

    stop();
    expect(socket.readyState).toBe(3);
  });

  it('reconnects after a drop and tells listeners to resync', async () => {
    vi.useFakeTimers();
    const changes: Change[] = [];
    realtime().on('task', (change) => changes.push(change));
    FakeSocket.instances[0]!.open();
    await vi.runOnlyPendingTimersAsync();

    FakeSocket.instances[0]!.drop();
    await vi.advanceTimersByTimeAsync(RECONNECT_BASE_MS);
    const second = FakeSocket.instances[1]!;
    second.open();
    await vi.runOnlyPendingTimersAsync();

    expect(changes).toEqual([{ event: 'resync', table: 'task', row: null, key: null }]);
    expect(second.sent).toContainEqual({ type: 'subscribe', table: 'task' });
  });

  it('passes a server resync to the table it names', async () => {
    const changes: Change[] = [];
    realtime().on('task', (change) => changes.push(change));
    FakeSocket.instances[0]!.open();
    await flush();
    FakeSocket.instances[0]!.receive({ type: 'resync', table: 'task' });
    expect(changes).toEqual([{ event: 'resync', table: 'task', row: null, key: null }]);
  });
});
