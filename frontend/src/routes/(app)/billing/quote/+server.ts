import type { RequestHandler } from './$types';
import { forwardOrder } from '../orderProxy';

/** What an order would cost (POST /api/billing/quote), for the checkout's summary. */
export const POST: RequestHandler = ({ cookies, fetch, request }) => forwardOrder(fetch, cookies, request, '/api/billing/quote');
