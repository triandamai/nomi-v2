import { onNavigate } from '$app/navigation';

/**
 * Moving between pages animates the page area (named `page` in the layout) with the View
 * Transitions API: the old page fades back, the new one rises in (see material3.css). The
 * sidebar stays put. Browsers without the API, a reduced-motion setting, and changes within the
 * same page (search, paging) just switch as before. Call from a layout's script.
 */
export function enablePageTransitions() {
	onNavigate((navigation) => {
		if (!document.startViewTransition) return;
		if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) return;
		if (navigation.from?.url.pathname === navigation.to?.url.pathname) return;
		const back = navigation.delta !== undefined && navigation.delta < 0;
		document.documentElement.dataset.navDirection = back ? 'back' : 'forward';
		return new Promise((resolve) => {
			const transition = document.startViewTransition(async () => {
				resolve();
				await navigation.complete;
			});
			transition.finished.finally(() => delete document.documentElement.dataset.navDirection);
		});
	});
}
