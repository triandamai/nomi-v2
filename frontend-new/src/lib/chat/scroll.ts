/**
 * Forwards touch and wheel scroll gestures from an unscrollable region (like the composer)
 * to the target scroller (like the chat thread), preventing the browser from scrolling
 * ancestor/parent containers or the document window.
 */
export function forwardScrollTo(node: HTMLElement, getTarget: () => HTMLElement | null | undefined) {
	let startY = 0;
	let startX = 0;
	let tracking = false;

	function ontouchstart(event: TouchEvent) {
		if (event.touches.length !== 1) {
			tracking = false;
			return;
		}
		startY = event.touches[0].clientY;
		startX = event.touches[0].clientX;
		tracking = true;
	}

	function ontouchmove(event: TouchEvent) {
		if (!tracking || event.touches.length !== 1) return;
		const touch = event.touches[0];
		const deltaY = startY - touch.clientY;
		const deltaX = startX - touch.clientX;

		// Only handle vertical swipes
		if (Math.abs(deltaY) > Math.abs(deltaX)) {
			const target = event.target as HTMLElement | null;
			// If inside an overflowing textarea that can scroll in this direction, let it scroll
			if (target instanceof HTMLTextAreaElement) {
				const canScrollUp = deltaY < 0 && target.scrollTop > 0;
				const canScrollDown = deltaY > 0 && target.scrollTop + target.clientHeight < target.scrollHeight;
				if (canScrollUp || canScrollDown) return;
			}

			const scroller = getTarget();
			if (scroller) {
				scroller.scrollTop += deltaY;
			}
			startY = touch.clientY;
			startX = touch.clientX;
			if (event.cancelable) {
				event.preventDefault();
			}
		}
	}

	function ontouchend() {
		tracking = false;
	}

	function onwheel(event: WheelEvent) {
		const target = event.target as HTMLElement | null;
		if (target instanceof HTMLTextAreaElement) {
			const canScrollUp = event.deltaY < 0 && target.scrollTop > 0;
			const canScrollDown = event.deltaY > 0 && target.scrollTop + target.clientHeight < target.scrollHeight;
			if (canScrollUp || canScrollDown) return;
		}
		const scroller = getTarget();
		if (scroller) {
			scroller.scrollTop += event.deltaY;
			if (event.cancelable) {
				event.preventDefault();
			}
		}
	}

	node.addEventListener('touchstart', ontouchstart, { passive: true });
	node.addEventListener('touchmove', ontouchmove, { passive: false });
	node.addEventListener('touchend', ontouchend, { passive: true });
	node.addEventListener('touchcancel', ontouchend, { passive: true });
	node.addEventListener('wheel', onwheel, { passive: false });

	return {
		destroy() {
			node.removeEventListener('touchstart', ontouchstart);
			node.removeEventListener('touchmove', ontouchmove);
			node.removeEventListener('touchend', ontouchend);
			node.removeEventListener('touchcancel', ontouchend);
			node.removeEventListener('wheel', onwheel);
		}
	};
}
