<script lang="ts">
	import { ArrowLeft, BadgeCheck, Hand, Maximize2, MessageSquarePlus, Monitor, MousePointer2, PanelRight, Smartphone, Tablet, ZoomIn, ZoomOut } from '@lucide/svelte';
	import { invalidateAll } from '$app/navigation';
	import { m } from '$lib/paraglide/messages';
	import Icon from '$lib/ds/foundations/Icon.svelte';
	import OpenPencilViewer, { type CanvasPin } from '$lib/ds/design/OpenPencilViewer.svelte';
	import { buildStudioSceneModel, type StudioHitResult } from '$lib/design/openPencilCanvas';
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
		loadPenDocument,
		modeMessage,
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
		type PenLoad,
		type StudioApproval,
		type StudioDirection,
		type StudioDirectionSet,
		type StudioElement,
		type StudioElementKind,
		type StudioFeedback,
		type StudioPreview,
	} from '$lib/design/designStudio';

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
	// frontend-new proxies the Studio's API under the chat (chatRoutes.ts).
	const apiBase = $derived(`/chat/${sessionId}/api/design`);

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

	// The version's own .pen files, loaded from where its files are served. Until they're here, or
	// when they can't be, the canvas says so and draws nothing of the version in their place.
	let pen = $state<{ token: string; load: PenLoad } | null>(null);
	$effect(() => {
		const preview = current;
		if (!preview || canvasEngine !== 'openpencil' || pen?.token === preview.preview_token) return;
		const token = preview.preview_token;
		pen = { token, load: { state: 'loading' } };
		void loadPenDocument(preview).then((load) => {
			if (current?.preview_token === token) pen = { token, load };
		});
	});
	const penLoad = $derived<PenLoad | null>(
		current && canvasEngine === 'openpencil' ? (pen?.token === current.preview_token ? pen.load : { state: 'loading' }) : null
	);
	const penJson = $derived(penLoad?.state === 'ready' ? penLoad.json : null);
	const openPencilModel = $derived(
		buildStudioSceneModel({
			directions: directionList,
			preview: current,
			activeDirection,
			device,
			rawPenJson: penJson
		})
	);

	// What the canvas says in place of a .pen version it hasn't drawn.
	const penNotice = $derived<
		{ kind: 'loading' } | { kind: 'failed'; reason: 'unreachable' | 'unreadable'; status: number | null } | null
	>(
		!penLoad
			? null
			: penLoad.state === 'loading'
				? { kind: 'loading' }
				: penLoad.state === 'failed'
					? { kind: 'failed', reason: penLoad.reason, status: penLoad.status }
					: openPencilModel.penUnreadable
						? { kind: 'failed', reason: 'unreadable', status: null }
						: null
	);
	function retryPen() {
		pen = null;
	}

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
		if (target?.closest('button, a, input, textarea, select, iframe, [data-part="pin-popover"]')) return;
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
			const response = await fetch(`${apiBase}/feedback`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(draft) });
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
			const response = await fetch(`${apiBase}/feedback`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(pinDraft) });
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
			const response = await fetch(`${apiBase}/refine`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ text }) });
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
			const response = await fetch(`${apiBase}/approval`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ version: current.version }) });
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
			const response = await fetch(`${apiBase}/refine`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ text: request }) });
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

