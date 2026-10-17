import { m } from '$lib/paraglide/messages';

// The tagline's rolling verb: "A small crew of agents that [remembers] for you." TaglineWheel
// rolls through these in order and wraps back to the first.
//
// To add a verb: append it here (after the last one) and add its message to messages/, then
// list it in the plain `auth_line` too, which is what reduced motion and screen readers get.
export const TAGLINE_VERBS: readonly (() => string)[] = [m.tagline_remembers, m.tagline_builds, m.tagline_works];
