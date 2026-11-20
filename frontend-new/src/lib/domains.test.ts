import { describe, expect, it } from 'vitest';
import { appHref, crossDomainRedirect, domainsFrom, landingHome, landingRoute, onLanding } from './domains';

describe('domainsFrom', () => {
	it('reads both hosts, treating empty as unset', () => {
		expect(domainsFrom({ PUBLIC_LANDING_HOST: 'nomi.build', PUBLIC_APP_HOST: 'app.nomi.build' })).toEqual({ landing: 'nomi.build', app: 'app.nomi.build' });
		expect(domainsFrom({ PUBLIC_LANDING_HOST: '' })).toEqual({ landing: undefined, app: undefined });
	});
});

describe('appHref and landingHome', () => {
	const split = { landing: 'nomi.build', app: 'app.nomi.build' };
	it('point the landing domain at the app domain, and home at its root', () => {
		expect(appHref('/sign-in', new URL('https://nomi.build/'), split)).toBe('https://app.nomi.build/sign-in');
		expect(landingHome('nomi.build', split)).toBe('/');
	});

	it('stay relative on one domain', () => {
		expect(appHref('/sign-in', new URL('http://localhost:5175/welcome'), {})).toBe('/sign-in');
		expect(landingHome('localhost', {})).toBe('/welcome');
	});
});

const domains = { landing: 'nomi.build', app: 'app.nomi.build' };
const at = (href: string) => {
	const url = new URL(href);
	return { hostname: url.hostname, pathname: url.pathname, search: url.search, protocol: url.protocol };
};

describe('landingRoute', () => {
	it('shows the landing page at the landing domain root', () => {
		expect(landingRoute(at('https://nomi.build/'), domains)).toBe('/welcome');
	});

	it('leaves the app domain and other paths alone', () => {
		expect(landingRoute(at('https://app.nomi.build/'), domains)).toBeUndefined();
		expect(landingRoute(at('https://nomi.build/chat'), domains)).toBeUndefined();
	});

	it('does nothing with one domain (dev, previews)', () => {
		expect(landingRoute(at('http://localhost:5175/'), {})).toBeUndefined();
		expect(landingRoute(at('https://nomi.build/'), { landing: 'nomi.build' })).toBeUndefined();
	});
});

describe('onLanding', () => {
	it('is true only for the landing host when both are set', () => {
		expect(onLanding('nomi.build', domains)).toBe(true);
		expect(onLanding('app.nomi.build', domains)).toBe(false);
		expect(onLanding('nomi.build', {})).toBe(false);
	});
});

describe('crossDomainRedirect', () => {
	it('sends app pages on the landing domain to the app domain, keeping the query', () => {
		expect(crossDomainRedirect(at('https://nomi.build/sign-in?redirect_to=%2Fchat'), domains)).toBe('https://app.nomi.build/sign-in?redirect_to=%2Fchat');
		expect(crossDomainRedirect(at('https://nomi.build/projects/p1'), domains)).toBe('https://app.nomi.build/projects/p1');
	});

	it('serves the landing root, built files, static files and page data on the landing domain', () => {
		for (const href of ['https://nomi.build/', 'https://nomi.build/_app/immutable/start.js', 'https://nomi.build/health', 'https://nomi.build/favicon.png', 'https://nomi.build/__data.json']) {
			expect(crossDomainRedirect(at(href), domains)).toBeNull();
		}
	});

	it('sends /welcome to the landing root on either domain', () => {
		expect(crossDomainRedirect(at('https://nomi.build/welcome'), domains)).toBe('/');
		expect(crossDomainRedirect(at('https://app.nomi.build/welcome'), domains)).toBe('https://nomi.build/');
	});

	it('leaves app pages on the app domain alone', () => {
		for (const href of ['https://app.nomi.build/', 'https://app.nomi.build/chat']) expect(crossDomainRedirect(at(href), domains)).toBeNull();
	});

	it('does nothing with one domain', () => {
		expect(crossDomainRedirect(at('http://localhost:5175/welcome'), {})).toBeNull();
	});
});
