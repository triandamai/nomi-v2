/** This build's version: the release tag (`1.2.3`) in a release build, `git describe` otherwise. */
export const APP_VERSION: string = typeof __APP_VERSION__ === 'string' ? __APP_VERSION__ : 'dev';

/** Sent to the backend with every call, so it can tell which frontend is talking to it. */
export const CLIENT_VERSION_HEADER = 'X-Client-Version';
