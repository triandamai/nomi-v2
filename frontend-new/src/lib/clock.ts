import { getContext, setContext } from 'svelte';

// The app shell's clock: what time it is and where the person is, for the top bar's date and
// Home's greeting. The layout starts it at the minute the page was loaded (so the server's render
// and the browser's agree), ticks it, and switches to the browser's own time zone when the person
// never saved one.

export type Clock = { readonly now: Date; readonly timeZone: string | undefined };

const KEY = Symbol('clock');

export const setClock = (clock: Clock) => setContext(KEY, clock);

export const clock = (): Clock => getContext<Clock | undefined>(KEY) ?? { now: new Date(), timeZone: undefined };
