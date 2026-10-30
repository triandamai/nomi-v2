/** The design-system gallery: on in development and CI (PUBLIC_DS_GALLERY=1), off in production. */
export function galleryEnabled(value: string | undefined): boolean {
	return value === '1' || value === 'true';
}
