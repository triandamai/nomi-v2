// What a link to Nomi shows when it's shared (WhatsApp, Telegram, Slack, X, iMessage…): the
// description, Open Graph and a large Twitter card, as the old app had them. Crawlers need absolute
// addresses, so they're built from the deployment's own origin (adapter-node's ORIGIN, or the
// request's host), never a fixed one: staging links show staging's image.

/** The link-preview image in static/, drawn by scripts/brand/render.mjs. */
export const SHARE_IMAGE = { path: '/og-image.png', width: 1200, height: 630, type: 'image/png' } as const;

/**
 * The browser bar's colour: the page token (--nm-bg) in light and dark. A meta tag can't read a
 * CSS variable, so the values are copied here; seo.test.ts holds them to tokens.css.
 */
export const THEME_COLOR = { light: '#F6F6F4', dark: '#0A0A0B' } as const;

/** Open Graph's locale for each of the app's languages. */
const OG_LOCALE: Record<string, string> = { en: 'en_US', id: 'id_ID' };

export type MetaTag = { name?: string; property?: string; content: string };

export type ShareInput = {
	/** The page's origin, e.g. https://nomi.build. */
	origin: string;
	pathname: string;
	title: string;
	description: string;
	siteName: string;
	imageAlt: string;
	locale: string;
};

/** A path on this deployment as an absolute address. */
export const absoluteUrl = (origin: string, path: string) => `${origin.replace(/\/+$/, '')}${path}`;

/** Every meta tag a shared link reads, in the order they go in the head. */
export function shareMeta({ origin, pathname, title, description, siteName, imageAlt, locale }: ShareInput): MetaTag[] {
	const image = absoluteUrl(origin, SHARE_IMAGE.path);
	return [
		{ name: 'description', content: description },
		{ property: 'og:type', content: 'website' },
		{ property: 'og:site_name', content: siteName },
		{ property: 'og:title', content: title },
		{ property: 'og:description', content: description },
		{ property: 'og:url', content: absoluteUrl(origin, pathname) },
		{ property: 'og:locale', content: OG_LOCALE[locale] ?? OG_LOCALE.en },
		{ property: 'og:image', content: image },
		{ property: 'og:image:type', content: SHARE_IMAGE.type },
		{ property: 'og:image:width', content: String(SHARE_IMAGE.width) },
		{ property: 'og:image:height', content: String(SHARE_IMAGE.height) },
		{ property: 'og:image:alt', content: imageAlt },
		{ name: 'twitter:card', content: 'summary_large_image' },
		{ name: 'twitter:title', content: title },
		{ name: 'twitter:description', content: description },
		{ name: 'twitter:image', content: image },
		{ name: 'twitter:image:alt', content: imageAlt },
	];
}
