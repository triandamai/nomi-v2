<script lang="ts">
	import { ArrowLeft, BadgeCheck, ExternalLink, Maximize2, Monitor, MousePointerClick, Smartphone, Tablet, ZoomIn, ZoomOut } from '@lucide/svelte';
	import { invalidateAll } from '$app/navigation';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { formatDate } from '$lib/dates';
	import Icon from '$lib/components/ds/foundations/Icon.svelte';
	import {
		CANVAS_ARTBOARD_WIDTHS,
		DEVICE_WIDTHS,
		activeDirectionForPreview,
		adjustDirectionMessage,
		annotateMessage,
		annotationsOn,
		clampZoom,
		compareWith,
		elementPinDraft,
		elementsOnPage,
		feedbackDraft,
		fitZoom,
		highlightMessage,
		modeMessage,
		pickDirectionMessage,
		pickVersion,
		previewUrl,
		readPreviewMessage,
		refineMessage,
		splitCanvasPages,
		versionsNewestFirst,
		zoomLabel,
		zoomStep,
		type CanvasStageFilter,
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
		type StudioPublishOutcome,
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

	let selecting = $state(false);
	let selection = $state<Selection | null>(null);
	let frame = $state<HTMLIFrameElement | null>(null);
	let canvasFrames = $state<Record<string, HTMLIFrameElement | null>>({});

	// Looking at a version, exploring the 2D spatial canvas, giving feedback, or comparing two versions.
	type Mode = 'preview' | 'canvas' | 'feedback' | 'compare';
	const MODES: { mode: Mode; label: string }[] = [
		{ mode: 'preview', label: m.studio_mode_preview() },
		{ mode: 'canvas', label: m.studio_mode_canvas() },
		{ mode: 'feedback', label: m.studio_mode_feedback() },
		{ mode: 'compare', label: m.studio_mode_compare() },
	];
	let mode = $state<Mode>('preview');
	const picking = $derived(selecting || mode === 'feedback' || mode === 'canvas');

	// Spatial canvas state: zoom, pan, stage filter, and direct click-to-pin popover.
	let zoom = $state(1);
	let panX = $state(0);
	let panY = $state(0);
	let isPanning = $state(false);
	let panOrigin = $state<{ x: number; y: number; panX: number; panY: number } | null>(null);
	let stageFilter = $state<CanvasStageFilter>('all');
	let viewportEl = $state<HTMLElement | null>(null);
	let pinPopoverOpen = $state(false);
	let pinWords = $state('');
	let pinInteraction = $state(false);

	// Direction exploration state (Stage 1)
	let adjustingDirectionId = $state<string | null>(null);
	let directionNote = $state('');
	let sendingDirection = $state(false);
	let directionSent = $state(false);
	let directionError = $state('');

	// Everything said about the chat's previews: what the page loaded with, and what was added here.
	let added = $state<StudioFeedback[]>([]);
	// Feedback added here that a reload has since brought in is shown once.
	const feedback = $derived([...given, ...added.filter((a) => !given.some((g) => g.id === a.id))]);
	const history = $derived([...feedback].sort((a, b) => b.number - a.number));
	const marks = $derived(current && (mode === 'feedback' || mode === 'canvas' || selecting) ? annotationsOn(feedback, current, shownPage) : []);

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
			if (mode === 'feedback') {
				scope = 'element';
			} else {
				pinPopoverOpen = true;
			}
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

	function toggleSelecting() {
		selecting = !selecting;
	}

	function pick(element: StudioElement) {
		const targetPage = element.pages.includes(shownPage) ? shownPage : (element.pages[0] ?? shownPage);
		selection = { id: element.id, page: targetPage, kind: element.kind, label: element.label, tag: '', text: '', point: null };
		if (mode === 'feedback') {
			scope = 'element';
		} else {
			pinPopoverOpen = true;
		}
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
		const boardCount = Math.max(1, (current ? splitPages.flowPages.length + 1 : 0) + (stageFilter === 'all' || stageFilter === 'directions' ? directionList.length : 0));
		const totalWidth = boardCount * (canvasBoardWidth + 32);
		const vw = viewportEl?.clientWidth ?? 1200;
		zoom = fitZoom(vw, totalWidth);
		panX = 0;
		panY = 0;
	}

	function startCanvasPan(event: PointerEvent) {
		if (event.button !== 0) return;
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
		if (event.ctrlKey || event.metaKey) {
			event.preventDefault();
			zoom = clampZoom(zoom + (event.deltaY < 0 ? 0.1 : -0.1));
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
			adjustingDirectionId = null;
			directionNote = '';
		} catch {
			directionError = m.studio_refine_failed();
		} finally {
			sendingDirection = false;
		}
	}

	function useDirection(direction: StudioDirection) {
		void sendDirectionMessage(pickDirectionMessage(direction));
	}

	function submitDirectionAdjustment(event: SubmitEvent, direction: StudioDirection) {
		event.preventDefault();
		const text = adjustDirectionMessage(direction, directionNote);
		if (text) void sendDirectionMessage(text);
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

	// Publishing a version again, after it couldn't be: what came of it here wins over the page's data.
	let publishedHere = $state<Record<number, StudioPublishOutcome>>({});
	let publishing = $state(false);
	let publishAgainError = $state('');
	const publishedUrl = $derived(current ? (publishedHere[current.version]?.published_url ?? current.published_url) : null);
	const publishError = $derived(current ? (publishedHere[current.version] ? publishedHere[current.version].publish_error : (current.publish_error ?? null)) : null);

	async function publishAgain() {
		if (!current || publishing) return;
		const version = current.version;
		publishing = true;
		publishAgainError = '';
		try {
			const response = await fetch(`/chat/${data.sessionId}/design/publish`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ version }) });
			if (!response.ok) {
				publishAgainError = m.studio_publish_failed();
				return;
			}
			publishedHere = { ...publishedHere, [version]: (await response.json()) as StudioPublishOutcome };
		} catch {
			publishAgainError = m.studio_publish_failed();
		} finally {
			publishing = false;
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
		tellAll(highlightMessage(null));
	}
