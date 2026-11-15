// Where this tab goes once its sign-in ends. Closing the account signs every device out, this one
// too: the activity feed's sign-out can arrive before the close's own answer, and both have to
// land on the same page ("Your account is closed"), not on a bare sign in.

let target: string | null = null;

/** Set while something ending the sign-in on purpose is under way; null once it didn't. */
export function leaveTo(path: string | null): void {
	target = path && path.startsWith('/') && !path.startsWith('//') ? path : null;
}

/** Where a sign-out from elsewhere takes this tab. */
export function signedOutTarget(): string {
	return target ?? '/sign-in';
}
