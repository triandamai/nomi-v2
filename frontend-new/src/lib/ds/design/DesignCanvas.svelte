<script lang="ts">
	import { ArrowLeft, BadgeCheck, ExternalLink, Maximize2, Monitor, MousePointerClick, Smartphone, Tablet, ZoomIn, ZoomOut } from '@lucide/svelte';
	import { invalidateAll } from '$app/navigation';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import Icon from '$lib/ds/foundations/Icon.svelte';
	import OpenPencilViewer from '$lib/ds/design/OpenPencilViewer.svelte';
	import { buildStudioSceneModel, type StudioHitResult } from '$lib/design/openPencilCanvas';
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
	} from '$lib/design/designStudio';

	// The Design Studio Spatial Canvas (Flow / Build & Chat → Design Studio): Sora's proposed
	// style-tile directions, the active design system board, and multi-page app flow artboards on a
	// 2D pan-and-zoom canvas, plus single-page preview, direct click-to-pin element feedback,
	// version comparison, and approval handoff to Koda.
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
		wanted?: string | null;
		embedded?: boolean;
		onrefresh?: () => Promise<void>;
	} = $props();
	const data = $derived({ sessionId, directions, previews, wanted });

	const ENTRY_PAGE = 'index.html';
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

	let pickedVersion = $state<number | null>(null);
	const current = $derived(pickVersion(data.previews, pickedVersion !== null ? String(pickedVersion) : data.wanted));
	const versions = $derived(versionsNewestFirst(data.previews));
	const newest = $derived(versions[0]?.version ?? null);
	const directionList = $derived(data.directions?.directions ?? []);
	const activeDirection = $derived(activeDirectionForPreview(data.directions, current));
	const splitPages = $derived(current ? splitCanvasPages(current.pages) : { systemPages: [], flowPages: [] });

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

	type Mode = 'preview' | 'canvas' | 'feedback' | 'compare';
	const MODES: { mode: Mode; label: string }[] = [
		{ mode: 'preview', label: m.studio_mode_preview() },
		{ mode: 'canvas', label: m.studio_mode_canvas() },
		{ mode: 'feedback', label: m.studio_mode_feedback() },
		{ mode: 'compare', label: m.studio_mode_compare() },
	];
	let mode = $state<Mode>('canvas');
	const picking = $derived(selecting || mode === 'feedback' || mode === 'canvas');

	// Spatial canvas pan, zoom, stage filter, and direct click-to-pin popover.
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

	type CanvasEngine = 'openpencil' | 'frames';
	let userEngine = $state<CanvasEngine | null>(null);
	const canvasEngine = $derived(userEngine ?? (current?.pages.some((p) => p.path.endsWith('.pen')) ? 'openpencil' : 'frames'));

	const openPencilModel = $derived(
		buildStudioSceneModel({
			directions: directionList,
			preview: current,
			activeDirection,
			device
		})
	);

	function handleOpenPencilHit(hit: StudioHitResult) {
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
			if (mode === 'feedback') {
				scope = 'element';
			} else {
				pinPopoverOpen = true;
			}
		}
	}

	// Stage 1: direction exploration
	let adjustingDirectionId = $state<string | null>(null);
	let directionNote = $state('');
	let sendingDirection = $state(false);
	let directionSent = $state(false);
	let directionError = $state('');

	let added = $state<StudioFeedback[]>([]);
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
		if (event.ctrlKey || event.metaKey) {
			event.preventDefault();
			zoom = clampZoom(zoom + (event.deltaY < 0 ? 0.1 : -0.1));
		}
	}

	const apiBase = $derived(`/chat/${data.sessionId}/api/design`);

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

	let comparedVersion = $state<number | null>(null);
	const compared = $derived(current && mode === 'compare' ? compareWith(data.previews, current, comparedVersion) : null);
	const otherVersions = $derived(current ? versions.filter((v) => v.version !== current.version) : []);

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
			const response = await fetch(`${apiBase}/publish`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ version }) });
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

	let refineWords = $state('');
	let refining = $state(false);
	let refineSent = $state(false);
	let refineError = $state('');
	const refineText = $derived(refineMessage(refineWords, feedback, (numbers) => m.studio_refine_default({ numbers })));

	async function askSora(event?: SubmitEvent) {
		event?.preventDefault();
		if (!refineText || refining) return;
		refining = true;
		refineError = '';
		refineSent = false;
		try {
			const response = await fetch(`${apiBase}/refine`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ text: refineText }) });
			if (!response.ok) {
				refineError = m.studio_refine_failed();
				return;
			}
			refineWords = '';
			refineSent = true;
		} catch {
			refineError = m.studio_refine_failed();
		} finally {
			refining = false;
		}
	}

	async function checkForNewVersion() {
		await (onrefresh ? onrefresh() : invalidateAll());
		pickedVersion = null;
	}

	function showFeedback(item: StudioFeedback) {
		pickedVersion = item.version;
		if (item.page) {
			openedPage = item.page;
			shownPage = item.page;
		}
		if (item.element_id && current) {
			const element = current.elements.find((e) => e.id === item.element_id);
			if (element) selection = { id: element.id, page: item.page ?? shownPage, kind: element.kind, label: element.label, tag: '', text: '', point: item.pin };
			tellAll(highlightMessage(item.element_id));
		}
	}

	function whereLabel(item: StudioFeedback): string {
		const element = item.element_id ? (current?.elements.find((e) => e.id === item.element_id)?.label ?? item.element_id) : null;
		const page = item.page ? (current?.pages.find((p) => p.path === item.page)?.title ?? item.page) : null;
		if (item.level === 'global') return m.studio_feedback_where_global();
		if (item.level === 'page') return m.studio_feedback_where_page({ page: page ?? '' });
		if (item.level === 'interaction') return m.studio_feedback_where_interaction({ target: element ?? page ?? m.studio_feedback_where_global() });
		return m.studio_feedback_where_element({ element: element ?? '', page: page ?? '' });
	}

	function formatShortDate(iso: string): string {
		const parsed = new Date(iso);
		if (Number.isNaN(parsed.getTime())) return iso;
		return new Intl.DateTimeFormat(getLocale(), { month: 'short', day: 'numeric' }).format(parsed);
	}
</script>

<svelte:window onmessage={heard} />

