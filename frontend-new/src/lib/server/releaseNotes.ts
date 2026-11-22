import { APP_VERSION } from '$lib/version';
import { BAKED_RELEASE_NOTES, releasesFor, type Release, type ReleaseNotes } from '$lib/releaseNotes';
import { renderMarkdown } from './markdown';

/** A release ready to show: its notes as sanitised HTML for Prose. */
export type RenderedRelease = Omit<Release, 'body'> & { html: string };

let rendered: Promise<RenderedRelease[]> | undefined;

/** The notes of this build's environment, rendered once: they're fixed for the build's lifetime. */
export function loadReleaseNotes(notes: ReleaseNotes | null = BAKED_RELEASE_NOTES, version: string = APP_VERSION): Promise<RenderedRelease[]> {
	const render = () => Promise.all(releasesFor(notes, version).map(async ({ body, ...release }) => ({ ...release, html: await renderMarkdown(body) })));
	if (notes !== BAKED_RELEASE_NOTES || version !== APP_VERSION) return render();
	rendered ??= render();
	return rendered;
}
