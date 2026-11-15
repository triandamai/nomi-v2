import { forwardSiteReport } from '$lib/server/siteHook';
import type { RequestHandler } from './$types';

export const POST: RequestHandler = ({ params, request, fetch }) => forwardSiteReport(params.projectId, request, fetch);