<div class="flex h-full min-h-0 flex-col bg-bg text-text">
	<header class="flex flex-wrap items-center gap-2.5 border-b border-border bg-surface px-4 py-2.5">
		{#if !embedded}
			<a href={`/chat/${data.sessionId}`} class="inline-flex items-center gap-1.5 text-ui text-text-2 hover:text-text">
				<Icon icon={ArrowLeft} size={16} />
				<span>{m.studio_back()}</span>
			</a>
		{/if}
		<div class="flex min-w-0 flex-1 flex-wrap items-baseline gap-2">
			<h1 class="m-0 text-ui font-semibold">{m.studio_title()}</h1>
			{#if current}
				<span class="truncate text-ui text-text-2">{current.title}</span>
				<span class="rounded-xs bg-surface-2 px-2 py-0.5 font-mono text-label text-text-2 outline -outline-offset-1 outline-border">
					{m.studio_direction({ direction: current.direction_id })}
				</span>
			{/if}
		</div>

		{#if current || directionList.length > 0}
			<div class="flex flex-wrap items-center gap-2">
				<div class="inline-flex rounded-sm bg-surface-2 p-0.5 outline -outline-offset-1 outline-border" role="group" aria-label={m.studio_mode()}>
					{#each MODES as option (option.mode)}
						<button
							type="button"
							class={[
								'cursor-pointer rounded-xs px-2.5 py-1 text-caption',
								mode === option.mode ? 'bg-surface font-medium text-text outline -outline-offset-1 outline-border-strong' : 'text-text-2 hover:text-text',
							]}
							aria-pressed={mode === option.mode}
							onclick={() => openMode(option.mode)}
						>
							{option.label}
						</button>
					{/each}
				</div>

				{#if mode === 'canvas'}
					<div class="inline-flex items-center gap-0.5 rounded-sm bg-surface-2 p-0.5 outline -outline-offset-1 outline-border" role="group" aria-label={m.studio_mode_canvas()}>
						<button type="button" class="inline-flex size-6.5 cursor-pointer items-center justify-center rounded-xs text-text-2 hover:text-text" aria-label={m.studio_zoom_out()} onclick={zoomOut}>
							<Icon icon={ZoomOut} size={14} />
						</button>
						<span class="min-w-11 text-center font-mono text-label text-text" data-testid="canvas-zoom-label">{zoomLabel(zoom)}</span>
						<button type="button" class="inline-flex size-6.5 cursor-pointer items-center justify-center rounded-xs text-text-2 hover:text-text" aria-label={m.studio_zoom_in()} onclick={zoomIn}>
							<Icon icon={ZoomIn} size={14} />
						</button>
						<button type="button" class="inline-flex cursor-pointer items-center gap-1 rounded-xs px-2 py-1 font-mono text-label text-text-2 hover:text-text" onclick={fitCanvas}>
							<Icon icon={Maximize2} size={12} />
							<span>{m.studio_zoom_fit()}</span>
						</button>
					</div>
				{/if}

				<div class="inline-flex rounded-sm bg-surface-2 p-0.5 outline -outline-offset-1 outline-border" role="group" aria-label={m.studio_title()}>
					{#each DEVICES as option (option.device)}
						<button
							type="button"
							class={[
								'inline-flex cursor-pointer items-center gap-1 rounded-xs px-2 py-1 text-caption',
								device === option.device ? 'bg-surface font-medium text-text outline -outline-offset-1 outline-border-strong' : 'text-text-2 hover:text-text',
							]}
							aria-pressed={device === option.device}
							onclick={() => (device = option.device)}
						>
							<Icon icon={option.icon} size={14} />
							<span>{option.label}</span>
						</button>
					{/each}
				</div>

				{#if current && mode !== 'compare'}
					<button
						type="button"
						class={[
							'inline-flex cursor-pointer items-center gap-1.5 rounded-sm px-2.5 py-1 text-caption outline -outline-offset-1',
							picking ? 'bg-accent font-medium text-accent-ink outline-accent' : 'bg-surface-2 text-text outline-border hover:outline-border-strong',
						]}
						aria-pressed={picking}
						onclick={toggleSelecting}
					>
						<Icon icon={MousePointerClick} size={14} />
						<span>{m.studio_select()}</span>
					</button>
				{/if}

				{#if current}
					{#if approval?.version === current.version}
						<span class="inline-flex items-center gap-1.5 rounded-sm bg-surface-2 px-2.5 py-1 font-mono text-label text-accent outline -outline-offset-1 outline-border">
							<Icon icon={BadgeCheck} size={14} />
							<span>{m.studio_approved()}</span>
						</span>
					{:else}
						<button
							type="button"
							class="inline-flex cursor-pointer items-center gap-1.5 rounded-sm bg-accent px-2.5 py-1 text-caption font-medium text-accent-ink"
							aria-expanded={confirming}
							onclick={() => (confirming = !confirming)}
						>
							<Icon icon={BadgeCheck} size={14} />
							<span>{m.studio_approve()}</span>
						</button>
					{/if}
				{/if}

				{#if publishedUrl}
					<a
						href={publishedUrl}
						target="_blank"
						rel="noopener noreferrer"
						class="inline-flex items-center gap-1.5 rounded-sm bg-surface-2 px-2.5 py-1 text-caption text-text outline -outline-offset-1 outline-border hover:outline-border-strong"
					>
						<span>{m.studio_open_published()}</span>
						<Icon icon={ExternalLink} size={14} />
					</a>
				{/if}
			</div>
		{/if}
	</header>

	{#if current && !publishedUrl && publishError}
		<div class="flex flex-wrap items-center justify-between gap-3 border-b border-border bg-surface-2 px-4 py-2 text-caption" role="status">
			<span class="text-text-2">{m.studio_not_published({ reason: publishError })}</span>
			<div class="flex items-center gap-2.5">
				{#if publishAgainError}
					<span class="text-danger" role="alert">{publishAgainError}</span>
				{/if}
				<button
					type="button"
					class="cursor-pointer rounded-sm bg-surface px-2.5 py-1 text-caption text-text outline -outline-offset-1 outline-border hover:outline-border-strong disabled:opacity-50"
					disabled={publishing}
					onclick={publishAgain}
				>
					{publishing ? m.studio_publishing() : m.studio_publish_again()}
				</button>
			</div>
		</div>
	{/if}

	{#if current && confirming && approval?.version !== current.version}
		<section class="flex flex-wrap items-center justify-between gap-3 border-b border-border bg-surface-2 px-4 py-3 text-ui" aria-label={m.studio_approve()}>
			<div class="flex flex-col gap-0.5">
				<strong class="font-medium text-text">{m.studio_approve_question({ version: current.version })}</strong>
				<p class="m-0 text-caption text-text-2">{m.studio_approve_explain()}</p>
				{#if waiting > 0}
					<p class="m-0 font-mono text-label text-text-2">{m.studio_approve_waiting({ count: waiting })}</p>
				{/if}
				{#if approveError}
					<p class="m-0 text-caption text-danger" role="alert">{approveError}</p>
				{/if}
			</div>
			<div class="flex items-center gap-2">
				<button
					type="button"
					class="cursor-pointer rounded-sm bg-accent px-3 py-1.5 text-caption font-medium text-accent-ink disabled:opacity-50"
					disabled={approving}
					onclick={approve}
				>
					{m.studio_approve_confirm({ version: current.version })}
				</button>
				<button
					type="button"
					class="cursor-pointer rounded-sm bg-surface px-3 py-1.5 text-caption text-text outline -outline-offset-1 outline-border hover:outline-border-strong"
					onclick={() => (confirming = false)}
				>
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
	{:else if !current && directionList.length > 0}
		<!-- Stage 1 only: Sora proposed style directions before the first HTML preview is built -->
		<div class="flex min-h-0 flex-1 flex-col overflow-auto p-5">
			<div class="mb-4 flex flex-wrap items-center justify-between gap-2">
				<span class="font-mono text-label tracking-label text-text-3 uppercase">{m.studio_stage_directions()}</span>
				{#if directionSent}
					<div class="flex items-center gap-2 text-caption text-text-2" role="status">
						<span>{m.studio_refine_sent()}</span>
						<button type="button" class="cursor-pointer rounded-xs bg-surface px-2 py-1 text-caption text-text outline -outline-offset-1 outline-border" onclick={checkForNewVersion}>
							{m.studio_refine_refresh()}
						</button>
					</div>
				{/if}
				{#if directionError}
					<span class="text-caption text-danger" role="alert">{directionError}</span>
				{/if}
			</div>
			<div class="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
				{#each directionList as dir (dir.id)}
					<article class="flex flex-col gap-3 rounded-md bg-surface p-4 outline -outline-offset-1 outline-border" data-testid={`direction-board-${dir.id}`}>
						<div class="flex items-start justify-between gap-2">
							<div>
								<h2 class="m-0 text-body font-semibold text-text">{dir.name}</h2>
								<span class="font-mono text-label text-text-3">{dir.id}</span>
							</div>
							{#if dir.recommended}
								<span class="rounded-xs bg-surface-2 px-2 py-0.5 font-mono text-label text-accent outline -outline-offset-1 outline-border">{m.studio_direction_recommended()}</span>
							{/if}
						</div>
						<p class="m-0 text-ui text-text-2">{dir.summary}</p>

						{#if dir.palette.length > 0}
							<div class="flex flex-col gap-1.5">
								<span class="font-mono text-label tracking-label text-text-3 uppercase">{m.studio_direction_palette()}</span>
								<div class="flex flex-wrap gap-2">
									{#each dir.palette as swatch (`${dir.id}-${swatch.name}-${swatch.hex}`)}
										<div class="flex items-center gap-1.5 rounded-xs bg-surface-2 px-2 py-1 outline -outline-offset-1 outline-border">
											<span class="size-3.5 rounded-xs outline -outline-offset-1 outline-border" style:background-color={swatch.hex}></span>
											<span class="text-caption text-text">{swatch.name}</span>
											<span class="font-mono text-label text-text-3">{swatch.hex}</span>
										</div>
									{/each}
								</div>
							</div>
						{/if}

						<div class="grid grid-cols-2 gap-2 rounded-sm bg-surface-2 p-2.5 outline -outline-offset-1 outline-border">
							<div class="flex flex-col gap-0.5">
								<span class="font-mono text-label text-text-3 uppercase">{m.studio_direction_typography()}</span>
								<span class="text-caption font-medium text-text">{dir.typography.heading}</span>
								<span class="text-caption text-text-2">{dir.typography.body}</span>
							</div>
							<div class="flex flex-col gap-0.5">
								<span class="font-mono text-label text-text-3 uppercase">{m.studio_direction_layout()}</span>
								<span class="text-caption text-text-2">{dir.layout}</span>
							</div>
						</div>

						<div class="flex flex-col gap-1">
							<span class="font-mono text-label text-text-3 uppercase">{m.studio_direction_rationale()}</span>
							<p class="m-0 text-caption text-text-2">{dir.rationale}</p>
						</div>

						{#if dir.trade_offs.length > 0}
							<div class="flex flex-col gap-1">
								<span class="font-mono text-label text-text-3 uppercase">{m.studio_direction_tradeoffs()}</span>
								<ul class="m-0 list-disc pl-4 text-caption text-text-2">
									{#each dir.trade_offs as item (item)}
										<li>{item}</li>
									{/each}
								</ul>
							</div>
						{/if}

						<div class="mt-auto flex flex-wrap items-center gap-2 pt-2">
							<button
								type="button"
								class="cursor-pointer rounded-sm bg-accent px-3 py-1.5 text-caption font-medium text-accent-ink disabled:opacity-50"
								disabled={sendingDirection}
								onclick={() => useDirection(dir)}
							>
								{m.studio_direction_use()}
							</button>
							<button
								type="button"
								class="cursor-pointer rounded-sm bg-surface-2 px-3 py-1.5 text-caption text-text outline -outline-offset-1 outline-border hover:outline-border-strong"
								onclick={() => {
									adjustingDirectionId = adjustingDirectionId === dir.id ? null : dir.id;
									directionNote = '';
								}}
							>
								{m.studio_direction_adjust()}
							</button>
						</div>

						{#if adjustingDirectionId === dir.id}
							<form class="flex flex-col gap-2 border-t border-border pt-2.5" onsubmit={(e) => submitDirectionAdjustment(e, dir)}>
								<textarea
									rows="2"
									class="w-full rounded-sm bg-bg p-2 text-caption text-text outline -outline-offset-1 outline-border focus:outline-border-strong"
									bind:value={directionNote}
									placeholder={m.studio_direction_adjust_placeholder()}
								></textarea>
								<button
									type="submit"
									class="self-start cursor-pointer rounded-sm bg-surface-2 px-3 py-1 text-caption font-medium text-text outline -outline-offset-1 outline-border-strong disabled:opacity-50"
									disabled={!directionNote.trim() || sendingDirection}
								>
									{m.studio_direction_send()}
								</button>
							</form>
						{/if}
					</article>
				{/each}
			</div>
		</div>
	{:else if current}
		<div class="grid min-h-0 flex-1 grid-cols-1 md:grid-cols-4">
			<aside class="flex flex-col gap-4 overflow-y-auto border-b border-border bg-surface p-3.5 md:col-span-1 md:border-r md:border-b-0">
				<section class="flex flex-col gap-2" aria-label={m.studio_versions()}>
					<h2 class="m-0 font-mono text-label tracking-label text-text-3 uppercase">{m.studio_versions()}</h2>
					<ul class="m-0 flex list-none flex-col gap-1 p-0">
						{#each versions as v (v.version)}
							<li>
								<button
									type="button"
									class={[
										'flex w-full cursor-pointer flex-col gap-0.5 rounded-sm p-2 text-left outline -outline-offset-1',
										v.version === current.version ? 'bg-surface-2 outline-border-strong' : 'bg-transparent outline-transparent hover:bg-surface-2',
									]}
									aria-current={v.version === current.version ? 'true' : undefined}
									onclick={() => openVersion(v.version)}
								>
									<span class="flex items-center justify-between gap-1.5 font-mono text-label text-text">
										<span>{m.studio_version({ version: v.version })}</span>
										{#if approval?.version === v.version}
											<span class="rounded-xs bg-surface px-1.5 py-0.5 font-mono text-micro text-accent outline -outline-offset-1 outline-border">{m.studio_approved()}</span>
										{:else if v.version === newest}
											<span class="rounded-xs bg-surface px-1.5 py-0.5 font-mono text-micro text-text-2 outline -outline-offset-1 outline-border">{m.studio_latest()}</span>
										{/if}
									</span>
									<span class="truncate text-caption text-text">{v.title}</span>
									<span class="font-mono text-label text-text-3">{v.direction_id} · {formatShortDate(v.created_at)}</span>
								</button>
							</li>
						{/each}
					</ul>
				</section>

				<section class="flex flex-col gap-2" aria-label={m.studio_pages()}>
					<h2 class="m-0 font-mono text-label tracking-label text-text-3 uppercase">{m.studio_pages()}</h2>
					<ul class="m-0 flex list-none flex-col gap-1 p-0">
						{#each current.pages as p (p.path)}
							<li>
								<button
									type="button"
									class={[
										'flex w-full cursor-pointer flex-col gap-0.5 rounded-sm p-2 text-left outline -outline-offset-1',
										p.path === shownPage ? 'bg-surface-2 outline-border-strong' : 'bg-transparent outline-transparent hover:bg-surface-2',
									]}
									aria-current={p.path === shownPage ? 'page' : undefined}
									onclick={() => openPage(p.path)}
								>
									<span class="text-caption text-text">{p.title}</span>
									<span class="font-mono text-label text-text-3">{p.path}</span>
								</button>
							</li>
						{/each}
					</ul>
				</section>

				{#if mode === 'compare'}
					<section class="flex flex-col gap-2" aria-label={m.studio_compare_with()}>
						<h2 class="m-0 font-mono text-label tracking-label text-text-3 uppercase">{m.studio_compare_with()}</h2>
						{#if otherVersions.length === 0}
							<p class="m-0 text-caption text-text-2">{m.studio_compare_none()}</p>
						{:else}
							<ul class="m-0 flex list-none flex-col gap-1 p-0">
								{#each otherVersions as v (v.version)}
									<li>
										<button
											type="button"
											class={[
												'flex w-full cursor-pointer flex-col gap-0.5 rounded-sm p-2 text-left outline -outline-offset-1',
												compared?.version === v.version ? 'bg-surface-2 outline-border-strong' : 'bg-transparent outline-transparent hover:bg-surface-2',
											]}
											aria-pressed={compared?.version === v.version}
											onclick={() => (comparedVersion = v.version)}
										>
											<span class="font-mono text-label text-text">{m.studio_version({ version: v.version })}</span>
											<span class="truncate text-caption text-text">{v.title}</span>
										</button>
									</li>
								{/each}
							</ul>
						{/if}
					</section>
				{:else if mode === 'feedback'}
					<section class="flex flex-col gap-2.5" aria-label={m.studio_mode_feedback()}>
						<h2 class="m-0 font-mono text-label tracking-label text-text-3 uppercase">{m.studio_mode_feedback()}</h2>
						<form class="flex flex-col gap-2.5" onsubmit={addFeedback}>
							<fieldset class="m-0 flex flex-col gap-1.5 border-0 p-0">
								<legend class="mb-1 text-caption text-text-2">{m.studio_feedback_about()}</legend>
								<div class="flex flex-col gap-1">
									{#each SCOPES as option (option.scope)}
										<button
											type="button"
											class={[
												'cursor-pointer rounded-sm px-2.5 py-1.5 text-left text-caption outline -outline-offset-1',
												scope === option.scope ? 'bg-surface-2 font-medium text-text outline-border-strong' : 'bg-bg text-text-2 outline-border hover:text-text',
											]}
											aria-pressed={scope === option.scope}
											onclick={() => (scope = option.scope)}
										>
											{option.label}
										</button>
									{/each}
								</div>
							</fieldset>

							{#if scope === 'element'}
								{#if selection && selection.page === shownPage}
									<div class="flex flex-col gap-0.5 rounded-sm bg-surface-2 p-2 outline -outline-offset-1 outline-border">
										<span class="font-mono text-label text-text-3">{KIND_LABELS[selection.kind]}</span>
										<strong class="text-caption font-medium text-text">{selection.label}</strong>
										<span class="font-mono text-label text-text-3">{selection.id}</span>
									</div>
								{:else}
									<p class="m-0 text-caption text-text-2">{m.studio_feedback_pick_first()}</p>
								{/if}
							{/if}

							{#if scope !== 'global'}
								<label class="flex items-center gap-2 text-caption text-text-2">
									<input type="checkbox" bind:checked={interaction} />
									<span>{m.studio_feedback_interaction()}</span>
								</label>
							{/if}

							<label class="flex flex-col gap-1 text-caption text-text-2">
								<span>{m.studio_feedback_label()}</span>
								<textarea
									rows="3"
									maxlength={FEEDBACK_CHARS}
									class="w-full rounded-sm bg-bg p-2 text-caption text-text outline -outline-offset-1 outline-border focus:outline-border-strong"
									bind:value={words}
									placeholder={m.studio_feedback_placeholder()}
								></textarea>
							</label>

							{#if feedbackError}
								<p class="m-0 text-caption text-danger" role="alert">{feedbackError}</p>
							{/if}

							<button
								type="submit"
								class="cursor-pointer rounded-sm bg-accent px-3 py-1.5 text-caption font-medium text-accent-ink disabled:opacity-50"
								disabled={!draft || sending}
							>
								{m.studio_feedback_add()}
							</button>
						</form>
					</section>

					<section class="flex flex-col gap-2" aria-label={m.studio_refine_title()}>
						<h2 class="m-0 font-mono text-label tracking-label text-text-3 uppercase">{m.studio_refine_title()}</h2>
						<form class="flex flex-col gap-2" onsubmit={(e) => askSora(e)}>
							<label class="flex flex-col gap-1 text-caption text-text-2">
								<span>{m.studio_refine_label()}</span>
								<textarea
									rows="2"
									maxlength={FEEDBACK_CHARS}
									class="w-full rounded-sm bg-bg p-2 text-caption text-text outline -outline-offset-1 outline-border focus:outline-border-strong"
									bind:value={refineWords}
									placeholder={m.studio_refine_placeholder()}
								></textarea>
							</label>
							{#if refineError}
								<p class="m-0 text-caption text-danger" role="alert">{refineError}</p>
							{/if}
							{#if refineSent}
								<div class="flex flex-col gap-1.5 rounded-sm bg-surface-2 p-2 text-caption text-text-2 outline -outline-offset-1 outline-border" role="status">
									<p class="m-0">{m.studio_refine_sent()}</p>
									<div class="flex flex-wrap items-center gap-2">
										<button type="button" class="cursor-pointer rounded-xs bg-surface px-2 py-1 text-caption text-text outline -outline-offset-1 outline-border" onclick={checkForNewVersion}>
											{m.studio_refine_refresh()}
										</button>
										{#if !embedded}
											<a href={`/chat/${data.sessionId}`} class="text-caption text-accent">{m.studio_refine_chat()}</a>
										{/if}
									</div>
								</div>
							{/if}
							<button
								type="submit"
								class="cursor-pointer rounded-sm bg-surface-2 px-3 py-1.5 text-caption font-medium text-text outline -outline-offset-1 outline-border-strong disabled:opacity-50"
								disabled={!refineText || refining}
							>
								{m.studio_refine_send()}
							</button>
						</form>
					</section>

					<section class="flex flex-col gap-2" aria-label={m.studio_feedback_history()}>
						<h2 class="m-0 font-mono text-label tracking-label text-text-3 uppercase">{m.studio_feedback_history()}</h2>
						{#if history.length === 0}
							<p class="m-0 text-caption text-text-2">{m.studio_feedback_none()}</p>
						{:else}
							<ul class="m-0 flex list-none flex-col gap-1.5 p-0">
								{#each history as item (item.id)}
									<li>
										<button
											type="button"
											class="flex w-full cursor-pointer flex-col gap-1 rounded-sm bg-surface-2 p-2 text-left outline -outline-offset-1 outline-border hover:outline-border-strong"
											aria-label={m.studio_feedback_show({ number: item.number })}
											onclick={() => showFeedback(item)}
										>
											<span class="flex items-center justify-between gap-1.5">
												<span class="font-mono text-label text-accent">{m.studio_feedback_number({ number: item.number })}</span>
												<span class="font-mono text-micro text-text-2">
													{item.status === 'resolved' && item.resolved_in_version !== null
														? m.studio_feedback_resolved({ version: item.resolved_in_version })
														: m.studio_feedback_open()}
												</span>
											</span>
											<span class="text-caption font-medium text-text">{whereLabel(item)}</span>
											<span class="text-caption text-text-2">{item.body}</span>
											{#if item.resolution}
												<span class="font-mono text-label text-text-3">{item.resolution}</span>
											{/if}
											<span class="font-mono text-label text-text-3">{m.studio_feedback_given_on({ version: item.version })}</span>
										</button>
									</li>
								{/each}
							</ul>
						{/if}
					</section>
				{:else}
					<section class="flex flex-col gap-2" aria-label={m.studio_selected()}>
						<div class="flex items-center justify-between">
							<h2 class="m-0 font-mono text-label tracking-label text-text-3 uppercase">{m.studio_selected()}</h2>
							{#if selection}
								<button
									type="button"
									class="cursor-pointer text-caption text-text-2 hover:text-text"
									onclick={() => {
										selection = null;
										pinPopoverOpen = false;
										tellAll(highlightMessage(null));
									}}
								>
									{m.studio_clear()}
								</button>
							{/if}
						</div>
						{#if selection}
							<div class="flex flex-col gap-1 rounded-sm bg-surface-2 p-2.5 outline -outline-offset-1 outline-border">
								<span class="font-mono text-label text-text-3">{KIND_LABELS[selection.kind]}</span>
								<strong class="text-caption font-medium text-text">{selection.label}</strong>
								<span class="font-mono text-label text-text-3">{selection.id}</span>
								<span class="text-caption text-text-2">{m.studio_selected_on({ page: selection.page })}</span>
								{#if selection.text}
									<p class="m-0 truncate text-caption text-text-2">“{selection.text}”</p>
								{/if}
							</div>
						{:else}
							<p class="m-0 text-caption text-text-2">{selecting ? m.studio_select_hint() : m.studio_selected_none()}</p>
						{/if}
					</section>
				{/if}

				{#if mode !== 'compare'}
					<section class="flex flex-col gap-2" aria-label={m.studio_elements()}>
						<h2 class="m-0 font-mono text-label tracking-label text-text-3 uppercase">{m.studio_elements()}</h2>
						<ul class="m-0 flex list-none flex-col gap-1 p-0">
							{#each pageElements as el (el.id)}
								<li>
									<button
										type="button"
										class={[
											'flex w-full cursor-pointer flex-col gap-0.5 rounded-sm p-2 text-left outline -outline-offset-1',
											selection?.id === el.id ? 'bg-surface-2 outline-border-strong' : 'bg-transparent outline-transparent hover:bg-surface-2',
										]}
										onclick={() => pick(el)}
									>
										<span class="text-caption text-text">{el.label}</span>
										<span class="font-mono text-label text-text-3">{KIND_LABELS[el.kind]} · {el.id}</span>
									</button>
								</li>
							{/each}
						</ul>
					</section>
				{/if}
			</aside>

			<div class="relative flex min-h-0 flex-col bg-bg md:col-span-3">
				{#if mode === 'compare'}
					{#if compared}
						<div class="grid min-h-0 flex-1 grid-cols-1 gap-4 overflow-auto p-4 lg:grid-cols-2">
							<div class="flex min-h-0 flex-col gap-2">
								<div class="font-mono text-label text-text-2">{m.studio_compare_frame({ version: current.version, title: current.title })}</div>
								<iframe
									class="min-h-120 w-full flex-1 rounded-md bg-surface outline -outline-offset-1 outline-border"
									style:max-width={frameWidth}
									src={previewUrl(current, shownPage)}
									sandbox="allow-scripts"
									referrerpolicy="no-referrer"
									title={m.studio_compare_frame({ version: current.version, title: current.title })}
								></iframe>
							</div>
							<div class="flex min-h-0 flex-col gap-2">
								<div class="font-mono text-label text-text-2">{m.studio_compare_frame({ version: compared.version, title: compared.title })}</div>
								<iframe
									class="min-h-120 w-full flex-1 rounded-md bg-surface outline -outline-offset-1 outline-border"
									style:max-width={frameWidth}
									src={previewUrl(compared, shownPage)}
									sandbox="allow-scripts"
									referrerpolicy="no-referrer"
									title={m.studio_compare_frame({ version: compared.version, title: compared.title })}
								></iframe>
							</div>
						</div>
					{:else}
						<div class="m-auto p-8 text-center text-ui text-text-2">{m.studio_compare_none()}</div>
					{/if}
				{:else if mode === 'canvas'}
					<!-- 2D Spatial Canvas: Stage filter bar + zoomable/pannable artboard plane -->
					<div class="flex flex-wrap items-center justify-between gap-2 border-b border-border bg-surface px-4 py-2">
						<div class="flex flex-wrap items-center gap-2">
							<div class="inline-flex rounded-sm bg-surface-2 p-0.5 outline -outline-offset-1 outline-border" role="group" aria-label={m.studio_mode_canvas()}>
								{#each [
									{ id: 'all' as const, label: m.studio_stage_all() },
									...(directionList.length > 0 ? [{ id: 'directions' as const, label: m.studio_stage_directions() }] : []),
									{ id: 'system' as const, label: m.studio_stage_system() },
									{ id: 'flow' as const, label: m.studio_stage_flow() },
								] as tab (tab.id)}
									<button
										type="button"
										class={[
											'cursor-pointer rounded-xs px-2.5 py-1 text-caption',
											stageFilter === tab.id ? 'bg-surface font-medium text-text outline -outline-offset-1 outline-border-strong' : 'text-text-2 hover:text-text',
										]}
										aria-pressed={stageFilter === tab.id}
										onclick={() => (stageFilter = tab.id)}
									>
										{tab.label}
									</button>
								{/each}
							</div>

							<div class="inline-flex rounded-sm bg-surface-2 p-0.5 outline -outline-offset-1 outline-border" role="group" aria-label={m.studio_canvas_openpencil()}>
								<button
									type="button"
									class={[
										'cursor-pointer rounded-xs px-2.5 py-1 text-caption',
										canvasEngine === 'openpencil' ? 'bg-surface font-medium text-text outline -outline-offset-1 outline-border-strong' : 'text-text-2 hover:text-text',
									]}
									aria-pressed={canvasEngine === 'openpencil'}
									onclick={() => (userEngine = 'openpencil')}
								>
									{m.studio_canvas_openpencil()}
								</button>
								<button
									type="button"
									class={[
										'cursor-pointer rounded-xs px-2.5 py-1 text-caption',
										canvasEngine === 'frames' ? 'bg-surface font-medium text-text outline -outline-offset-1 outline-border-strong' : 'text-text-2 hover:text-text',
									]}
									aria-pressed={canvasEngine === 'frames'}
									onclick={() => (userEngine = 'frames')}
								>
									{m.studio_canvas_frames()}
								</button>
							</div>
						</div>
						<span class="font-mono text-label text-text-3">{m.studio_select_hint()}</span>
					</div>

					{#if canvasEngine === 'openpencil'}
						<OpenPencilViewer
							model={openPencilModel}
							bind:zoom
							bind:panX
							bind:panY
							selectedNodeId={selection?.id ?? null}
							onhit={handleOpenPencilHit}
							onpickDirection={(dirId) => {
								const dir = directionList.find((d) => d.id === dirId);
								if (dir) useDirection(dir);
							}}
						/>
					{:else}
						<!-- svelte-ignore a11y_no_static_element_interactions -->
						<div
							bind:this={viewportEl}
						class="relative min-h-0 flex-1 overflow-auto p-6"
						data-testid="spatial-canvas-viewport"
						onpointerdown={startCanvasPan}
						onpointermove={moveCanvasPan}
						onpointerup={endCanvasPan}
						onpointercancel={endCanvasPan}
						onwheel={handleCanvasWheel}
					>
						<div
							class="flex flex-col gap-8"
							data-testid="spatial-canvas-plane"
							style:transform={`translate(${panX}px, ${panY}px) scale(${zoom})`}
							style:transform-origin="0 0"
						>
							{#if (stageFilter === 'all' || stageFilter === 'directions') && directionList.length > 0}
								<section class="flex flex-col gap-3" aria-label={m.studio_stage_directions()}>
									<span class="font-mono text-label tracking-label text-text-3 uppercase">{m.studio_stage_directions()}</span>
									<div class="flex flex-wrap items-start gap-6">
										{#each directionList as dir (dir.id)}
											<article
												class={[
													'flex w-88 shrink-0 flex-col gap-3 rounded-md bg-surface p-4 outline -outline-offset-1',
													current.direction_id.toLowerCase() === dir.id.toLowerCase() ? 'outline-border-strong' : 'outline-border',
												]}
												data-testid={`direction-board-${dir.id}`}
											>
												<div class="flex items-start justify-between gap-2">
													<div>
														<h3 class="m-0 text-ui font-semibold text-text">{dir.name}</h3>
														<span class="font-mono text-label text-text-3">{dir.id}</span>
													</div>
													{#if dir.recommended}
														<span class="rounded-xs bg-surface-2 px-1.5 py-0.5 font-mono text-micro text-accent outline -outline-offset-1 outline-border">
															{m.studio_direction_recommended()}
														</span>
													{/if}
												</div>
												<p class="m-0 text-caption text-text-2">{dir.summary}</p>
												{#if dir.palette.length > 0}
													<div class="flex flex-wrap gap-1.5">
														{#each dir.palette as swatch (`${dir.id}-${swatch.name}-${swatch.hex}`)}
															<div class="flex items-center gap-1.5 rounded-xs bg-surface-2 px-2 py-1 outline -outline-offset-1 outline-border">
																<span class="size-3 rounded-xs outline -outline-offset-1 outline-border" style:background-color={swatch.hex}></span>
																<span class="text-caption text-text">{swatch.name}</span>
																<span class="font-mono text-micro text-text-3">{swatch.hex}</span>
															</div>
														{/each}
													</div>
												{/if}
												<div class="flex items-center justify-between gap-2 border-t border-border pt-2 font-mono text-label text-text-3">
													<span>{dir.typography.heading} / {dir.typography.body}</span>
												</div>
												<div class="flex flex-wrap items-center gap-2">
													<button
														type="button"
														class="cursor-pointer rounded-sm bg-accent px-2.5 py-1 text-caption font-medium text-accent-ink disabled:opacity-50"
														disabled={sendingDirection}
														onclick={() => useDirection(dir)}
													>
														{m.studio_direction_use()}
													</button>
													<button
														type="button"
														class="cursor-pointer rounded-sm bg-surface-2 px-2.5 py-1 text-caption text-text outline -outline-offset-1 outline-border hover:outline-border-strong"
														onclick={() => {
															adjustingDirectionId = adjustingDirectionId === dir.id ? null : dir.id;
															directionNote = '';
														}}
													>
														{m.studio_direction_adjust()}
													</button>
												</div>
												{#if adjustingDirectionId === dir.id}
													<form class="flex flex-col gap-2 border-t border-border pt-2" onsubmit={(e) => submitDirectionAdjustment(e, dir)}>
														<textarea
															rows="2"
															class="w-full rounded-sm bg-bg p-2 text-caption text-text outline -outline-offset-1 outline-border focus:outline-border-strong"
															bind:value={directionNote}
															placeholder={m.studio_direction_adjust_placeholder()}
														></textarea>
														<button
															type="submit"
															class="self-start cursor-pointer rounded-sm bg-surface-2 px-2.5 py-1 text-caption font-medium text-text outline -outline-offset-1 outline-border-strong disabled:opacity-50"
															disabled={!directionNote.trim() || sendingDirection}
														>
															{m.studio_direction_send()}
														</button>
													</form>
												{/if}
											</article>
										{/each}
									</div>
								</section>
							{/if}

							<div class="flex items-start gap-8">
								{#if stageFilter === 'all' || stageFilter === 'system'}
									<section class="flex shrink-0 flex-col gap-3" aria-label={m.studio_system_board()} data-testid="design-system-artboard">
										<div class="flex items-center justify-between gap-2">
											<span class="font-mono text-label tracking-label text-text-3 uppercase">{m.studio_system_board()}</span>
											<span class="font-mono text-label text-text-3">{current.direction_id}</span>
										</div>
										{#if splitPages.systemPages.length > 0}
											{#each splitPages.systemPages as sysPage (sysPage.path)}
												<div class="flex flex-col gap-2 rounded-md bg-surface p-3 outline -outline-offset-1 outline-border" style:width={`${canvasBoardWidth}px`}>
													<div class="flex items-center justify-between font-mono text-label text-text-2">
														<span>{sysPage.title}</span>
														<span class="text-text-3">{sysPage.path}</span>
													</div>
													<iframe
														bind:this={canvasFrames[sysPage.path]}
														class="h-140 w-full rounded-sm bg-bg outline -outline-offset-1 outline-border"
														src={previewUrl(current, sysPage.path)}
														sandbox="allow-scripts"
														referrerpolicy="no-referrer"
														title={m.studio_frame_title({ title: sysPage.title })}
													></iframe>
												</div>
											{/each}
										{:else}
											<article class="flex w-88 flex-col gap-3 rounded-md bg-surface p-4 outline -outline-offset-1 outline-border">
												<div class="flex items-center justify-between">
													<h3 class="m-0 text-ui font-semibold text-text">{activeDirection?.name ?? current.title}</h3>
													<span class="font-mono text-label text-text-3">v{current.version}</span>
												</div>
												{#if activeDirection}
													{#if activeDirection.palette.length > 0}
														<div class="flex flex-col gap-1.5">
															<span class="font-mono text-label text-text-3 uppercase">{m.studio_direction_palette()}</span>
															<div class="flex flex-wrap gap-1.5">
																{#each activeDirection.palette as swatch (`sys-${swatch.name}-${swatch.hex}`)}
																	<div class="flex items-center gap-1.5 rounded-xs bg-surface-2 px-2 py-1 outline -outline-offset-1 outline-border">
																		<span class="size-3 rounded-xs outline -outline-offset-1 outline-border" style:background-color={swatch.hex}></span>
																		<span class="text-caption text-text">{swatch.name}</span>
																		<span class="font-mono text-micro text-text-3">{swatch.hex}</span>
																	</div>
																{/each}
															</div>
														</div>
													{/if}
													<div class="flex flex-col gap-1 rounded-sm bg-surface-2 p-2.5 outline -outline-offset-1 outline-border">
														<span class="font-mono text-label text-text-3 uppercase">{m.studio_direction_typography()}</span>
														<span class="text-caption font-medium text-text">{activeDirection.typography.heading}</span>
														<span class="text-caption text-text-2">{activeDirection.typography.body}</span>
													</div>
												{/if}
												<div class="flex flex-col gap-1.5">
													<span class="font-mono text-label text-text-3 uppercase">
														{m.studio_system_components({ count: current.elements.length })}
													</span>
													<div class="flex flex-wrap gap-1.5">
														{#each current.elements as el (el.id)}
															<button
																type="button"
																class={[
																	'cursor-pointer rounded-xs px-2 py-1 text-left font-mono text-label outline -outline-offset-1',
																	selection?.id === el.id ? 'bg-surface-2 text-text outline-border-strong' : 'bg-bg text-text-2 outline-border hover:text-text',
																]}
																onclick={() => pick(el)}
															>
																{el.label}
															</button>
														{/each}
													</div>
												</div>
											</article>
										{/if}
									</section>
								{/if}

								{#if stageFilter === 'all' || stageFilter === 'flow'}
									<section class="flex flex-col gap-3" aria-label={m.studio_stage_flow()}>
										<span class="font-mono text-label tracking-label text-text-3 uppercase">{m.studio_stage_flow()}</span>
										<div class="flex items-start gap-6">
											{#each splitPages.flowPages as flowPage (flowPage.path)}
												<div
													class={[
														'flex shrink-0 flex-col gap-2 rounded-md bg-surface p-3 outline -outline-offset-1',
														shownPage === flowPage.path ? 'outline-border-strong' : 'outline-border',
													]}
													style:width={`${canvasBoardWidth}px`}
													data-testid={`flow-artboard-${flowPage.path}`}
												>
													<div class="flex items-center justify-between gap-2">
														<button
															type="button"
															class="cursor-pointer text-left text-caption font-medium text-text hover:underline"
															onclick={() => openPage(flowPage.path)}
														>
															{flowPage.title}
														</button>
														<span class="font-mono text-label text-text-3">{flowPage.path} · {canvasBoardWidth}px</span>
													</div>
													<iframe
														bind:this={canvasFrames[flowPage.path]}
														class="h-150 w-full rounded-sm bg-bg outline -outline-offset-1 outline-border"
														src={previewUrl(current, flowPage.path)}
														sandbox="allow-scripts"
														referrerpolicy="no-referrer"
														title={m.studio_frame_title({ title: flowPage.title })}
													></iframe>
												</div>
											{/each}
										</div>
									</section>
								{/if}
							</div>
						</div>
					</div>
					{/if}
				{:else}
					<div class="flex min-h-0 flex-1 justify-center overflow-auto p-4">
						{#if frameSrc}
							<iframe
								bind:this={frame}
								class="min-h-140 w-full flex-1 rounded-md bg-surface outline -outline-offset-1 outline-border"
								style:max-width={frameWidth}
								src={frameSrc}
								sandbox="allow-scripts"
								referrerpolicy="no-referrer"
								title={m.studio_frame_title({ title: current.title })}
							></iframe>
						{/if}
					</div>
				{/if}

				{#if pinPopoverOpen && selection && mode !== 'feedback'}
					<aside
						data-part="pin-popover"
						data-testid="direct-pin-popover"
						aria-label={m.studio_feedback_label()}
						class="absolute right-5 bottom-16 z-20 flex w-80 max-w-full flex-col gap-2.5 rounded-md bg-surface p-3.5 shadow-float outline -outline-offset-1 outline-border-strong"
					>
						<div class="flex items-start justify-between gap-2">
							<div class="flex flex-col">
								<span class="font-mono text-label text-text-3">{KIND_LABELS[selection.kind]} · {selection.page}</span>
								<strong class="text-caption font-medium text-text">{selection.label}</strong>
							</div>
							<button
								type="button"
								class="cursor-pointer font-mono text-label text-text-2 hover:text-text"
								onclick={() => (pinPopoverOpen = false)}
							>
								{m.studio_approve_cancel()}
							</button>
						</div>
						<form class="flex flex-col gap-2" onsubmit={savePinFeedback}>
							<textarea
								rows="3"
								maxlength={FEEDBACK_CHARS}
								class="w-full rounded-sm bg-bg p-2 text-caption text-text outline -outline-offset-1 outline-border focus:outline-border-strong"
								bind:value={pinWords}
								placeholder={m.studio_feedback_placeholder()}
							></textarea>
							<label class="flex items-center gap-1.5 text-caption text-text-2">
								<input type="checkbox" bind:checked={pinInteraction} />
								<span>{m.studio_feedback_interaction()}</span>
							</label>
							{#if feedbackError}
								<p class="m-0 text-caption text-danger" role="alert">{feedbackError}</p>
							{/if}
							<div class="flex items-center justify-end gap-2">
								<button
									type="submit"
									class="cursor-pointer rounded-sm bg-accent px-3 py-1 text-caption font-medium text-accent-ink disabled:opacity-50"
									disabled={!pinDraft || sending}
								>
									{m.studio_pin_save()}
								</button>
							</div>
						</form>
					</aside>
				{/if}

				{#if waiting > 0}
					<div
						data-testid="revise-with-sora-bar"
						class="flex flex-wrap items-center justify-between gap-3 border-t border-border bg-surface px-4 py-2.5"
					>
						<div class="flex items-center gap-2">
							<span class="font-mono text-label text-accent">{m.studio_revise_bar_count({ count: waiting })}</span>
							{#if refineSent}
								<span class="text-caption text-text-2" role="status">{m.studio_refine_sent()}</span>
							{/if}
							{#if refineError}
								<span class="text-caption text-danger" role="alert">{refineError}</span>
							{/if}
						</div>
						<div class="flex items-center gap-2">
							{#if refineSent}
								<button
									type="button"
									class="cursor-pointer rounded-sm bg-surface-2 px-2.5 py-1 text-caption text-text outline -outline-offset-1 outline-border hover:outline-border-strong"
									onclick={checkForNewVersion}
								>
									{m.studio_refine_refresh()}
								</button>
							{/if}
							<button
								type="button"
								class="cursor-pointer rounded-sm bg-accent px-3 py-1 text-caption font-medium text-accent-ink disabled:opacity-50"
								disabled={refining}
								onclick={() => void askSora()}
							>
								{m.studio_revise_cta()}
							</button>
						</div>
					</div>
				{/if}
			</div>
		</div>
	{/if}
</div>
