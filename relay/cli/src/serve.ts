// `nomi-relay functions serve`: runs nomi/functions/ on the local machine with Deno, the way Relay
// runs them, at http://localhost:<port>/functions/<name>. There's no gateway here, so the bearer
// token's claims are read without checking its signature: for local development only.

/** The Deno program that serves `entries` (function name → file path). */
export function serveEntry(entries: Record<string, string>, sdk: string): string {
  const loaders = Object.entries(entries)
    .map(([name, path]) => `  '${name}': () => import(${JSON.stringify(`file://${path}`)}),`)
    .join('\n');
  return `// Written by nomi-relay functions serve. Don't edit it.
import { createFunctionContext } from '${sdk}/functions';

type Handler = (req: Request, ctx: unknown) => Response | Promise<Response>;
const loaders: Record<string, () => Promise<{ default: Handler }>> = {
${loaders}
};
const env = Deno.env.toObject();

/** The bearer token's claims, unverified (Relay's gateway checks them in the hosted backend). */
function claims(req: Request): string {
  const token = req.headers.get('authorization')?.replace(/^Bearer\\s+/i, '') ?? '';
  const payload = token.split('.')[1];
  if (!payload) return btoa(JSON.stringify({ role: 'anon' }));
  return payload;
}

Deno.serve({ port: Number(env.PORT ?? 8787) }, async (req) => {
  const url = new URL(req.url);
  const match = url.pathname.match(/^\\/functions\\/([a-z0-9-]+)(\\/.*)?$/);
  const load = match ? loaders[match[1]] : undefined;
  if (!match || !load) return Response.json({ code: 'NO_FUNCTION', message: 'No function at ' + url.pathname }, { status: 404 });
  const headers = new Headers(req.headers);
  headers.set('x-nomi-claims', claims(req));
  const request = new Request(req, { headers });
  try {
    const handler = (await load()).default;
    return await handler(request, createFunctionContext(request, env));
  } catch (error) {
    console.error(error);
    return Response.json({ code: 'FUNCTION_ERROR', message: String(error) }, { status: 500 });
  }
});
`;
}
