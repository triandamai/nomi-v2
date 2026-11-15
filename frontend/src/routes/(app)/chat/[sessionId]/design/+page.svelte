<script lang="ts">
	import { ArrowLeft, ExternalLink, Monitor, MousePointerClick, Smartphone, Tablet } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { formatDate } from '$lib/dates';
	import Icon from '$lib/components/ds/foundations/Icon.svelte';
	import {
		DEVICE_WIDTHS,
		elementsOnPage,
		highlightMessage,
		modeMessage,
		pickVersion,
		previewUrl,
		readPreviewMessage,
		versionsNewestFirst,
		type Device,
		type Selection,
		type StudioElement,
		type StudioElementKind,
	} from '$lib/designStudio';
	import type { PageData } from './$types';

	// The Design Studio: a chat's design previews, one version at a time, at phone, tablet or
	// desktop width. The preview runs in a frame with an origin of its own (no sign-in, nothing
	// of Nomi's within reach); it only tells the Studio which element was pointed at.
	let { data }: { data: PageData } = $props();

	const ENTRY_PAGE = 'index.html';
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

	// The page the frame was sent to, and the page it says it is on (links inside it move it).
	let openedPage = $state(ENTRY_PAGE);
	let shownPage = $state(ENTRY_PAGE);
	const frameSrc = $derived(current ? previewUrl(current, openedPage) : null);
	const pageElements = $derived(current ? elementsOnPage(current, shownPage) : []);

	let device = $state<Device>('desktop');
	const frameWidth = $derived(DEVICE_WIDTHS[device] === null ? '100%' : `${DEVICE_WIDTHS[device]}px`);

	let selecting = $state(false);
	let selection = $state<Selection | null>(null);
	let frame = $state<HTMLIFrameElement | null>(null);

	// The frame's origin is its own and unnameable, so messages are addressed to its window.
	function tell(message: object) {
		frame?.contentWindow?.postMessage(message, '*');
	}

	function heard(event: MessageEvent) {
		if (!current || !frame || event.source !== frame.contentWindow) return;
		const said = readPreviewMessage(event.data, current);
		if (!said) return;
		if (said.type === 'ready') {
			shownPage = said.page;
			tell(modeMessage(selecting));
			if (selection && selection.page === said.page) tell(highlightMessage(selection.id));
		} else {
			selection = said.selection;
		}
	}

	function openVersion(version: number) {
		pickedVersion = version;
		openedPage = ENTRY_PAGE;
		shownPage = ENTRY_PAGE;
		selection = null;
	}

	function openPage(path: string) {
		openedPage = path;
		shownPage = path;
	}

	function toggleSelecting() {
		selecting = !selecting;
		tell(modeMessage(selecting));
	}

	function pick(element: StudioElement) {
		selection = { id: element.id, page: shownPage, kind: element.kind, label: element.label, tag: '', text: '' };
		tell(highlightMessage(element.id));
	}

	function clearSelection() {
		selection = null;
		tell(highlightMessage(null));
	}
</script>

<svelte:window onmessage={heard} />

<svelte:head><title>{m.studio_title()}</title></svelte:head>

<div class="studio">
	<header class="studio__bar">
		<a class="studio__back" href={`/chat/${data.sessionId}`} aria-label={m.studio_back()} title={m.studio_back()}><Icon icon={ArrowLeft} size={20} /></a>
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
				<button type="button" class="studio__tool studio__tool--wide" aria-pressed={selecting} onclick={toggleSelecting}>
					<Icon icon={MousePointerClick} size={18} /><span>{m.studio_select()}</span>
				</button>
				{#if current.published_url}
					<a class="studio__tool studio__tool--wide" href={current.published_url} target="_blank" rel="noopener noreferrer">
						<Icon icon={ExternalLink} size={18} /><span>{m.studio_open_published()}</span>
					</a>
				{/if}
			</div>
		{/if}
	</header>

	{#if !current || !frameSrc}
		<div class="studio__empty">
			<h2 class="md-title-large">{m.studio_empty_title()}</h2>
			<p class="md-body-large">{m.studio_empty_body()}</p>
		</div>
	{:else}
		<div class="studio__body">
			<nav class="studio__rail" aria-label={m.studio_versions()}>
				<h2 class="md-label-large studio__label">{m.studio_versions()}</h2>
				<ul class="studio__list">
					{#each versions as version (version.version)}
						<li>
							<button type="button" class="studio__row" aria-current={version.version === current.version ? 'true' : undefined} onclick={() => openVersion(version.version)}>
								<span class="md-label-large">{m.studio_version({ version: version.version })}{#if version.version === newest} · {m.studio_latest()}{/if}</span>
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

			<div class="studio__stage" class:studio__stage--device={device !== 'desktop'}>
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
					<p class="md-body-medium studio__muted">{selecting ? m.studio_select_hint() : m.studio_selected_none()}</p>
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
		background: var(--md-sys-color-surface);
		color: var(--md-sys-color-on-surface);
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
		display: flex;
		justify-content: center;
		min-width: 0;
		min-height: 0;
		background: var(--md-sys-color-surface-container);
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
	.studio__clear {
		margin-top: 4px;
		padding: 6px 0;
		border: 0;
		background: none;
		color: var(--md-sys-color-primary);
		cursor: pointer;
	}
	@media (max-width: 900px) {
		.studio {
			height: auto;
			min-height: 100dvh;
		}
		.studio__body {
			grid-template-columns: minmax(0, 1fr);
		}
		.studio__rail,
		.studio__panel {
			border: 0;
			border-top: 1px solid var(--md-sys-color-outline-variant);
		}
		.studio__stage {
			order: -1;
			height: 70dvh;
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
