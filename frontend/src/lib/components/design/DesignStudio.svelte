<script lang="ts">
	import { ArrowLeft, BadgeCheck, Hand, Maximize2, MessageSquarePlus, Monitor, MousePointer2, PanelRight, Smartphone, Tablet, ZoomIn, ZoomOut } from '@lucide/svelte';
	import { invalidateAll } from '$app/navigation';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { formatDate } from '$lib/dates';
	import Icon from '$lib/components/ds/foundations/Icon.svelte';
	import OpenPencilViewer, { type CanvasPin } from '$lib/components/design/OpenPencilViewer.svelte';
	import { buildStudioSceneModel, type StudioHitResult } from '$lib/openPencilCanvas';
	import {
		CANVAS_ARTBOARD_WIDTHS,
		DEVICE_WIDTHS,
		activeDirectionForPreview,
		annotateMessage,
		annotationsOn,
		clampZoom,
		compareWith,
		elementPinDraft,
		elementsOnPage,
		feedbackDraft,
		fitZoom,
		highlightMessage,
		isPenPreview,
		mergePenDocuments,
		modeMessage,
		penUrl,
		pickDirectionMessage,
		pickVersion,
		previewUrl,
		readPreviewMessage,
		refineMessage,
		splitCanvasPages,
		toolForKey,
		versionsNewestFirst,
		zoomLabel,
		zoomStep,
		type CanvasTool,
		type Device,
		type FeedbackScope,
		type Selection,
		type StudioApproval,
		type StudioDirection,
		type StudioDirectionSet,
		type StudioElement,
		type StudioElementKind,
		type StudioFeedback,
		type StudioPreview,
	} from '$lib/designStudio';

	// The Design Studio: a chat's design directions, design system, and multi-page previews on an
	// interactive spatial canvas or single-page frame. Previews run in sandboxed frames with an
	// opaque origin; they only tell the Studio which [data-design-id] element was pointed at.
	let {
		sessionId,
		directions = null,
		previews,
		feedback: given,
		approval: approvedBefore,
		wanted = null,
		embedded = false,
		onrefresh,
	}: {
		sessionId: string;
		directions?: StudioDirectionSet | null;
		previews: StudioPreview[];
		feedback: StudioFeedback[];
		approval: StudioApproval | null;
		/** The version a link asked for (`?version=`). */
		wanted?: string | null;
		/** Inside a project page: no way back (the chat is beside it) and it fills its pane. */
		embedded?: boolean;
		/** Loads what there is now; the page reloads itself when not given. */
		onrefresh?: () => Promise<void>;
	} = $props();
	const data = $derived({ sessionId, directions, previews, wanted });

	const ENTRY_PAGE = 'index.html';
	// The most characters in one piece of feedback (design_feedback::MAX_BODY_CHARS).
	const FEEDBACK_CHARS = 2000;
	// How much one wheel notch with Ctrl/⌘ zooms the frames canvas.
	const WHEEL_ZOOM_STEP = 0.1;
	// The least room above the design when the canvas fits it, for the frame names.
	const FIT_TOP = 32;
	// How far the comment popover stays from the stage's edges, and how big it is.
	const POPOVER_MARGIN = 16;
	const POPOVER_WIDTH = 300;
	const POPOVER_HEIGHT = 240;
	const DEVICES: { device: Device; icon: typeof Monitor; label: string }[] = [
		{ device: 'desktop', icon: Monitor, label: m.device_desktop() },
		{ device: 'tablet', icon: Tablet, label: m.studio_device_tablet() },
		{ device: 'phone', icon: Smartphone, label: m.device_phone() },
	];
	const KIND_LABELS: Record<StudioElementKind, string> = {
		section: m.studio_kind_section(),
		component: m.studio_kind_component(),
		element: m.studio_kind_element(),
	};

	// The version on screen: the one picked here, else the one the link asked for, else the newest.
	let pickedVersion = $state<number | null>(null);
	const current = $derived(pickVersion(data.previews, pickedVersion !== null ? String(pickedVersion) : data.wanted));
	const versions = $derived(versionsNewestFirst(data.previews));
	const newest = $derived(versions[0]?.version ?? null);
	const directionList = $derived(data.directions?.directions ?? []);
	const activeDirection = $derived(activeDirectionForPreview(data.directions, current));
	const splitPages = $derived(current ? splitCanvasPages(current.pages) : { systemPages: [], flowPages: [] });

	// The page the frame was sent to, and the page it says it is on (links inside it move it).
	let openedPage = $state(ENTRY_PAGE);
	let shownPage = $state(ENTRY_PAGE);
	const frameSrc = $derived(current ? previewUrl(current, openedPage) : null);
	const pageElements = $derived(current ? elementsOnPage(current, shownPage) : []);

	let device = $state<Device>('desktop');
	const frameWidth = $derived(DEVICE_WIDTHS[device] === null ? '100%' : `${DEVICE_WIDTHS[device]}px`);
	const canvasBoardWidth = $derived(CANVAS_ARTBOARD_WIDTHS[device]);

	// What a click on the canvas does: move pans, select picks, comment pins feedback (V, H, C).
	let tool = $state<CanvasTool>('select');
	const TOOLS: { tool: CanvasTool; icon: typeof Monitor; label: string; key: string }[] = [
		{ tool: 'move', icon: Hand, label: m.studio_tool_move(), key: 'H' },
		{ tool: 'select', icon: MousePointer2, label: m.studio_tool_select(), key: 'V' },
		{ tool: 'comment', icon: MessageSquarePlus, label: m.studio_tool_comment(), key: 'C' },
	];
	let selection = $state<Selection | null>(null);
	let frame = $state<HTMLIFrameElement | null>(null);
	let canvasFrames = $state<Record<string, HTMLIFrameElement | null>>({});

	type Mode = 'preview' | 'canvas' | 'feedback' | 'compare';
	let mode = $state<Mode>('canvas');
	const picking = $derived(tool !== 'move');

	// Spatial canvas state: zoom, pan, stage filter, and direct click-to-pin popover.
	let zoom = $state(1);
	let panX = $state(0);
	let panY = $state(0);
	let isPanning = $state(false);
	let panOrigin = $state<{ x: number; y: number; panX: number; panY: number } | null>(null);
	let viewportEl = $state<HTMLElement | null>(null);
	let stageEl = $state<HTMLElement | null>(null);
	let inspectorOpen = $state(false);
	let pinPopoverOpen = $state(false);
	// Where the comment was dropped, inside the stage; null puts the popover in its corner.
	let pinAt = $state<{ x: number; y: number } | null>(null);
	let pinWords = $state('');
	let pinInteraction = $state(false);

	// A version Sora drew as a .pen canvas is shown on the vector canvas; an older HTML one in frames.
	type CanvasEngine = 'openpencil' | 'frames';
	const canvasEngine = $derived<CanvasEngine>(current && !isPenPreview(current) ? 'frames' : 'openpencil');

	// The version's own .pen files, loaded from where its files are served.
	let pen = $state<{ token: string; json: string | null } | null>(null);
	$effect(() => {
		const preview = current;
		if (!preview || canvasEngine !== 'openpencil') return;
		const files = preview.pen_files ?? [];
		if (files.length === 0 || pen?.token === preview.preview_token) return;
		const token = preview.preview_token;
		void Promise.all(files.map((path) => fetch(penUrl(preview, path)).then((r) => (r.ok ? r.text() : '')).catch(() => '')))
			.then((contents) => {
				if (current?.preview_token === token) pen = { token, json: mergePenDocuments(contents) };
			});
	});
	const penJson = $derived(current && pen?.token === current.preview_token ? pen.json : null);

	const openPencilModel = $derived(
		buildStudioSceneModel({
			directions: directionList,
			preview: current,
			activeDirection,
			device,
			rawPenJson: penJson
		})
	);

	// The canvas fits what is on it whenever something new is drawn.
	let fittedFor = $state<string | null>(null);
	$effect(() => {
		const key = `${current?.preview_token ?? 'directions'}:${penJson ? 'pen' : 'scene'}:${openPencilModel.frames.length}`;
		if (canvasEngine !== 'openpencil' || !stageEl || fittedFor === key || openPencilModel.frames.length === 0) return;
		fittedFor = key;
		fitPen();
	});

	function fitPen() {
		const frames = openPencilModel.frames;
		if (!stageEl || frames.length === 0) return;
		const left = Math.min(...frames.map((f) => f.x));
		const top = Math.min(...frames.map((f) => f.y));
		const right = Math.max(...frames.map((f) => f.x + f.width));
		const bottom = Math.max(...frames.map((f) => f.y + f.height));
		const room = { w: stageEl.clientWidth || 1200, h: stageEl.clientHeight || 800 };
		zoom = Math.min(fitZoom(room.w, right - left), fitZoom(room.h, bottom - top));
		panX = (room.w - (right - left) * zoom) / 2 - left * zoom;
		panY = Math.max(FIT_TOP, (room.h - (bottom - top) * zoom) / 2) - top * zoom;
	}

	function openComment(at: { x: number; y: number } | null) {
		pinAt = at;
		pinPopoverOpen = true;
	}

	function handleOpenPencilHit(hit: StudioHitResult, at: { x: number; y: number }) {
		if (hit.target.kind === 'direction-cta' && hit.target.directionId) {
			const dir = directionList.find((d) => d.id === hit.target.directionId);
			if (dir) useDirection(dir);
			return;
		}
		if (hit.target.elementId) {
			const kind: StudioElementKind =
				hit.target.kind === 'page' || hit.target.kind === 'direction-cta'
					? 'section'
					: hit.target.kind;
			selection = {
				id: hit.target.elementId,
				kind,
				label: hit.target.label,
				page: hit.target.pagePath ?? shownPage,
				tag: 'section',
				text: hit.target.label,
				point: hit.pin
			};
			if (hit.target.pagePath) {
				shownPage = hit.target.pagePath;
			}
			if (tool === 'comment') openComment(at);
			else pinPopoverOpen = false;
		}
	}

	function pickTool(next: CanvasTool) {
		tool = next;
		if (next !== 'comment') pinPopoverOpen = false;
	}

	function keydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			clearSelection();
			return;
		}
		const next = toolForKey(event);
		if (next) pickTool(next);
	}

	// Direction exploration state (Stage 1)
	let sendingDirection = $state(false);
	let directionSent = $state(false);
	let directionError = $state('');

	// Everything said about the chat's previews: what the page loaded with, and what was added here.
	let added = $state<StudioFeedback[]>([]);
	// Feedback added here that a reload has since brought in is shown once.
	const feedback = $derived([...given, ...added.filter((a) => !given.some((g) => g.id === a.id))]);
	const history = $derived([...feedback].sort((a, b) => b.number - a.number));
	// Open feedback marked on the canvas, at the point it was dropped.
	const canvasPins = $derived<CanvasPin[]>(
		current
			? feedback
					.filter((f) => f.status === 'open' && f.element_id)
					.map((f) => ({ number: f.number, pagePath: f.page, elementId: f.element_id, x: f.pin?.x ?? 0.5, y: f.pin?.y ?? 0 }))
			: []
	);

	const marks = $derived(current ? annotationsOn(feedback, current, shownPage) : []);

	const SCOPES: { scope: FeedbackScope; label: string }[] = [
		{ scope: 'global', label: m.studio_scope_global() },
		{ scope: 'page', label: m.studio_scope_page() },
		{ scope: 'element', label: m.studio_scope_element() },
	];
	let scope = $state<FeedbackScope>('page');
	let interaction = $state(false);
	let words = $state('');
	let sending = $state(false);
	let feedbackError = $state('');
	const draft = $derived(current ? feedbackDraft({ version: current.version, scope, page: shownPage, selection, interaction, body: words }) : null);
	const pinDraft = $derived(current ? elementPinDraft(current.version, selection, pinWords, pinInteraction) : null);

	$effect(() => {
		if (frame) tell(modeMessage(picking));
		for (const pageFrame of Object.values(canvasFrames)) {
			pageFrame?.contentWindow?.postMessage(modeMessage(picking), '*');
		}
	});
	$effect(() => {
		if (frame) tell(annotateMessage(marks));
		if (current) {
			for (const [pagePath, pageFrame] of Object.entries(canvasFrames)) {
				pageFrame?.contentWindow?.postMessage(annotateMessage(annotationsOn(feedback, current, pagePath)), '*');
			}
		}
	});

	// The frame's origin is its own and unnameable, so messages are addressed to its window.
	function tell(message: object) {
		frame?.contentWindow?.postMessage(message, '*');
	}

	function tellAll(message: object) {
		tell(message);
		for (const pageFrame of Object.values(canvasFrames)) {
			pageFrame?.contentWindow?.postMessage(message, '*');
		}
	}

	function isKnownFrame(source: MessageEventSource | null): source is Window {
		if (!source) return false;
		if (frame?.contentWindow === source) return true;
		return Object.values(canvasFrames).some((f) => f?.contentWindow === source);
	}

	function heard(event: MessageEvent) {
		if (!current || !isKnownFrame(event.source)) return;
		const said = readPreviewMessage(event.data, current);
		if (!said) return;
		if (said.type === 'ready') {
			shownPage = said.page;
			event.source.postMessage(modeMessage(picking), '*');
			event.source.postMessage(annotateMessage(annotationsOn(feedback, current, said.page)), '*');
			if (selection && selection.page === said.page) event.source.postMessage(highlightMessage(selection.id), '*');
		} else {
			selection = said.selection;
			shownPage = said.selection.page;
			if (tool === 'comment') openComment(null);
		}
	}

	function openVersion(version: number) {
		pickedVersion = version;
		confirming = false;
		openedPage = ENTRY_PAGE;
		shownPage = ENTRY_PAGE;
		selection = null;
		pinPopoverOpen = false;
	}

	function openPage(path: string) {
		openedPage = path;
		shownPage = path;
	}

	function pick(element: StudioElement) {
		const targetPage = element.pages.includes(shownPage) ? shownPage : (element.pages[0] ?? shownPage);
		selection = { id: element.id, page: targetPage, kind: element.kind, label: element.label, tag: '', text: '', point: null };
		if (tool === 'comment') openComment(null);
		tellAll(highlightMessage(element.id));
	}

	function openMode(next: Mode) {
		mode = next;
		if (next === 'feedback') {
			pinPopoverOpen = false;
			if (selection) scope = 'element';
		}
	}

	function zoomIn() {
		zoom = zoomStep(zoom, 'in');
	}

	function zoomOut() {
		zoom = zoomStep(zoom, 'out');
	}

	function fitCanvas() {
		if (canvasEngine === 'openpencil') {
			fitPen();
			return;
		}
		const boardCount = Math.max(1, (current ? splitPages.flowPages.length + 1 : 0) + directionList.length);
		const totalWidth = boardCount * (canvasBoardWidth + 32);
		const vw = viewportEl?.clientWidth ?? 1200;
		zoom = fitZoom(vw, totalWidth);
		panX = 0;
		panY = 0;
	}

	function startCanvasPan(event: PointerEvent) {
		if (event.button !== 0 && event.button !== 1) return;
		const target = event.target as HTMLElement | null;
		if (target?.closest('button, a, input, textarea, select, iframe, .studio__pin-popover')) return;
		isPanning = true;
		panOrigin = { x: event.clientX, y: event.clientY, panX, panY };
	}

	function moveCanvasPan(event: PointerEvent) {
		if (!isPanning || !panOrigin) return;
		panX = panOrigin.panX + (event.clientX - panOrigin.x);
		panY = panOrigin.panY + (event.clientY - panOrigin.y);
	}

	function endCanvasPan() {
		isPanning = false;
		panOrigin = null;
	}

	function handleCanvasWheel(event: WheelEvent) {
		event.preventDefault();
		if (event.ctrlKey || event.metaKey) {
			zoom = clampZoom(zoom + (event.deltaY < 0 ? WHEEL_ZOOM_STEP : -WHEEL_ZOOM_STEP));
		} else {
			panX -= event.deltaX;
			panY -= event.deltaY;
		}
	}

	async function addFeedback(event: SubmitEvent) {
		event.preventDefault();
		if (!draft || sending) return;
		sending = true;
		feedbackError = '';
		try {
			const response = await fetch(`/chat/${data.sessionId}/design/feedback`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(draft) });
			if (!response.ok) {
				const refused = (await response.json().catch(() => null)) as { message?: string } | null;
				feedbackError = refused?.message || m.studio_feedback_failed();
				return;
			}
			added = [...added, (await response.json()) as StudioFeedback];
			words = '';
		} catch {
			feedbackError = m.studio_feedback_failed();
		} finally {
			sending = false;
		}
	}

	async function savePinFeedback(event: SubmitEvent) {
		event.preventDefault();
		if (!pinDraft || sending) return;
		sending = true;
		feedbackError = '';
		try {
			const response = await fetch(`/chat/${data.sessionId}/design/feedback`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(pinDraft) });
			if (!response.ok) {
				const refused = (await response.json().catch(() => null)) as { message?: string } | null;
				feedbackError = refused?.message || m.studio_feedback_failed();
				return;
			}
			added = [...added, (await response.json()) as StudioFeedback];
			pinWords = '';
			pinInteraction = false;
			pinPopoverOpen = false;
		} catch {
			feedbackError = m.studio_feedback_failed();
		} finally {
			sending = false;
		}
	}

	async function sendDirectionMessage(text: string) {
		if (!text || sendingDirection) return;
		sendingDirection = true;
		directionError = '';
		directionSent = false;
		try {
			const response = await fetch(`/chat/${data.sessionId}/design/refine`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ text }) });
			if (!response.ok) {
				directionError = m.studio_refine_failed();
				return;
			}
			directionSent = true;
		} catch {
			directionError = m.studio_refine_failed();
		} finally {
			sendingDirection = false;
		}
	}

	function useDirection(direction: StudioDirection) {
		void sendDirectionMessage(pickDirectionMessage(direction));
	}

	// Comparing: another version next to the one on screen, on the same page when it has it.
	let comparedVersion = $state<number | null>(null);
	const compared = $derived(current && mode === 'compare' ? compareWith(data.previews, current, comparedVersion) : null);
	const otherVersions = $derived(current ? versions.filter((v) => v.version !== current.version) : []);

	// The version the person said is the design to build. Approving is its own, confirmed step.
	let approvedHere = $state<StudioApproval | null>(null);
	const approval = $derived(approvedHere ?? approvedBefore);
	const waiting = $derived(feedback.filter((f) => f.status === 'open').length);
	let confirming = $state(false);
	let approving = $state(false);
	let approveError = $state('');

	async function approve() {
		if (!current || approving) return;
		approving = true;
		approveError = '';
		try {
			const response = await fetch(`/chat/${data.sessionId}/design/approval`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ version: current.version }) });
			if (!response.ok) {
				approveError = m.studio_approve_failed();
				return;
			}
			approvedHere = (await response.json()) as StudioApproval;
			confirming = false;
		} catch {
			approveError = m.studio_approve_failed();
		} finally {
			approving = false;
		}
	}

	// Asking Sora for a new version: a message in the chat, so the request and her answer are there.
	let ask = $state('');
	let asking = $state(false);
	let asked = $state(false);
	let askError = $state('');
	const request = $derived(refineMessage(ask, feedback, (numbers) => m.studio_refine_default({ numbers })));

	async function askSora(event?: SubmitEvent) {
		event?.preventDefault();
		if (!request || asking) return;
		asking = true;
		askError = '';
		try {
			const response = await fetch(`/chat/${data.sessionId}/design/refine`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ text: request }) });
			if (!response.ok) {
				askError = m.studio_refine_failed();
				return;
			}
			asked = true;
			ask = '';
		} catch {
			askError = m.studio_refine_failed();
		} finally {
			asking = false;
		}
	}

	// A new version is saved by Sora in the chat; this loads what there is now.
	async function refresh() {
		await (onrefresh ? onrefresh() : invalidateAll());
		added = [];
		approvedHere = null;
		pickedVersion = null;
	}

	// What a piece of feedback points at, in words.
	function where(item: StudioFeedback): string {
		const page = item.page ?? '';
		const element = item.element_id ? (current?.elements.find((e) => e.id === item.element_id)?.label ?? item.element_id) : null;
		if (item.level === 'global') return m.studio_feedback_where_global();
		const target = element ? m.studio_feedback_where_element({ element, page }) : m.studio_feedback_where_page({ page });
		return item.level === 'interaction' ? m.studio_feedback_where_interaction({ target }) : target;
	}

	// Shows what a piece of feedback is about, when this version still has it.
	function showFeedback(item: StudioFeedback) {
		if (!current || !item.page || !current.pages.some((p) => p.path === item.page)) return;
		if (item.page !== shownPage) openPage(item.page);
		const element = current.elements.find((e) => e.id === item.element_id && e.pages.includes(item.page as string));
		if (!element) return;
		selection = { id: element.id, page: item.page, kind: element.kind, label: element.label, tag: '', text: '', point: null };
		tellAll(highlightMessage(element.id));
	}

	function clearSelection() {
		selection = null;
		pinPopoverOpen = false;
		pinAt = null;
		tellAll(highlightMessage(null));
	}
