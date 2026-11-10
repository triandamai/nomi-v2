/** This build's version: the release tag (`1.2.3`) in a release build, `git describe` otherwise. */
export const APP_VERSION: string = typeof __APP_VERSION__ === 'string' ? __APP_VERSION__ : 'dev';

/** The version as people see it (the account menu's What's new): "v0.0.75"; a dev build's as it is. */
export const VERSION_LABEL: string = /^\d/.test(APP_VERSION) ? `v${APP_VERSION}` : APP_VERSION;

/** Sent to the backend with every call, so it can tell which frontend is talking to it. */
export const CLIENT_VERSION_HEADER = 'X-Client-Version';
