// Added to every page of a design preview shown in the Design Studio. It tells the Studio which
// element (by its data-design-id) the person points at, and outlines the one the Studio names.
// The page runs in a sandbox with its own origin, so this is its only line to the Studio.
(function () {
	var ATTRIBUTE = 'data-design-id';
	var FROM_PREVIEW = 'nomi-design-preview';
	var FROM_STUDIO = 'nomi-design-studio';
	var TEXT_CHARS = 120;
	var page = window.__nomiDesignPage || '';
	var selecting = false;
	var outline = null;
	var MAX_MARKS = 200;
	var marks = [];
	var marked = [];

	function tell(type, detail) {
		var message = { source: FROM_PREVIEW, type: type, page: page };
		for (var key in detail) message[key] = detail[key];
		window.parent.postMessage(message, '*');
	}

	function owner(target) {
		return target && target.closest ? target.closest('[' + ATTRIBUTE + ']') : null;
	}

	function within(value) {
		return Math.min(1, Math.max(0, value));
	}

	// Where in the element the click landed: x across and y down, each from 0 to 1.
	function pointIn(box, event) {
		if (!event || !box.width || !box.height) return null;
		return { x: within((event.clientX - box.left) / box.width), y: within((event.clientY - box.top) / box.height) };
	}

	function describe(element, event) {
		var box = element.getBoundingClientRect();
		return {
			point: pointIn(box, event),
			id: element.getAttribute(ATTRIBUTE),
			tag: element.tagName.toLowerCase(),
			text: (element.textContent || '').replace(/\s+/g, ' ').trim().slice(0, TEXT_CHARS),
			rect: { x: box.left, y: box.top, width: box.width, height: box.height },
		};
	}

	function show(element) {
		if (!element) {
			if (outline) outline.style.display = 'none';
			return;
		}
		if (!outline) {
			outline = document.createElement('div');
			outline.setAttribute('aria-hidden', 'true');
			outline.style.cssText = 'position:fixed;z-index:2147483647;pointer-events:none;border:2px solid #2563eb;background:rgba(37,99,235,.12);border-radius:4px;box-sizing:border-box';
			document.documentElement.appendChild(outline);
		}
		var box = element.getBoundingClientRect();
		outline.style.display = 'block';
		outline.style.left = box.left + 'px';
		outline.style.top = box.top + 'px';
		outline.style.width = box.width + 'px';
		outline.style.height = box.height + 'px';
	}

	function find(id) {
		var all = document.querySelectorAll('[' + ATTRIBUTE + ']');
		for (var i = 0; i < all.length; i++) if (all[i].getAttribute(ATTRIBUTE) === id) return all[i];
		return null;
	}

	// A numbered mark for each piece of feedback the Studio names, placed on its element.
	function place() {
		for (var i = 0; i < marks.length; i++) {
			var element = find(marked[i].id);
			if (!element) {
				marks[i].style.display = 'none';
				continue;
			}
			var box = element.getBoundingClientRect();
			var pin = marked[i].pin || { x: 1, y: 0 };
			marks[i].style.display = 'flex';
			marks[i].style.left = box.left + box.width * pin.x + 'px';
			marks[i].style.top = box.top + box.height * pin.y + 'px';
		}
	}

	function annotate(items) {
		for (var i = 0; i < marks.length; i++) marks[i].remove();
		marks = [];
		marked = [];
		for (var j = 0; j < items.length && j < MAX_MARKS; j++) {
			var item = items[j];
			if (!item || typeof item.id !== 'string' || typeof item.number !== 'number') continue;
			var pin = item.pin && typeof item.pin.x === 'number' && typeof item.pin.y === 'number' ? { x: within(item.pin.x), y: within(item.pin.y) } : null;
			var mark = document.createElement('div');
			mark.setAttribute('aria-hidden', 'true');
			mark.textContent = String(Math.floor(item.number));
			mark.style.cssText =
				'position:fixed;z-index:2147483646;pointer-events:none;transform:translate(-50%,-50%);min-width:22px;height:22px;padding:0 6px;box-sizing:border-box;align-items:center;justify-content:center;border-radius:11px;background:#b3261e;color:#fff;font:600 12px/1 system-ui,sans-serif;box-shadow:0 1px 3px rgba(0,0,0,.4)';
			document.documentElement.appendChild(mark);
			marks.push(mark);
			marked.push({ id: item.id, pin: pin });
		}
		place();
	}

	window.addEventListener('scroll', place, true);
	window.addEventListener('resize', place);

	document.addEventListener(
		'mouseover',
		function (event) {
			if (selecting) show(owner(event.target));
		},
		true,
	);

	document.addEventListener(
		'click',
		function (event) {
			if (!selecting) return;
			event.preventDefault();
			event.stopPropagation();
			var element = owner(event.target);
			if (!element) return;
			show(element);
			tell('select', describe(element, event));
		},
		true,
	);

	window.addEventListener('message', function (event) {
		var message = event.data;
		if (event.source !== window.parent || !message || message.source !== FROM_STUDIO) return;
		if (message.type === 'mode') {
			selecting = message.select === true;
			document.documentElement.style.cursor = selecting ? 'crosshair' : '';
			if (!selecting) show(null);
		} else if (message.type === 'highlight') {
			var element = typeof message.id === 'string' ? find(message.id) : null;
			if (element && element.scrollIntoView) element.scrollIntoView({ block: 'center' });
			show(element);
		} else if (message.type === 'annotate') {
			annotate(Array.isArray(message.items) ? message.items : []);
		}
	});

	tell('ready', {});
})();
