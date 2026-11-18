// A long press, as a phone's chat uses one (Flow / Mobile Chat — Message controls, Like & dislike):
// holding a message half a second opens its sheet, as does the context menu (a right click, the
// menu key, or iOS's own long press where it fires one). A press that moves (a scroll) isn't one,
// and the tap that ends a long press doesn't also act.

const HOLD_MS = 500;
const SLOP_PX = 10;

export function longPress(node: HTMLElement, onpress: () => void) {
	let handler = onpress;
	let timer: ReturnType<typeof setTimeout> | undefined;
	let start: { x: number; y: number } | null = null;
	let fired = false;

	const cancel = () => {
		clearTimeout(timer);
		start = null;
	};
	function down(event: PointerEvent) {
		fired = false;
		start = { x: event.clientX, y: event.clientY };
		timer = setTimeout(() => {
			fired = true;
			handler();
		}, HOLD_MS);
	}
	function move(event: PointerEvent) {
		if (start && Math.hypot(event.clientX - start.x, event.clientY - start.y) > SLOP_PX) cancel();
	}
	function menu(event: MouseEvent) {
		event.preventDefault();
		if (!fired) handler();
		fired = true;
	}
	function click(event: MouseEvent) {
		if (!fired) return;
		event.preventDefault();
		event.stopPropagation();
		fired = false;
	}

	node.addEventListener('pointerdown', down);
	node.addEventListener('pointermove', move);
	node.addEventListener('pointerup', cancel);
	node.addEventListener('pointercancel', cancel);
	node.addEventListener('pointerleave', cancel);
	node.addEventListener('contextmenu', menu);
	node.addEventListener('click', click, true);
	return {
		update(next: () => void) {
			handler = next;
		},
		destroy() {
			cancel();
			node.removeEventListener('pointerdown', down);
			node.removeEventListener('pointermove', move);
			node.removeEventListener('pointerup', cancel);
			node.removeEventListener('pointercancel', cancel);
			node.removeEventListener('pointerleave', cancel);
			node.removeEventListener('contextmenu', menu);
			node.removeEventListener('click', click, true);
		},
	};
}
