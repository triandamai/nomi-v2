/**
 * Whether the person is on a touch screen (a phone or tablet), where focusing a text field opens
 * the on-screen keyboard: after sending, the composer lets go of focus there so the keyboard
 * closes and the reply is in view, instead of grabbing focus again as it does with a mouse.
 */
export function touchScreen(): boolean {
	return typeof matchMedia === 'function' && matchMedia('(pointer: coarse)').matches;
}