</script>

<svelte:window onmessage={heard} onkeydown={(e) => e.key === 'Escape' && clearSelection()} />

{#snippet directionBoards()}
	{#each directionList as direction (direction.id)}
		{@const bgHex = direction.palette[0]?.hex ?? '#F8F7F4'}
		{@const accentHex = direction.palette[1]?.hex ?? direction.palette[0]?.hex ?? '#2ED3B7'}
		<article class="studio__artboard studio__artboard--direction" data-direction-id={direction.id}>
			<header class="studio__artboard-bar">
				<span class="studio__artboard-kind">{m.studio_stage_directions()}</span>
				<strong class="studio__artboard-name">{direction.name}</strong>
				<code class="studio__mono">{direction.id}</code>
				{#if direction.recommended}
					<span class="studio__badge">{m.studio_direction_recommended()}</span>
				{/if}
			</header>
			<div class="studio__tile-specimen" style:background-color={bgHex}>
				<div class="studio__tile-card">
					<span class="studio__mono">{direction.typography.heading} · {direction.typography.body}</span>
					<h3 class="studio__tile-heading" style:font-family={direction.typography.heading}>{direction.name}</h3>
					<p class="studio__tile-summary" style:font-family={direction.typography.body}>{direction.summary}</p>
					<span class="studio__tile-cta" style:background-color={accentHex}>{direction.id}</span>
				</div>
			</div>
			<div class="studio__tile-body">
				<div class="studio__tile-section">
					<span class="studio__tile-label">{m.studio_direction_palette()}</span>
					<div class="studio__swatches">
						{#each direction.palette as colour (`${colour.name}-${colour.hex}`)}
							<div class="studio__swatch">
								<span class="studio__swatch-chip" style:background-color={colour.hex}></span>
								<span class="studio__swatch-name">{colour.name}</span>
								<code class="studio__mono">{colour.hex}</code>
							</div>
						{/each}
					</div>
				</div>
				<div class="studio__tile-section">
					<span class="studio__tile-label">{m.studio_direction_typography()}</span>
					<p class="md-body-small"><strong>{direction.typography.heading}</strong> / {direction.typography.body}</p>
				</div>
				<div class="studio__tile-section">
					<span class="studio__tile-label">{m.studio_direction_layout()}</span>
					<p class="md-body-small">{direction.layout}</p>
				</div>
				<div class="studio__tile-section">
					<span class="studio__tile-label">{m.studio_direction_rationale()}</span>
					<p class="md-body-small">{direction.rationale}</p>
				</div>
				{#if direction.trade_offs.length > 0}
					<div class="studio__tile-section">
						<span class="studio__tile-label">{m.studio_direction_tradeoffs()}</span>
						<ul class="studio__tradeoffs">
							{#each direction.trade_offs as tradeoff (tradeoff)}
								<li class="md-body-small">{tradeoff}</li>
							{/each}
						</ul>
					</div>
				{/if}
				<div class="studio__tile-actions">
					<button type="button" class="studio__send md-label-large" disabled={sendingDirection} onclick={() => useDirection(direction)}>
						{m.studio_direction_use()}
					</button>
					<button
						type="button"
						class="studio__tool studio__tool--wide"
						onclick={() => {
							adjustingDirectionId = adjustingDirectionId === direction.id ? null : direction.id;
							directionNote = '';
						}}
					>
						{m.studio_direction_adjust()}
					</button>
				</div>
				{#if adjustingDirectionId === direction.id}
					<form class="studio__form" onsubmit={(e) => submitDirectionAdjustment(e, direction)}>
						<textarea
							class="studio__words md-body-medium"
							rows="2"
							maxlength={FEEDBACK_CHARS}
							placeholder={m.studio_direction_adjust_placeholder()}
							bind:value={directionNote}
						></textarea>
						<button type="submit" class="studio__send md-label-large" disabled={!directionNote.trim() || sendingDirection}>
							{m.studio_direction_send()}
						</button>
					</form>
				{/if}
			</div>
		</article>
	{/each}
{/snippet}

<div class="studio" class:studio--embedded={embedded}>
	<header class="studio__bar">
		{#if !embedded}<a class="studio__back" href={`/chat/${data.sessionId}`} aria-label={m.studio_back()} title={m.studio_back()}><Icon icon={ArrowLeft} size={20} /></a>{/if}
		<div class="studio__heading">
			<h1 class="md-title-medium studio__title">{current ? current.title : m.studio_title()}</h1>
			{#if current}<p class="md-body-small studio__sub">{m.studio_direction({ direction: current.direction_id })} · {m.studio_version({ version: current.version })}</p>{/if}
		</div>
		{#if current}
			<div class="studio__tools">
				<div class="studio__devices" role="group" aria-label={m.pj_preview_device()}>
					{#each DEVICES as option (option.device)}
						<button type="button" class="studio__tool" aria-pressed={device === option.device} aria-label={option.label} title={option.label} onclick={() => (device = option.device)}>
							<Icon icon={option.icon} size={20} />
						</button>
					{/each}
				</div>
				<div class="studio__devices" role="group" aria-label={m.studio_mode()}>
					{#each MODES as option (option.mode)}
						<button type="button" class="studio__tool studio__tool--wide" aria-pressed={mode === option.mode} onclick={() => openMode(option.mode)}>{option.label}</button>
					{/each}
				</div>
				{#if mode === 'canvas'}
					<div class="studio__devices" role="group" aria-label="Zoom">
						<button type="button" class="studio__tool" aria-label={m.studio_zoom_out()} title={m.studio_zoom_out()} onclick={zoomOut}><Icon icon={ZoomOut} size={18} /></button>
						<span class="studio__mono studio__zoom-readout">{zoomLabel(zoom)}</span>
						<button type="button" class="studio__tool" aria-label={m.studio_zoom_in()} title={m.studio_zoom_in()} onclick={zoomIn}><Icon icon={ZoomIn} size={18} /></button>
						<button type="button" class="studio__tool studio__tool--wide" onclick={fitCanvas}><Icon icon={Maximize2} size={16} /><span>{m.studio_zoom_fit()}</span></button>
					</div>
				{/if}
				<button type="button" class="studio__tool studio__tool--wide" aria-pressed={picking} disabled={mode === 'feedback' || mode === 'canvas'} onclick={toggleSelecting}>
					<Icon icon={MousePointerClick} size={18} /><span>{m.studio_select()}</span>
				</button>
				{#if approval?.version === current.version}
					<span class="studio__approved md-label-large"><Icon icon={BadgeCheck} size={18} /><span>{m.studio_approved()}</span></span>
				{:else}
					<button type="button" class="studio__tool studio__tool--wide studio__tool--primary" onclick={() => (confirming = true)}>{m.studio_approve()}</button>
				{/if}
				{#if publishedUrl}
					<a class="studio__tool studio__tool--wide" href={publishedUrl} target="_blank" rel="noopener noreferrer">
						<Icon icon={ExternalLink} size={18} /><span>{m.studio_open_published()}</span>
					</a>
				{/if}
			</div>
		{:else if directionList.length > 0}
			<div class="studio__tools">
				<div class="studio__devices" role="group" aria-label="Zoom">
					<button type="button" class="studio__tool" aria-label={m.studio_zoom_out()} title={m.studio_zoom_out()} onclick={zoomOut}><Icon icon={ZoomOut} size={18} /></button>
					<span class="studio__mono studio__zoom-readout">{zoomLabel(zoom)}</span>
					<button type="button" class="studio__tool" aria-label={m.studio_zoom_in()} title={m.studio_zoom_in()} onclick={zoomIn}><Icon icon={ZoomIn} size={18} /></button>
					<button type="button" class="studio__tool studio__tool--wide" onclick={fitCanvas}><Icon icon={Maximize2} size={16} /><span>{m.studio_zoom_fit()}</span></button>
				</div>
			</div>
		{/if}
	</header>

	{#if current && publishError}
		<div class="studio__unpublished" role="status">
			<p class="md-body-small">{m.studio_not_published({ reason: publishError })}</p>
			{#if publishAgainError}<p class="md-body-small studio__error" role="alert">{publishAgainError}</p>{/if}
			<button type="button" class="studio__tool studio__tool--wide" disabled={publishing} onclick={publishAgain}>
				{publishing ? m.studio_publishing() : m.studio_publish_again()}
			</button>
		</div>
	{/if}

	{#if directionSent}
		<div class="studio__unpublished" role="status">
			<p class="md-body-small">{m.studio_refine_sent()}</p>
			<button type="button" class="studio__tool studio__tool--wide" onclick={refresh}>{m.studio_refine_refresh()}</button>
		</div>
	{/if}
	{#if directionError}
		<div class="studio__unpublished" role="alert">
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

	{#if !current || !frameSrc}
		{#if directionList.length > 0}
			<!-- Stage 1: Direction Exploration Canvas before any HTML preview exists -->
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				bind:this={viewportEl}
				class="studio__canvas-viewport"
				onpointerdown={startCanvasPan}
				onpointermove={moveCanvasPan}
				onpointerup={endCanvasPan}
				onpointercancel={endCanvasPan}
				onwheel={handleCanvasWheel}
			>
				<div class="studio__canvas-plane" style:transform={`translate(${panX}px, ${panY}px) scale(${zoom})`}>
					{@render directionBoards()}
				</div>
			</div>
		{:else}
			<div class="studio__empty">
				<h2 class="md-title-large">{m.studio_empty_title()}</h2>
				<p class="md-body-large">{m.studio_empty_body()}</p>
			</div>
		{/if}
	{:else}
		<div class="studio__body">
			<nav class="studio__rail" aria-label={m.studio_versions()}>
				<h2 class="md-label-large studio__label">{m.studio_versions()}</h2>
				<ul class="studio__list">
					{#each versions as version (version.version)}
						<li>
							<button type="button" class="studio__row" aria-current={version.version === current.version ? 'true' : undefined} onclick={() => openVersion(version.version)}>
								<span class="md-label-large">{m.studio_version({ version: version.version })}{#if version.version === approval?.version} · {m.studio_approved()}{/if}{#if version.version === newest} · {m.studio_latest()}{/if}</span>
								<span class="md-body-small studio__muted">{version.title}</span>
								<span class="md-body-small studio__muted">{version.direction_id} · {formatDate(version.created_at, getLocale())}</span>
							</button>
						</li>
					{/each}
				</ul>
				{#if current.pages.length > 1}
					<h2 class="md-label-large studio__label">{m.studio_pages()}</h2>
					<ul class="studio__list">
						{#each current.pages as page (page.path)}
							<li>
								<button type="button" class="studio__row" aria-current={page.path === shownPage ? 'page' : undefined} onclick={() => openPage(page.path)}>
									<span class="md-label-large">{page.title}</span>
									<span class="md-body-small studio__muted">{page.path}</span>
								</button>
							</li>
						{/each}
					</ul>
				{/if}
			</nav>

			<div class="studio__stage" class:studio__stage--device={device !== 'desktop' && mode !== 'canvas'} class:studio__stage--compare={mode === 'compare'} class:studio__stage--canvas={mode === 'canvas'}>
				{#if mode === 'canvas'}
					<div class="studio__canvas-toolbar" role="group" aria-label="Canvas stages">
						<button type="button" class="studio__tool studio__tool--wide" aria-pressed={stageFilter === 'all'} onclick={() => (stageFilter = 'all')}>{m.studio_stage_all()}</button>
						{#if directionList.length > 0}
							<button type="button" class="studio__tool studio__tool--wide" aria-pressed={stageFilter === 'directions'} onclick={() => (stageFilter = 'directions')}>{m.studio_stage_directions()}</button>
						{/if}
						<button type="button" class="studio__tool studio__tool--wide" aria-pressed={stageFilter === 'system'} onclick={() => (stageFilter = 'system')}>{m.studio_stage_system()}</button>
						<button type="button" class="studio__tool studio__tool--wide" aria-pressed={stageFilter === 'flow'} onclick={() => (stageFilter = 'flow')}>{m.studio_stage_flow()}</button>
					</div>
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div
						bind:this={viewportEl}
						class="studio__canvas-viewport"
						onpointerdown={startCanvasPan}
						onpointermove={moveCanvasPan}
						onpointerup={endCanvasPan}
						onpointercancel={endCanvasPan}
						onwheel={handleCanvasWheel}
					>
						<div class="studio__canvas-plane" style:transform={`translate(${panX}px, ${panY}px) scale(${zoom})`}>
							{#if (stageFilter === 'all' || stageFilter === 'directions') && directionList.length > 0}
								{@render directionBoards()}
							{/if}

							{#if stageFilter === 'all' || stageFilter === 'system'}
								<article class="studio__artboard studio__artboard--system" data-artboard="design-system">
									<header class="studio__artboard-bar">
										<span class="studio__artboard-kind">{m.studio_system_board()}</span>
										<strong class="studio__artboard-name">{activeDirection?.name ?? current.direction_id}</strong>
										<code class="studio__mono">v{current.version}</code>
									</header>
									<div class="studio__tile-body">
										{#if activeDirection}
											<div class="studio__tile-section">
												<span class="studio__tile-label">{m.studio_direction_palette()}</span>
												<div class="studio__swatches">
													{#each activeDirection.palette as colour (`${colour.name}-${colour.hex}`)}
														<div class="studio__swatch">
															<span class="studio__swatch-chip" style:background-color={colour.hex}></span>
															<span class="studio__swatch-name">{colour.name}</span>
															<code class="studio__mono">{colour.hex}</code>
														</div>
													{/each}
												</div>
											</div>
											<div class="studio__tile-section">
												<span class="studio__tile-label">{m.studio_direction_typography()}</span>
												<p class="md-body-small"><strong>{activeDirection.typography.heading}</strong> / {activeDirection.typography.body}</p>
											</div>
										{/if}
										<div class="studio__tile-section">
											<span class="studio__tile-label">{m.studio_system_components({ count: current.elements.length })}</span>
											<div class="studio__component-chips">
												{#each current.elements as el (el.id)}
													<button type="button" class="studio__chip" onclick={() => pick(el)}>
														<span>{el.label}</span>
														<code class="studio__mono">{el.id}</code>
													</button>
												{/each}
											</div>
										</div>
										{#each splitPages.systemPages as sysPage (sysPage.path)}
											<div class="studio__system-frame-wrap" style:width={`${canvasBoardWidth}px`}>
												<iframe
													bind:this={canvasFrames[sysPage.path]}
													class="studio__frame studio__frame--canvas"
													title={`${sysPage.title} (${sysPage.path})`}
													src={previewUrl(current, sysPage.path)}
													sandbox="allow-scripts"
													referrerpolicy="no-referrer"
													style:width={`${canvasBoardWidth}px`}
												></iframe>
											</div>
										{/each}
									</div>
								</article>
							{/if}

							{#if stageFilter === 'all' || stageFilter === 'flow'}
								{#each splitPages.flowPages as flowPage, index (flowPage.path)}
									{@const pagePins = annotationsOn(feedback, current, flowPage.path)}
									<article class="studio__artboard studio__artboard--flow" data-artboard-page={flowPage.path} style:width={`${canvasBoardWidth}px`}>
										<header class="studio__artboard-bar">
											<code class="studio__mono">{String(index + 1).padStart(2, '0')}</code>
											<strong class="studio__artboard-name">{flowPage.title}</strong>
											<code class="studio__mono">{flowPage.path}</code>
											{#if pagePins.length > 0}
												<span class="studio__badge">{pagePins.length}</span>
											{/if}
										</header>
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
							{/if}
						</div>
					</div>
				{:else}
					{#if mode === 'compare'}
						{#if compared}
							<div class="studio__side">
								<label class="studio__side-label md-label-large">
									<span>{m.studio_compare_with()}</span>
									<select class="studio__pick md-body-medium" value={String(compared.version)} onchange={(event) => (comparedVersion = Number(event.currentTarget.value))}>
										{#each otherVersions as version (version.version)}
											<option value={String(version.version)}>{m.studio_compare_frame({ version: version.version, title: version.title })}</option>
										{/each}
									</select>
								</label>
								{#key `${compared.preview_token}/${openedPage}`}
									<iframe class="studio__frame" title={m.studio_compare_frame({ version: compared.version, title: compared.title })} src={previewUrl(compared, openedPage)} sandbox="allow-scripts" referrerpolicy="no-referrer" style:width={frameWidth}></iframe>
								{/key}
							</div>
						{:else}
							<p class="md-body-large studio__lone">{m.studio_compare_none()}</p>
						{/if}
					{/if}
					<div class="studio__side">
						{#if mode === 'compare' && compared}<p class="studio__side-label md-label-large">{m.studio_compare_frame({ version: current.version, title: current.title })}</p>{/if}
						{#key `${current.preview_token}/${openedPage}`}
							<iframe
								bind:this={frame}
								class="studio__frame"
								title={m.studio_frame_title({ title: current.title })}
								src={frameSrc}
								sandbox="allow-scripts"
								referrerpolicy="no-referrer"
								style:width={frameWidth}
							></iframe>
						{/key}
					</div>
				{/if}

				{#if pinPopoverOpen && selection && mode !== 'feedback'}
					<div class="studio__pin-popover" role="dialog" aria-label={selection.label}>
						<div class="studio__pin-popover-head">
							<strong>{selection.label}</strong>
							<code class="studio__mono">{selection.id} · {selection.page}</code>
						</div>
						<form class="studio__form" onsubmit={savePinFeedback}>
							<textarea
								class="studio__words md-body-medium"
								rows="3"
								maxlength={FEEDBACK_CHARS}
								placeholder={m.studio_feedback_placeholder()}
								bind:value={pinWords}
							></textarea>
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

				{#if waiting > 0 && mode !== 'feedback'}
					<div class="studio__revise-bar" role="region" aria-label={m.studio_refine_title()}>
						<span class="studio__mono">{m.studio_revise_bar_count({ count: waiting })}</span>
						{#if asked}
							<span class="md-body-small studio__muted" role="status">{m.studio_refine_sent()}</span>
							<button type="button" class="studio__tool studio__tool--wide" onclick={refresh}>{m.studio_refine_refresh()}</button>
						{:else}
							<button type="button" class="studio__send md-label-large" disabled={asking} onclick={() => askSora()}>
								{m.studio_revise_cta()}
							</button>
						{/if}
					</div>
				{/if}
			</div>

			<aside class="studio__panel">
				<h2 class="md-label-large studio__label">{m.studio_selected()}</h2>
				{#if selection}
					<div class="studio__selected">
						<p class="md-title-small">{selection.label}</p>
						<p class="md-body-small studio__muted">{KIND_LABELS[selection.kind]} · <code>{selection.id}</code></p>
						<p class="md-body-small studio__muted">{m.studio_selected_on({ page: selection.page })}</p>
						{#if selection.text}<p class="md-body-medium studio__quote">{selection.text}</p>{/if}
						<button type="button" class="studio__clear md-label-large" onclick={clearSelection}>{m.studio_clear()}</button>
					</div>
				{:else}
					<p class="md-body-medium studio__muted">{picking ? m.studio_select_hint() : m.studio_selected_none()}</p>
				{/if}

				{#if mode === 'feedback'}
					<form class="studio__form" onsubmit={addFeedback}>
						<fieldset class="studio__scopes">
							<legend class="md-label-large studio__label">{m.studio_feedback_about()}</legend>
							{#each SCOPES as option (option.scope)}
								<label class="studio__choice md-body-medium">
									<input type="radio" name="scope" value={option.scope} checked={scope === option.scope} onchange={() => (scope = option.scope)} />
									<span>{option.label}</span>
								</label>
							{/each}
						</fieldset>
						{#if scope === 'element' && !(selection && selection.page === shownPage)}
							<p class="md-body-small studio__muted">{m.studio_feedback_pick_first()}</p>
						{/if}
						{#if scope !== 'global'}
							<label class="studio__choice md-body-medium">
								<input type="checkbox" bind:checked={interaction} />
								<span>{m.studio_feedback_interaction()}</span>
							</label>
						{/if}
						<label class="md-label-large studio__label" for="studio-feedback">{m.studio_feedback_label()}</label>
						<textarea id="studio-feedback" class="studio__words md-body-medium" rows="4" maxlength={FEEDBACK_CHARS} placeholder={m.studio_feedback_placeholder()} bind:value={words}></textarea>
						{#if feedbackError}<p class="md-body-small studio__error" role="alert">{feedbackError}</p>{/if}
						<button type="submit" class="studio__send md-label-large" disabled={!draft || sending}>{m.studio_feedback_add()}</button>
					</form>

					<h2 class="md-label-large studio__label">{m.studio_feedback_history()}</h2>
					{#if history.length === 0}
						<p class="md-body-medium studio__muted">{m.studio_feedback_none()}</p>
					{:else}
						<ol class="studio__list studio__history">
							{#each history as item (item.id)}
								<li>
									<button type="button" class="studio__row" aria-label={m.studio_feedback_show({ number: item.number })} onclick={() => showFeedback(item)}>
										<span class="md-label-large">{m.studio_feedback_number({ number: item.number })} · {where(item)}</span>
										<span class="md-body-medium">{item.body}</span>
										<span class="md-body-small studio__muted">
											{m.studio_feedback_given_on({ version: item.version })} ·
											{item.status === 'resolved' && item.resolved_in_version !== null ? m.studio_feedback_resolved({ version: item.resolved_in_version }) : m.studio_feedback_open()}
										</span>
										{#if item.resolution}<span class="md-body-small studio__quote">{item.resolution}</span>{/if}
									</button>
								</li>
							{/each}
						</ol>
					{/if}

					<h2 class="md-label-large studio__label">{m.studio_refine_title()}</h2>
					<form class="studio__form" onsubmit={askSora}>
						<label class="md-body-small studio__muted" for="studio-ask">{m.studio_refine_label()}</label>
						<textarea id="studio-ask" class="studio__words md-body-medium" rows="2" maxlength={FEEDBACK_CHARS} placeholder={m.studio_refine_placeholder()} bind:value={ask}></textarea>
						{#if askError}<p class="md-body-small studio__error" role="alert">{askError}</p>{/if}
						<button type="submit" class="studio__send md-label-large" disabled={!request || asking}>{m.studio_refine_send()}</button>
						{#if asked}
							<p class="md-body-small studio__muted" role="status">
								{m.studio_refine_sent()}{#if !embedded} <a href={`/chat/${data.sessionId}`}>{m.studio_refine_chat()}</a>{/if}
							</p>
							<button type="button" class="studio__clear md-label-large" onclick={refresh}>{m.studio_refine_refresh()}</button>
						{/if}
					</form>
				{/if}

				<h2 class="md-label-large studio__label">{m.studio_elements()}</h2>
				<ul class="studio__list">
					{#each pageElements as element (element.id)}
						<li>
							<button type="button" class="studio__row" aria-current={selection?.id === element.id ? 'true' : undefined} onclick={() => pick(element)}>
								<span class="md-label-large">{element.label}</span>
								<span class="md-body-small studio__muted">{KIND_LABELS[element.kind]}</span>
							</button>
						</li>
					{/each}
				</ul>
			</aside>
		</div>
	{/if}
</div>

<style>
	.studio {
		display: flex;
		flex-direction: column;
		height: 100dvh;
		min-height: 0;
		overflow-y: auto;
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
	.studio__tool:hover,
	.studio__row:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
	.studio__tool[aria-pressed='true'] {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.studio__back:focus-visible,
	.studio__tool:focus-visible,
	.studio__row:focus-visible,
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
		flex-wrap: wrap;
	}
	.studio__mono {
		font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
		font-size: 12px;
	}
	.studio__zoom-readout {
		min-width: 44px;
		text-align: center;
		color: var(--md-sys-color-on-surface-variant);
	}
	.studio__empty {
		margin: auto;
		padding: 24px;
		max-width: 420px;
		text-align: center;
	}
	.studio__body {
		display: grid;
		grid-template-columns: 240px minmax(0, 1fr) 280px;
		flex: 1;
		min-height: 0;
	}
	.studio__rail,
	.studio__panel {
		padding: 12px;
		overflow-y: auto;
	}
	.studio__rail {
		border-right: 1px solid var(--md-sys-color-outline-variant);
	}
	.studio__panel {
		border-left: 1px solid var(--md-sys-color-outline-variant);
	}
	.studio__label {
		margin: 12px 4px 8px;
		color: var(--md-sys-color-on-surface-variant);
	}
	.studio__label:first-child {
		margin-top: 0;
	}
	.studio__list {
		display: flex;
		flex-direction: column;
		gap: 2px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.studio__row {
		display: flex;
		flex-direction: column;
		gap: 2px;
		width: 100%;
		padding: 8px 12px;
		border: 0;
		border-radius: var(--md-sys-shape-corner-medium);
		background: none;
		color: inherit;
		font: inherit;
		text-align: left;
		cursor: pointer;
	}
	.studio__row[aria-current] {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.studio__row[aria-current] .studio__muted {
		color: inherit;
	}
	.studio__stage {
		position: relative;
		display: flex;
		justify-content: center;
		min-width: 0;
		min-height: 0;
		background: var(--md-sys-color-surface-container);
	}
	.studio__stage--canvas {
		flex-direction: column;
		justify-content: flex-start;
	}
	.studio__canvas-toolbar {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 16px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
		background: var(--md-sys-color-surface);
	}
	.studio__canvas-viewport {
		position: relative;
		flex: 1;
		min-height: 480px;
		overflow: auto;
		padding: 24px;
		cursor: grab;
		background-color: var(--md-sys-color-surface-container);
		background-image: radial-gradient(color-mix(in srgb, var(--md-sys-color-on-surface) 14%, transparent) 1px, transparent 1px);
		background-size: 20px 20px;
	}
	.studio__canvas-viewport:active {
		cursor: grabbing;
	}
	.studio__canvas-plane {
		display: inline-flex;
		align-items: flex-start;
		gap: 24px;
		transform-origin: 0 0;
	}
	.studio__artboard {
		display: flex;
		flex-direction: column;
		width: 380px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface);
		overflow: hidden;
		flex-shrink: 0;
	}
	.studio__artboard--system {
		width: 420px;
	}
	.studio__artboard-bar {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 12px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
		background: var(--md-sys-color-surface-container-low);
	}
	.studio__artboard-kind {
		font-size: 11px;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		color: var(--md-sys-color-on-surface-variant);
	}
	.studio__artboard-name {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.studio__badge {
		padding: 2px 6px;
		border-radius: 4px;
		font-size: 11px;
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.studio__tile-specimen {
		padding: 16px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.studio__tile-card {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 14px;
		border-radius: 8px;
		background: color-mix(in srgb, var(--md-sys-color-surface) 90%, transparent);
		border: 1px solid var(--md-sys-color-outline-variant);
	}
	.studio__tile-heading {
		margin: 0;
		font-size: 20px;
	}
	.studio__tile-summary {
		margin: 0;
		font-size: 13px;
		color: var(--md-sys-color-on-surface-variant);
	}
	.studio__tile-cta {
		align-self: flex-start;
		padding: 4px 10px;
		border-radius: 4px;
		color: #fff;
		font-size: 12px;
		font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
	}
	.studio__tile-body {
		display: flex;
		flex-direction: column;
		gap: 12px;
		padding: 14px;
	}
	.studio__tile-section {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.studio__tile-section p {
		margin: 0;
	}
	.studio__tile-label {
		font-size: 11px;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		color: var(--md-sys-color-on-surface-variant);
	}
	.studio__swatches {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}
	.studio__swatch {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 4px 8px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 6px;
	}
	.studio__swatch-chip {
		width: 14px;
		height: 14px;
		border-radius: 3px;
		border: 1px solid var(--md-sys-color-outline-variant);
	}
	.studio__swatch-name {
		font-size: 12px;
	}
	.studio__tradeoffs {
		margin: 0;
		padding-left: 18px;
	}
	.studio__tile-actions {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-wrap: wrap;
	}
	.studio__component-chips {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}
	.studio__chip {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 4px 8px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 6px;
		background: none;
		color: inherit;
		cursor: pointer;
	}
	.studio__system-frame-wrap {
		margin-top: 8px;
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}
	.studio__frame--canvas {
		height: 680px;
	}
	.studio__pin-popover {
		position: absolute;
		top: 16px;
		right: 16px;
		z-index: 20;
		width: 300px;
		padding: 12px;
		border: 1px solid var(--md-sys-color-outline);
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface);
		box-shadow: 0 8px 24px rgba(0, 0, 0, 0.18);
	}
	.studio__pin-popover-head {
		display: flex;
		flex-direction: column;
		gap: 2px;
		margin-bottom: 8px;
	}
	.studio__revise-bar {
		position: absolute;
		bottom: 16px;
		left: 50%;
		transform: translateX(-50%);
		z-index: 15;
		display: inline-flex;
		align-items: center;
		gap: 12px;
		padding: 8px 16px;
		border: 1px solid var(--md-sys-color-outline);
		border-radius: 999px;
		background: var(--md-sys-color-surface);
		box-shadow: 0 6px 20px rgba(0, 0, 0, 0.16);
	}
	.studio__side {
		display: flex;
		flex: 1;
		flex-direction: column;
		align-items: center;
		min-width: 0;
		min-height: 0;
	}
	.studio__side-label {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		min-height: 40px;
		margin: 0;
		padding: 4px 12px;
		box-sizing: border-box;
		color: var(--md-sys-color-on-surface-variant);
	}
	.studio__pick {
		min-width: 0;
		height: 32px;
		border: 1px solid var(--md-sys-color-outline);
		border-radius: var(--md-sys-shape-corner-small);
		background: var(--md-sys-color-surface);
		color: var(--md-sys-color-on-surface);
	}
	.studio__stage--compare {
		gap: 1px;
	}
	.studio__stage--compare .studio__side + .studio__side {
		border-left: 1px solid var(--md-sys-color-outline-variant);
	}
	.studio__lone {
		margin: auto;
		padding: 24px;
		max-width: 320px;
		text-align: center;
		color: var(--md-sys-color-on-surface-variant);
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
	.studio__confirm .studio__send {
		align-self: center;
	}
	.studio__stage--device {
		padding: 16px;
		overflow-x: auto;
	}
	.studio__frame {
		flex: none;
		max-width: 100%;
		height: 100%;
		border: 0;
		background: #fff;
	}
	.studio__stage--device .studio__frame {
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-large);
	}
	.studio__selected {
		padding: 12px;
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface-container-high);
	}
	.studio__selected p {
		margin: 0 0 4px;
	}
	.studio__quote {
		padding-left: 8px;
		border-left: 2px solid var(--md-sys-color-outline);
	}
	.studio__form {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin-top: 12px;
	}
	.studio__scopes {
		display: flex;
		flex-direction: column;
		gap: 4px;
		margin: 0;
		padding: 0;
		border: 0;
	}
	.studio__scopes legend {
		padding: 0;
	}
	.studio__choice {
		display: flex;
		align-items: center;
		gap: 8px;
		min-height: 32px;
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
	.studio__unpublished {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px 12px;
		padding: 8px 16px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
		background: var(--md-sys-color-surface-container-low);
		color: var(--md-sys-color-on-surface-variant);
	}

	.studio__unpublished p {
		margin: 0;
		flex: 1 1 240px;
	}

	.studio__error {
		margin: 0;
		color: var(--md-sys-color-error);
	}
	.studio__send {
		align-self: flex-start;
		min-height: 40px;
		padding: 0 24px;
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
		margin-top: 4px;
		padding: 6px 0;
		border: 0;
		background: none;
		color: var(--md-sys-color-primary);
		cursor: pointer;
	}
	@container (max-width: 900px) {
		.studio__body {
			grid-template-columns: minmax(0, 1fr);
			flex: none;
		}
		.studio__rail,
		.studio__panel {
			border: 0;
			border-top: 1px solid var(--md-sys-color-outline-variant);
		}
		.studio__stage {
			order: -1;
			height: 70dvh;
			flex: none;
		}
		.studio__stage--compare {
			flex-direction: column;
			height: 140dvh;
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
