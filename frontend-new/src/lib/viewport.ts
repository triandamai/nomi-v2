/**
 * Keeps CSS custom properties (--viewport-height and --viewport-top) synchronized
 * with the browser's visual viewport.
 *
 * On iOS Safari / WebKit, dynamic viewport units like 100dvh do not shrink when the
 * virtual keyboard opens. Instead, Safari shrinks `visualViewport` and pans or
 * scrolls the window, causing fixed/dvh containers to have overflow scroll height
 * and push headers off-screen.
 *
 * By binding container heights to `--viewport-height` and container top to `--viewport-top`,
 * and keeping `window.scrollY` at 0, mobile web layouts fit the visible screen exactly
 * above the on-screen keyboard without causing parent overflow scroll.
 */
export function initViewport(): () => void {
	if (typeof window === 'undefined') return () => {};

	function isAppLayout(): boolean {
		return Boolean(
			document.querySelector('[data-app-shell], [data-phone-workspace], [data-workspace-shell]') ||
			document.activeElement?.closest('[data-app-shell], [data-phone-workspace], [data-workspace-shell], [style*="--viewport-height"]')
		);
	}

	function resetWindowScroll() {
		if (window.scrollY !== 0 || window.scrollX !== 0) {
			window.scrollTo(0, 0);
		}
	}

	function update() {
		const vv = window.visualViewport;
		if (vv) {
			const height = Math.round(vv.height);
			const top = Math.round(vv.offsetTop);
			document.documentElement.style.setProperty('--viewport-height', `${height}px`);
			document.documentElement.style.setProperty('--viewport-top', `${top}px`);
		} else {
			document.documentElement.style.setProperty('--viewport-height', `${window.innerHeight}px`);
			document.documentElement.style.setProperty('--viewport-top', '0px');
		}

		if (isAppLayout()) {
			resetWindowScroll();
		}
	}

	update();

	const vv = window.visualViewport;
	if (vv) {
		vv.addEventListener('resize', update);
		vv.addEventListener('scroll', update);
	}
	window.addEventListener('resize', update);

	const onWindowScroll = () => {
		if (isAppLayout()) {
			resetWindowScroll();
		}
	};
	window.addEventListener('scroll', onWindowScroll);

	const onOrientationChange = () => {
		setTimeout(update, 100);
	};
	window.addEventListener('orientationchange', onOrientationChange);

	const onFocus = (event: Event) => {
		const target = event.target as HTMLElement | null;
		const isEditable =
			target instanceof HTMLInputElement ||
			target instanceof HTMLTextAreaElement ||
			Boolean(target?.isContentEditable);
		if (isEditable) {
			if (isAppLayout()) {
				resetWindowScroll();
			} else {
				const isDocumentScrollable = document.documentElement.scrollHeight > window.innerHeight + 50;
				if (
					!isDocumentScrollable &&
					window.visualViewport &&
					window.visualViewport.height < window.innerHeight
				) {
					resetWindowScroll();
				}
			}
		}
		requestAnimationFrame(update);
		setTimeout(update, 50);
		setTimeout(update, 250);
	};
	window.addEventListener('focusin', onFocus);
	window.addEventListener('focusout', onFocus);

	return () => {
		if (vv) {
			vv.removeEventListener('resize', update);
			vv.removeEventListener('scroll', update);
		}
		window.removeEventListener('resize', update);
		window.removeEventListener('scroll', onWindowScroll);
		window.removeEventListener('orientationchange', onOrientationChange);
		window.removeEventListener('focusin', onFocus);
		window.removeEventListener('focusout', onFocus);
	};
}
