import { base } from '$app/paths';

// A link inside the docs or the sheets. The static copy (npm run build:gallery) is plain .html
// files reached relative to wherever it's hosted; everywhere else it's the route itself.
export const docHref = (path: string, hash?: string) => `${__DS_STATIC__ ? `${base}${path}.html` : path}${hash ? `#${hash}` : ''}`;
