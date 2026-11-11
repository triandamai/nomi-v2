import { pageAction, type PageActionResult } from '$lib/pageAction';

// The Inbox page's actions, called from its rows and menus and from the top bar's Mark all read
// (which the layout draws).

/** The quick reminder's field, for the top bar's "New reminder" to put focus in. */
export const QUICK_REMINDER = 'quick-reminder';

export type InboxActionResult = PageActionResult;

export const inboxAction = (name: string, fields: Record<string, string> = {}, options: { keepalive?: boolean } = {}) => pageAction('/inbox', name, fields, options);
