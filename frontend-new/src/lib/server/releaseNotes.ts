import { APP_VERSION } from '$lib/version';
import { BAKED_RELEASE_NOTES, releasesFor, type Release, type ReleaseNotes } from '$lib/releaseNotes';
import { renderMarkdown } from './markdown';

/** A release ready to show: its notes as sanitised HTML for Prose. */
export type RenderedRelease = Omit<Release, 'body'> & { html: string };

let rendered: Promise<RenderedRelease[]> | undefined;

/** Strips PR links and Full Changelog links so release notes keep only what changed and who made it. */
export function cleanReleaseNoteBody(body: string): string {
	return body
		.replace(/\r\n/g, '\n')
		.split('\n')
		.filter((line) => !/^\*\*Full Changelog\*\*:/.test(line.trim()))
		.map((line) =>
			line
				.replace(/\s+in\s+(?:\[#\d+\]\(https?:\/\/\S+\/pull\/\d+\)|https?:\/\/\S+\/pull\/\d+|#\d+)\s*$/i, '')
				.replace(/\s+\((?:\[#\d+\]\(https?:\/\/\S+\/pull\/\d+\)|https?:\/\/\S+\/pull\/\d+|#\d+)\)\s*$/i, '')
				.trimEnd(),
		)
		.join('\n')
		.replace(/\n{3,}/g, '\n\n')
		.trim();
}

/** The notes of this build's environment, rendered once: they're fixed for the build's lifetime. */
export function loadReleaseNotes(notes: ReleaseNotes | null = BAKED_RELEASE_NOTES, version: string = APP_VERSION): Promise<RenderedRelease[]> {
	const render = () => Promise.all(releasesFor(notes, version).map(async ({ body, ...release }) => ({ ...release, html: await renderMarkdown(cleanReleaseNoteBody(body)) })));
	if (notes !== BAKED_RELEASE_NOTES || version !== APP_VERSION) return render();
	rendered ??= render();
	return rendered;
}
