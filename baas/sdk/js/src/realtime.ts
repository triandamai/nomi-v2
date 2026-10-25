/** First reconnect waits this long; each failure doubles it, up to the max. */
export const RECONNECT_BASE_MS = 500;
export const RECONNECT_MAX_MS = 10_000;
const RECONNECT_FACTOR = 2;

export type ChangeEvent = 'insert' | 'update' | 'delete';

/**
 * What a listener gets. `resync` means changes may have been missed (a reconnect, or a burst over
 * the plan's limit): refetch instead of applying a diff.
 */
export type Change<Row = Record<string, unknown>> =
  | { event: ChangeEvent; table: string; row: Row | null; key: Record<string, unknown> }
  | { event: 'resync'; table: string; row: null; key: null };

export type ChangeListener<Row> = (change: Change<Row>) => void;

export interface RealtimeOptions {
  /** A WebSocket constructor, for runtimes without a global one. */
  WebSocket?: typeof WebSocket;
}

type ServerMessage =
  | { type: 'change'; table: string; event: ChangeEvent; row?: Record<string, unknown> | null; key: Record<string, unknown> }
  | { type: 'resync'; table?: string }
  | { type: 'error'; message: string };

interface Subscription {
  table: string;
  event?: ChangeEvent;
  listener: ChangeListener<never>;
}

/**
 * Live changes to the tables listed in `backend.json`'s `realtime`. Reached as `nomi.realtime`.
 * Connects on the first `on()` and disconnects when the last listener goes.
 */
export class RealtimeClient {
  private socket: WebSocket | null = null;
  private readonly subscriptions = new Set<Subscription>();
  private attempts = 0;
  private everConnected = false;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private readonly WebSocketImpl: typeof WebSocket | undefined;

  constructor(
    private readonly baseUrl: string,
    private readonly key: string,
    private readonly token: () => Promise<string>,
    options: RealtimeOptions = {}
  ) {
    this.WebSocketImpl = options.WebSocket ?? globalThis.WebSocket;
  }

  /** Calls `listener` for each change to `table` the signed-in user may see. Returns a function that stops it. */
  on<Row = Record<string, unknown>>(table: string, listener: ChangeListener<Row>, options: { event?: ChangeEvent } = {}): () => void {
    const subscription: Subscription = { table, event: options.event, listener: listener as ChangeListener<never> };
    const isNewTable = !this.hasTable(table);
    this.subscriptions.add(subscription);
    if (!this.socket) this.connect();
    else if (isNewTable) this.sendMessage({ type: 'subscribe', table });
    return () => {
      this.subscriptions.delete(subscription);
      if (!this.hasTable(table)) this.sendMessage({ type: 'unsubscribe', table });
      if (this.subscriptions.size === 0) this.disconnect();
    };
  }

  /** Sends the new access token after a sign-in, refresh or sign-out, so what's sent follows the user. */
  async updateToken(): Promise<void> {
    if (this.socket?.readyState === OPEN) this.sendMessage({ type: 'auth', token: await this.token() });
  }

  disconnect(): void {
    if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
    this.reconnectTimer = null;
    const socket = this.socket;
    this.socket = null;
    this.everConnected = false;
    this.attempts = 0;
    socket?.close();
  }

  private hasTable(table: string): boolean {
    for (const subscription of this.subscriptions) if (subscription.table === table) return true;
    return false;
  }

  private tables(): string[] {
    return [...new Set([...this.subscriptions].map((subscription) => subscription.table))];
  }

  private connect(): void {
    if (!this.WebSocketImpl) throw new Error('nomi-baas: no WebSocket in this runtime; pass realtime.WebSocket to createClient');
    const url = new URL(this.baseUrl.replace(/^http/, 'ws') + '/realtime');
    url.searchParams.set('apikey', this.key);
    const socket = new this.WebSocketImpl(url.toString());
    this.socket = socket;

    socket.onopen = async () => {
      if (this.socket !== socket) return;
      this.sendMessage({ type: 'auth', token: await this.token() });
      for (const table of this.tables()) this.sendMessage({ type: 'subscribe', table });
      if (this.everConnected) this.dispatchResync();
      this.everConnected = true;
      this.attempts = 0;
    };
    socket.onmessage = (event) => {
      if (this.socket !== socket) return;
      let message: ServerMessage;
      try {
        message = JSON.parse(String(event.data)) as ServerMessage;
      } catch {
        return;
      }
      if (message.type === 'change') {
        this.dispatch({ event: message.event, table: message.table, row: message.row ?? null, key: message.key });
      } else if (message.type === 'resync') {
        this.dispatchResync(message.table);
      }
    };
    socket.onclose = () => {
      if (this.socket !== socket) return;
      this.socket = null;
      if (this.subscriptions.size > 0) this.scheduleReconnect();
    };
  }

  private scheduleReconnect(): void {
    const delay = Math.min(RECONNECT_MAX_MS, RECONNECT_BASE_MS * RECONNECT_FACTOR ** this.attempts);
    this.attempts += 1;
    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      if (this.subscriptions.size > 0 && !this.socket) this.connect();
    }, delay);
  }

  private dispatch(change: Change): void {
    for (const subscription of this.subscriptions) {
      if (subscription.table !== change.table) continue;
      if (subscription.event && change.event !== 'resync' && subscription.event !== change.event) continue;
      subscription.listener(change as never);
    }
  }

  private dispatchResync(table?: string): void {
    const tables = table ? [table] : this.tables();
    for (const name of tables) this.dispatch({ event: 'resync', table: name, row: null, key: null });
  }

  private sendMessage(message: Record<string, unknown>): void {
    if (this.socket?.readyState === OPEN) this.socket.send(JSON.stringify(message));
  }
}

/** `WebSocket.OPEN`, without reading it off a global that may not exist. */
const OPEN = 1;