</script>

<svelte:window onmessage={heard} onkeydown={keydown} />

<div class="studio" class:studio--embedded={embedded}>
	<header class="studio__bar">
		{#if !embedded}<a class="studio__back" href={`/chat/${data.sessionId}`} aria-label={m.studio_back()} title={m.studio_back()}><Icon icon={ArrowLeft} size={20} /></a>{/if}
		<div class="studio__heading">
			<h1 class="md-title-medium studio__title">{current ? current.title : m.studio_title()}</h1>
			{#if current}<p class="md-body-small studio__sub">{activeDirection?.name ?? current.direction_id} · {m.studio_version({ version: current.version })}</p>{/if}
		</div>
		{#if current}
			<div class="studio__tools">
				{#if canvasEngine === 'frames'}
					<div class="studio__devices" role="group" aria-label={m.pj_preview_device()}>
						{#each DEVICES as option (option.device)}
							<button type="button" class="studio__tool" aria-pressed={device === option.device} aria-label={option.label} title={option.label} onclick={() => (device = option.device)}>
								<Icon icon={option.icon} size={20} />
							</button>
						{/each}
					</div>
				{/if}
				{#if approval?.version === current.version}
					<span class="studio__approved md-label-large"><Icon icon={BadgeCheck} size={18} /><span>{m.studio_approved()}</span></span>
				{:else}
					<button type="button" class="studio__tool studio__tool--wide studio__tool--primary" onclick={() => (confirming = true)}>{m.studio_approve()}</button>
				{/if}
			</div>
		{/if}
	</header>

	{#if directionSent}
		<div class="studio__notice" role="status">
			<p class="md-body-small">{m.studio_refine_sent()}</p>
			<button type="button" class="studio__tool studio__tool--wide" onclick={refresh}>{m.studio_refine_refresh()}</button>
		</div>
	{/if}
	{#if directionError}
		<div class="studio__notice" role="alert">
			<p class="md-body-small studio__error">{directionError}</p>
		</div>
	{/if}

	{#if current && confirming}
		<div class="studio__confirm" role="alertdialog" aria-labelledby="studio-approve-question">
			<div class="studio__confirm-text">
				<p class="md-title-small" id="studio-approve-question">{m.studio_approve_question({ version: current.version })}</p>
				<p class="md-body-small studio__muted">{m.studio_approve_explain()}{#if waiting > 0} {m.studio_approve_waiting({ count: waiting })}{/if}</p>
				{#if approveError}<p class="md-body-small studio__error" role="alert">{approveError}</p>{/if}
			</div>
			<button type="button" class="studio__send md-label-large" disabled={approving} onclick={approve}>{m.studio_approve_confirm({ version: current.version })}</button>
			<button type="button" class="studio__clear md-label-large" onclick={() => (confirming = false)}>{m.studio_approve_cancel()}</button>
		</div>
	{/if}

	{#if !current && directionList.length === 0}
		<div class="studio__empty">
			<h2 class="md-title-large">{m.studio_empty_title()}</h2>
			<p class="md-body-large">{m.studio_empty_body()}</p>
		</div>
	{:else}
		<div class="studio__body">
			<div class="studio__stage" bind:this={stageEl}>
				{#if canvasEngine === 'openpencil'}
					<OpenPencilViewer
						model={openPencilModel}
						{tool}
						bind:zoom
						bind:panX
						bind:panY
						bind:inspectorOpen
						pins={canvasPins}
						selectedNodeId={selection?.id ?? null}
						onhit={handleOpenPencilHit}
						onemptyclick={clearSelection}
						onpinclick={(pin) => {
							const item = feedback.find((f) => f.number === pin.number);
							if (item) showFeedback(item);
						}}
						onpickDirection={(dirId) => {
							const dir = directionList.find((d) => d.id === dirId);
							if (dir) useDirection(dir);
						}}
					/>
				{:else if current}
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div
						bind:this={viewportEl}
						class="studio__canvas-viewport"
						class:studio__canvas-viewport--move={tool === 'move'}
						onpointerdown={startCanvasPan}
						onpointermove={moveCanvasPan}
						onpointerup={endCanvasPan}
						onpointercancel={endCanvasPan}
						onwheel={handleCanvasWheel}
					>
						<div class="studio__canvas-plane" style:transform={`translate(${panX}px, ${panY}px) scale(${zoom})`}>
							{#each splitPages.systemPages as sysPage (sysPage.path)}
								<article class="studio__artboard studio__artboard--system" data-artboard="design-system" style:width={`${canvasBoardWidth}px`}>
									<span class="studio__artboard-name">{sysPage.title}</span>
									<iframe
										bind:this={canvasFrames[sysPage.path]}
										class="studio__frame studio__frame--canvas"
										title={`${sysPage.title} (${sysPage.path})`}
										src={previewUrl(current, sysPage.path)}
										sandbox="allow-scripts"
										referrerpolicy="no-referrer"
										style:width={`${canvasBoardWidth}px`}
									></iframe>
								</article>
							{/each}
							{#each splitPages.flowPages as flowPage (flowPage.path)}
								<article class="studio__artboard studio__artboard--flow" data-artboard-page={flowPage.path} style:width={`${canvasBoardWidth}px`}>
									<span class="studio__artboard-name">{flowPage.title}</span>
									{#key `${current.preview_token}/${flowPage.path}`}
										<iframe
											bind:this={canvasFrames[flowPage.path]}
											class="studio__frame studio__frame--canvas"
											title={`${flowPage.title} (${flowPage.path})`}
											src={previewUrl(current, flowPage.path)}
											sandbox="allow-scripts"
											referrerpolicy="no-referrer"
											style:width={`${canvasBoardWidth}px`}
										></iframe>
									{/key}
								</article>
							{/each}
						</div>
					</div>
				{/if}

				{#if pinPopoverOpen && selection && current}
					<div
						class="studio__pin-popover"
						role="dialog"
						aria-label={selection.label}
						style:left={pinAt ? `${Math.max(POPOVER_MARGIN, Math.min(pinAt.x + POPOVER_MARGIN, (stageEl?.clientWidth ?? 0) - POPOVER_WIDTH - POPOVER_MARGIN))}px` : null}
						style:top={pinAt ? `${Math.max(POPOVER_MARGIN, Math.min(pinAt.y, (stageEl?.clientHeight ?? 0) - POPOVER_HEIGHT - POPOVER_MARGIN))}px` : null}
						style:right={pinAt ? 'auto' : null}
					>
						<div class="studio__pin-popover-head">
							<strong class="md-title-small">{selection.label}</strong>
							<span class="md-body-small studio__muted">{KIND_LABELS[selection.kind]}</span>
						</div>
						<form class="studio__form" onsubmit={savePinFeedback}>
							<!-- svelte-ignore a11y_autofocus -->
							<textarea class="studio__words md-body-medium" rows="3" maxlength={FEEDBACK_CHARS} placeholder={m.studio_feedback_placeholder()} bind:value={pinWords} autofocus></textarea>
							<label class="studio__choice md-body-small">
								<input type="checkbox" bind:checked={pinInteraction} />
								<span>{m.studio_feedback_interaction()}</span>
							</label>
							{#if feedbackError}<p class="md-body-small studio__error" role="alert">{feedbackError}</p>{/if}
							<div class="studio__tile-actions">
								<button type="submit" class="studio__send md-label-large" disabled={!pinDraft || sending}>{m.studio_pin_save()}</button>
								<button type="button" class="studio__clear md-label-large" onclick={() => (pinPopoverOpen = false)}>{m.studio_approve_cancel()}</button>
							</div>
						</form>
					</div>
				{/if}

				{#if waiting > 0}
					<div class="studio__revise-bar" role="region" aria-label={m.studio_refine_title()}>
						<span class="md-body-small">{m.studio_revise_bar_count({ count: waiting })}</span>
						{#if asked}
							<span class="md-body-small studio__muted" role="status">{m.studio_refine_sent()}</span>
							<button type="button" class="studio__tool studio__tool--wide" onclick={refresh}>{m.studio_refine_refresh()}</button>
						{:else}
							<button type="button" class="studio__send md-label-large" disabled={asking} onclick={() => askSora()}>{m.studio_revise_cta()}</button>
						{/if}
					</div>
				{/if}

				<div class="studio__toolbar" role="toolbar" aria-label={m.studio_tools()}>
					{#each TOOLS as option (option.tool)}
						<button
							type="button"
							class="studio__tool"
							aria-pressed={tool === option.tool}
							aria-label={`${option.label} (${option.key})`}
							title={`${option.label} (${option.key})`}
							onclick={() => pickTool(option.tool)}
						>
							<Icon icon={option.icon} size={18} />
						</button>
					{/each}
					<span class="studio__toolbar-gap" aria-hidden="true"></span>
					<button type="button" class="studio__tool studio__zoom-step" aria-label={m.studio_zoom_out()} title={m.studio_zoom_out()} onclick={zoomOut}><Icon icon={ZoomOut} size={18} /></button>
					<span class="studio__zoom-readout md-label-medium">{zoomLabel(zoom)}</span>
					<button type="button" class="studio__tool studio__zoom-step" aria-label={m.studio_zoom_in()} title={m.studio_zoom_in()} onclick={zoomIn}><Icon icon={ZoomIn} size={18} /></button>
					<button type="button" class="studio__tool" aria-label={m.studio_zoom_fit()} title={m.studio_zoom_fit()} onclick={fitCanvas}><Icon icon={Maximize2} size={16} /></button>
					{#if canvasEngine === 'openpencil'}
						<span class="studio__toolbar-gap" aria-hidden="true"></span>
						<button type="button" class="studio__tool" aria-pressed={inspectorOpen} aria-label={m.studio_inspector()} title={m.studio_inspector()} onclick={() => (inspectorOpen = !inspectorOpen)}><Icon icon={PanelRight} size={18} /></button>
					{/if}
				</div>
			</div>
		</div>
	{/if}
</div>

<style>
	.studio {
		display: flex;
		flex-direction: column;
		height: 100dvh;
		min-height: 0;
		overflow: hidden;
		container-type: inline-size;
		background: var(--md-sys-color-surface);
		color: var(--md-sys-color-on-surface);
	}
	.studio--embedded {
		height: 100%;
	}
	.studio__bar {
		display: flex;
		align-items: center;
		gap: 12px;
		min-height: 56px;
		padding: 8px 16px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
		flex-wrap: wrap;
	}
	.studio__back,
	.studio__tool {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		min-width: 40px;
		height: 40px;
		border: 0;
		border-radius: var(--md-sys-shape-corner-medium);
		background: none;
		color: var(--md-sys-color-on-surface-variant);
		text-decoration: none;
		font: inherit;
		cursor: pointer;
	}
	.studio__tool--wide {
		padding: 0 12px;
	}
	.studio__back:hover,
	.studio__tool:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
	.studio__tool[aria-pressed='true'] {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.studio__back:focus-visible,
	.studio__tool:focus-visible,
	.studio__clear:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: -2px;
	}
	.studio__heading {
		flex: 1;
		min-width: 0;
	}
	.studio__title,
	.studio__sub {
		margin: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.studio__sub,
	.studio__muted {
		color: var(--md-sys-color-on-surface-variant);
	}
	.studio__tools,
	.studio__devices {
		display: inline-flex;
		align-items: center;
		gap: 4px;
	}
	.studio__tools {
		gap: 12px;
	}
	.studio__empty {
		margin: auto;
		padding: 24px;
		max-width: 420px;
		text-align: center;
	}
	.studio__body {
		display: flex;
		flex: 1;
		min-height: 0;
		width: 100%;
	}
	.studio__stage {
		position: relative;
		display: flex;
		flex: 1;
		min-width: 0;
		min-height: 0;
		overflow: hidden;
		background: var(--md-sys-color-surface-container);
	}
	.studio__canvas-viewport {
		position: relative;
		flex: 1;
		overflow: hidden;
		padding: 48px;
		background-color: var(--md-sys-color-surface-container);
	}
	.studio__canvas-viewport--move {
		cursor: grab;
	}
	.studio__canvas-viewport--move:active {
		cursor: grabbing;
	}
	.studio__canvas-viewport--move iframe {
		pointer-events: none;
	}
	.studio__canvas-plane {
		display: inline-flex;
		align-items: flex-start;
		gap: 48px;
		transform-origin: 0 0;
	}
	.studio__artboard {
		display: flex;
		flex-direction: column;
		gap: 6px;
		flex-shrink: 0;
	}
	.studio__artboard-name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-size: 12px;
		color: var(--md-sys-color-on-surface-variant);
	}
	.studio__frame {
		flex: none;
		max-width: 100%;
		border: 0;
		border-radius: 2px;
		background: var(--md-sys-color-surface-container-lowest);
		box-shadow: 0 1px 3px color-mix(in srgb, var(--md-sys-color-shadow) 16%, transparent);
	}
	.studio__frame--canvas {
		height: 680px;
	}
	.studio__toolbar {
		position: absolute;
		bottom: 16px;
		left: 50%;
		transform: translateX(-50%);
		z-index: 25;
		display: inline-flex;
		align-items: center;
		gap: 2px;
		padding: 4px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-lowest);
		box-shadow: 0 6px 20px color-mix(in srgb, var(--md-sys-color-shadow) 16%, transparent);
	}
	.studio__toolbar-gap {
		width: 1px;
		height: 24px;
		margin: 0 4px;
		background: var(--md-sys-color-outline-variant);
	}
	.studio__zoom-readout {
		min-width: 44px;
		text-align: center;
		color: var(--md-sys-color-on-surface-variant);
		font-variant-numeric: tabular-nums;
	}
	.studio__pin-popover {
		position: absolute;
		top: 16px;
		right: 16px;
		z-index: 30;
		width: 300px;
		max-width: calc(100% - 32px);
		box-sizing: border-box;
		padding: 12px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface-container-lowest);
		box-shadow: 0 8px 24px color-mix(in srgb, var(--md-sys-color-shadow) 18%, transparent);
	}
	.studio__pin-popover-head {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.studio__pin-popover-head > * {
		margin: 0;
	}
	.studio__revise-bar {
		position: absolute;
		bottom: 76px;
		left: 50%;
		transform: translateX(-50%);
		z-index: 24;
		display: inline-flex;
		align-items: center;
		gap: 12px;
		padding: 6px 6px 6px 16px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-lowest);
		box-shadow: 0 6px 20px color-mix(in srgb, var(--md-sys-color-shadow) 16%, transparent);
		white-space: nowrap;
	}
	.studio__tile-actions {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-wrap: wrap;
	}
	.studio__approved {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		height: 40px;
		padding: 0 12px;
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-tertiary-container);
		color: var(--md-sys-color-on-tertiary-container);
	}
	.studio__tool--primary {
		background: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
	}
	.studio__tool--primary:hover {
		background: color-mix(in srgb, var(--md-sys-color-primary) 92%, var(--md-sys-color-on-primary));
	}
	.studio__confirm {
		display: flex;
		align-items: center;
		gap: 16px;
		flex-wrap: wrap;
		padding: 12px 16px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
		background: var(--md-sys-color-surface-container-high);
	}
	.studio__confirm-text {
		flex: 1;
		min-width: 220px;
	}
	.studio__confirm-text p {
		margin: 0;
	}
	.studio__form {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin-top: 12px;
	}
	.studio__words {
		width: 100%;
		padding: 8px 12px;
		border: 1px solid var(--md-sys-color-outline);
		border-radius: var(--md-sys-shape-corner-small);
		background: var(--md-sys-color-surface);
		color: inherit;
		box-sizing: border-box;
		resize: vertical;
	}
	.studio__words:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: -1px;
	}
	.studio__choice {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		color: var(--md-sys-color-on-surface-variant);
	}
	.studio__notice {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px 12px;
		padding: 8px 16px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
		background: var(--md-sys-color-surface-container-low);
		color: var(--md-sys-color-on-surface-variant);
	}
	.studio__notice p {
		margin: 0;
		flex: 1 1 240px;
	}
	.studio__error {
		margin: 0;
		color: var(--md-sys-color-error);
	}
	.studio__send {
		min-height: 40px;
		padding: 0 20px;
		border: 0;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
		cursor: pointer;
	}
	.studio__send:disabled,
	.studio__tool:disabled {
		opacity: 0.38;
		cursor: default;
	}
	.studio__send:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.studio__clear {
		padding: 6px 8px;
		border: 0;
		background: none;
		color: var(--md-sys-color-primary);
		cursor: pointer;
	}
	@container (max-width: 480px) {
		.studio__toolbar .studio__zoom-step {
			display: none;
		}
	}
	@media (pointer: coarse) {
		.studio__back,
		.studio__tool {
			min-width: 44px;
			height: 44px;
		}
	}
</style>