<div class="flex h-full min-h-0 flex-col bg-bg text-text" data-testid="design-canvas">
	<header class="flex flex-wrap items-center gap-2.5 border-b border-border bg-surface px-4 py-2.5">
		{#if !embedded}
			<a href={`/chat/${data.sessionId}`} class="inline-flex size-8 items-center justify-center rounded-sm text-text-2 hover:bg-surface-2 hover:text-text" aria-label={m.studio_back()} title={m.studio_back()}>
				<Icon icon={ArrowLeft} size={16} />
			</a>
		{/if}
		<div class="flex min-w-0 flex-1 flex-col">
			<h1 class="m-0 truncate text-ui font-semibold">{current ? current.title : m.studio_title()}</h1>
			{#if current}
				<span class="truncate text-caption text-text-2">{activeDirection?.name ?? current.direction_id} · {m.studio_version({ version: current.version })}</span>
			{/if}
		</div>
		{#if current}
			<div class="flex flex-wrap items-center gap-2">
				{#if canvasEngine === 'frames'}
					<div class="inline-flex rounded-sm bg-surface-2 p-0.5 outline -outline-offset-1 outline-border" role="group" aria-label={m.studio_title()}>
						{#each DEVICES as option (option.device)}
							<button
								type="button"
								class={['inline-flex size-7 cursor-pointer items-center justify-center rounded-xs', device === option.device ? 'bg-surface text-text outline -outline-offset-1 outline-border-strong' : 'text-text-2 hover:text-text']}
								aria-pressed={device === option.device}
								aria-label={option.label}
								title={option.label}
								onclick={() => (device = option.device)}
							>
								<Icon icon={option.icon} size={14} />
							</button>
						{/each}
					</div>
				{/if}
				{#if approval?.version === current.version}
					<span class="inline-flex items-center gap-1.5 rounded-sm bg-accent-bg px-2.5 py-1 text-caption font-medium text-accent">
						<Icon icon={BadgeCheck} size={14} />
						<span>{m.studio_approved()}</span>
					</span>
				{:else}
					<button type="button" class="inline-flex cursor-pointer items-center gap-1.5 rounded-sm bg-accent px-3 py-1.5 text-caption font-medium text-accent-ink" aria-expanded={confirming} onclick={() => (confirming = !confirming)}>
						{m.studio_approve()}
					</button>
				{/if}
			</div>
		{/if}
	</header>

	{#if directionSent || directionError}
		<div class="flex flex-wrap items-center gap-3 border-b border-border bg-surface-2 px-4 py-2 text-caption" role={directionError ? 'alert' : 'status'}>
			{#if directionError}
				<span class="text-danger">{directionError}</span>
			{:else}
				<span class="text-text-2">{m.studio_refine_sent()}</span>
				<button type="button" class="cursor-pointer rounded-xs bg-surface px-2 py-1 text-caption text-text outline -outline-offset-1 outline-border hover:outline-border-strong" onclick={refresh}>{m.studio_refine_refresh()}</button>
			{/if}
		</div>
	{/if}

	{#if current && confirming && approval?.version !== current.version}
		<section class="flex flex-wrap items-center justify-between gap-3 border-b border-border bg-surface-2 px-4 py-3 text-ui" aria-label={m.studio_approve()}>
			<div class="flex flex-col gap-0.5">
				<strong class="font-medium text-text">{m.studio_approve_question({ version: current.version })}</strong>
				<p class="m-0 text-caption text-text-2">{m.studio_approve_explain()}{#if waiting > 0} {m.studio_approve_waiting({ count: waiting })}{/if}</p>
				{#if approveError}<p class="m-0 text-caption text-danger" role="alert">{approveError}</p>{/if}
			</div>
			<div class="flex items-center gap-2">
				<button type="button" class="cursor-pointer rounded-sm bg-accent px-3 py-1.5 text-caption font-medium text-accent-ink disabled:opacity-50" disabled={approving} onclick={approve}>
					{m.studio_approve_confirm({ version: current.version })}
				</button>
				<button type="button" class="cursor-pointer rounded-sm bg-surface px-3 py-1.5 text-caption text-text outline -outline-offset-1 outline-border hover:outline-border-strong" onclick={() => (confirming = false)}>
					{m.studio_approve_cancel()}
				</button>
			</div>
		</section>
	{/if}

	{#if !current && directionList.length === 0}
		<section class="m-auto flex max-w-96 flex-col items-center gap-1.5 p-8 text-center">
			<h2 class="m-0 text-body font-semibold text-text">{m.studio_empty_title()}</h2>
			<p class="m-0 text-ui text-text-2">{m.studio_empty_body()}</p>
		</section>
	{:else}
		<div class="relative flex min-h-0 flex-1 overflow-hidden bg-bg" bind:this={stageEl}>
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
				{#if penNotice?.kind === 'loading'}
					<p class="absolute inset-0 m-auto h-fit w-fit text-ui text-text-2" role="status">{m.studio_pen_loading()}</p>
				{:else if penNotice?.kind === 'failed'}
					<section class="absolute inset-0 m-auto flex h-fit max-w-96 flex-col items-center gap-1.5 p-8 text-center" role="alert">
						<h2 class="m-0 text-body font-semibold text-text">{m.studio_pen_failed_title()}</h2>
						<p class="m-0 text-ui text-text-2">
							{penNotice.reason === 'unreachable' ? m.studio_pen_unreachable() : m.studio_pen_unreadable()}
						</p>
						{#if penNotice.status !== null}
							<span class="font-mono text-label text-text-3">{m.studio_pen_status({ status: String(penNotice.status) })}</span>
						{/if}
						<button type="button" class="mt-2.5 cursor-pointer rounded-sm border border-border-strong bg-surface px-2.75 py-1.25 text-ui text-text hover:bg-surface-2 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus" onclick={retryPen}>
							{m.studio_pen_retry()}
						</button>
					</section>
				{/if}
			{:else if current}
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div
					bind:this={viewportEl}
					class={['relative flex-1 overflow-hidden p-12', tool === 'move' && 'cursor-grab active:cursor-grabbing']}
					data-testid="spatial-canvas-viewport"
					onpointerdown={startCanvasPan}
					onpointermove={moveCanvasPan}
					onpointerup={endCanvasPan}
					onpointercancel={endCanvasPan}
					onwheel={handleCanvasWheel}
				>
					<div class="inline-flex origin-top-left items-start gap-12" style:transform={`translate(${panX}px, ${panY}px) scale(${zoom})`} data-testid="spatial-canvas-plane">
						{#each [...splitPages.systemPages, ...splitPages.flowPages] as page (page.path)}
							<article class="flex shrink-0 flex-col gap-1.5" data-artboard-page={page.path} style:width={`${canvasBoardWidth}px`}>
								<span class="truncate text-caption text-text-2">{page.title}</span>
								{#key `${current.preview_token}/${page.path}`}
									<iframe
										bind:this={canvasFrames[page.path]}
										class={['h-170 rounded-xs border-0 bg-surface shadow-float', tool === 'move' && 'pointer-events-none']}
										style:width={`${canvasBoardWidth}px`}
										src={previewUrl(current, page.path)}
										sandbox="allow-scripts"
										referrerpolicy="no-referrer"
										title={m.studio_frame_title({ title: page.title })}
									></iframe>
								{/key}
							</article>
						{/each}
					</div>
				</div>
			{/if}

			{#if pinPopoverOpen && selection && current}
				<aside
					data-part="pin-popover"
					data-testid="direct-pin-popover"
					aria-label={m.studio_feedback_label()}
					class="absolute top-4 right-4 z-30 flex w-75 max-w-full flex-col gap-2.5 rounded-md bg-surface p-3.5 shadow-float outline -outline-offset-1 outline-border-strong"
					style:left={pinAt ? `${Math.max(POPOVER_MARGIN, Math.min(pinAt.x + POPOVER_MARGIN, (stageEl?.clientWidth ?? 0) - POPOVER_WIDTH - POPOVER_MARGIN))}px` : null}
					style:top={pinAt ? `${Math.max(POPOVER_MARGIN, Math.min(pinAt.y, (stageEl?.clientHeight ?? 0) - POPOVER_HEIGHT - POPOVER_MARGIN))}px` : null}
					style:right={pinAt ? 'auto' : null}
				>
					<div class="flex flex-col">
						<strong class="text-caption font-medium text-text">{selection.label}</strong>
						<span class="text-label text-text-3">{KIND_LABELS[selection.kind]}</span>
					</div>
					<form class="flex flex-col gap-2" onsubmit={savePinFeedback}>
						<!-- svelte-ignore a11y_autofocus -->
						<textarea
							rows="3"
							maxlength={FEEDBACK_CHARS}
							class="w-full rounded-sm bg-bg p-2 text-caption text-text outline -outline-offset-1 outline-border focus:outline-border-strong"
							bind:value={pinWords}
							placeholder={m.studio_feedback_placeholder()}
							autofocus
						></textarea>
						<label class="flex items-center gap-1.5 text-caption text-text-2">
							<input type="checkbox" bind:checked={pinInteraction} />
							<span>{m.studio_feedback_interaction()}</span>
						</label>
						{#if feedbackError}<p class="m-0 text-caption text-danger" role="alert">{feedbackError}</p>{/if}
						<div class="flex items-center justify-end gap-2">
							<button type="button" class="cursor-pointer rounded-sm px-2.5 py-1 text-caption text-text-2 hover:text-text" onclick={() => (pinPopoverOpen = false)}>{m.studio_approve_cancel()}</button>
							<button type="submit" class="cursor-pointer rounded-sm bg-accent px-3 py-1 text-caption font-medium text-accent-ink disabled:opacity-50" disabled={!pinDraft || sending}>{m.studio_pin_save()}</button>
						</div>
					</form>
				</aside>
			{/if}

			{#if waiting > 0}
				<div
					data-testid="revise-with-sora-bar"
					class="absolute bottom-19 left-1/2 z-24 inline-flex -translate-x-1/2 items-center gap-3 rounded-full bg-surface py-1.5 pr-1.5 pl-4 whitespace-nowrap shadow-float outline -outline-offset-1 outline-border"
					role="region"
					aria-label={m.studio_refine_title()}
				>
					<span class="text-caption text-text-2">{m.studio_revise_bar_count({ count: waiting })}</span>
					{#if asked}
						<span class="text-caption text-text-2" role="status">{m.studio_refine_sent()}</span>
						<button type="button" class="cursor-pointer rounded-full bg-surface-2 px-3 py-1 text-caption text-text" onclick={refresh}>{m.studio_refine_refresh()}</button>
					{:else}
						<button type="button" class="cursor-pointer rounded-full bg-accent px-3.5 py-1.5 text-caption font-medium text-accent-ink disabled:opacity-50" disabled={asking} onclick={() => askSora()}>{m.studio_revise_cta()}</button>
					{/if}
					{#if askError}<span class="text-caption text-danger" role="alert">{askError}</span>{/if}
				</div>
			{/if}

			<div class="absolute bottom-4 left-1/2 z-25 inline-flex -translate-x-1/2 items-center gap-0.5 rounded-md bg-surface p-1 shadow-float outline -outline-offset-1 outline-border" role="toolbar" aria-label={m.studio_tools()}>
				{#each TOOLS as option (option.tool)}
					<button
						type="button"
						class={['inline-flex size-9 cursor-pointer items-center justify-center rounded-sm focus-visible:outline-2 focus-visible:outline-accent max-md:size-11', tool === option.tool ? 'bg-accent text-accent-ink' : 'text-text-2 hover:bg-surface-2 hover:text-text']}
						aria-pressed={tool === option.tool}
						aria-label={`${option.label} (${option.key})`}
						title={`${option.label} (${option.key})`}
						onclick={() => pickTool(option.tool)}
					>
						<Icon icon={option.icon} size={16} />
					</button>
				{/each}
				<span class="mx-1 h-6 w-px bg-border" aria-hidden="true"></span>
				<button type="button" class="inline-flex size-9 cursor-pointer items-center justify-center rounded-sm text-text-2 hover:bg-surface-2 hover:text-text max-sm:hidden" aria-label={m.studio_zoom_out()} title={m.studio_zoom_out()} onclick={zoomOut}><Icon icon={ZoomOut} size={16} /></button>
				<span class="min-w-11 text-center font-mono text-label text-text-2 tabular-nums" data-testid="canvas-zoom-label">{zoomLabel(zoom)}</span>
				<button type="button" class="inline-flex size-9 cursor-pointer items-center justify-center rounded-sm text-text-2 hover:bg-surface-2 hover:text-text max-sm:hidden" aria-label={m.studio_zoom_in()} title={m.studio_zoom_in()} onclick={zoomIn}><Icon icon={ZoomIn} size={16} /></button>
				<button type="button" class="inline-flex size-9 cursor-pointer items-center justify-center rounded-sm text-text-2 hover:bg-surface-2 hover:text-text max-md:size-11" aria-label={m.studio_zoom_fit()} title={m.studio_zoom_fit()} onclick={fitCanvas}><Icon icon={Maximize2} size={14} /></button>
				{#if canvasEngine === 'openpencil'}
					<span class="mx-1 h-6 w-px bg-border" aria-hidden="true"></span>
					<button
						type="button"
						class={['inline-flex size-9 cursor-pointer items-center justify-center rounded-sm max-md:size-11', inspectorOpen ? 'bg-surface-2 text-text' : 'text-text-2 hover:bg-surface-2 hover:text-text']}
						aria-pressed={inspectorOpen}
						aria-label={m.studio_inspector()}
						title={m.studio_inspector()}
						onclick={() => (inspectorOpen = !inspectorOpen)}
					>
						<Icon icon={PanelRight} size={16} />
					</button>
				{/if}
			</div>
		</div>
	{/if}
</div>
