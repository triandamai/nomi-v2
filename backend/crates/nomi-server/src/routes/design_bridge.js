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

	function tell(type, detail) {
		var message = { source: FROM_PREVIEW, type: type, page: page };
		for (var key in detail) message[key] = detail[key];
		window.parent.postMessage(message, '*');
	}

	function owner(target) {
		return target && target.closest ? target.closest('[' + ATTRIBUTE + ']') : null;
	}

	function describe(element) {
		var box = element.getBoundingClientRect();
		return {
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
			tell('select', describe(element));
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
		}
	});

	tell('ready', {});
})();
