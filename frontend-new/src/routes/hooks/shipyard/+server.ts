import { forwardShipyardHook } from '$lib/server/shipyardHook';
import type { RequestHandler } from './$types';

export const POST: RequestHandler = ({ request, fetch }) => forwardShipyardHook(request, fetch);
