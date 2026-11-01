/**
 * Mid-deploy, a page from the new build can ask the old server for an app file it doesn't have
 * yet. SvelteKit answers 404 with max-age=0, which Cloudflare raises to its browser cache TTL
 * (4 hours), so the browser keeps the 404 long after the deploy finished and that page can't
 * load. `no-store` passes through Cloudflare as it is.
 */
export function noStoreMissingAppFiles(req, res) {
	if (!req.url?.startsWith('/_app/')) return;
	const writeHead = res.writeHead;
	res.writeHead = function (status, ...rest) {
		if (status === 404) this.setHeader('cache-control', 'no-store');
		return writeHead.call(this, status, ...rest);
	};
}
