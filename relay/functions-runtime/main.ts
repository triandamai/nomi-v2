// Starts the functions runtime: `RELAY_FUNCTIONS_SECRET` (required, the same one saved in Relay's
// settings), `PORT` (8000) and `RELAY_FUNCTIONS_DATA` (where builds are kept, /tmp/relay-functions).
import { createRuntime } from './runtime.ts';

const DEFAULT_PORT = 8000;
const INIT_TIMEOUT_MS = 10_000;
const IDLE_MS = 10 * 60_000;

const secret = Deno.env.get('RELAY_FUNCTIONS_SECRET') ?? '';
if (secret.length < 32) {
  console.error('RELAY_FUNCTIONS_SECRET must be set (32 characters or more)');
  Deno.exit(1);
}
const dataDir = Deno.env.get('RELAY_FUNCTIONS_DATA') ?? '/tmp/relay-functions';
await Deno.mkdir(dataDir, { recursive: true });
const runtime = createRuntime({ secret, dataDir, initTimeoutMs: INIT_TIMEOUT_MS, idleMs: IDLE_MS });
Deno.serve({ port: Number(Deno.env.get('PORT') ?? DEFAULT_PORT), hostname: '0.0.0.0' }, runtime.handle);
