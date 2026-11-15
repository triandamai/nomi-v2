import type { RequestHandler } from './$types';
import { forwardOrder } from '../orderProxy';

/** Starts the payment (POST /api/billing/checkout): the answer holds the invoice to send the person to. */
export const POST: RequestHandler = ({ cookies, fetch, request }) => forwardOrder(fetch, cookies, request, '/api/billing/checkout');
