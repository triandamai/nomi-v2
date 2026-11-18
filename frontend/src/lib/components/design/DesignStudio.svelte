<script lang="ts">
	import { ArrowLeft, BadgeCheck, ExternalLink, Monitor, MousePointerClick, Smartphone, Tablet } from '@lucide/svelte';
	import { invalidateAll } from '$app/navigation';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { formatDate } from '$lib/dates';
	import Icon from '$lib/components/ds/foundations/Icon.svelte';
	import {
		DEVICE_WIDTHS,
		annotateMessage,
		annotationsOn,
		compareWith,
		elementsOnPage,
		feedbackDraft,
		highlightMessage,
		modeMessage,
		pickVersion,
		previewUrl,
		readPreviewMessage,
		refineMessage,
		versionsNewestFirst,
		type Device,
		type FeedbackScope,
		type Selection,
		type StudioApproval,
		type StudioPublishOutcome,
		type StudioFeedback,
		type StudioElement,
		type StudioElementKind,
		type StudioPreview,
	} from '$lib/designStudio';

	// The Design Studio: a chat's design previews, one version at a time, at phone, tablet or
	// desktop width. The preview runs in a frame with an origin of its own (no sign-in, nothing
	// of Nomi's within reach); it only tells the Studio which element was pointed at. Its own page
	// shows it full screen; a project page shows it `embedded` in its Design tab, beside the chat.
	let {
		sessionId,
		previews,
		feedback: given,
		approval: approvedBefore,
		wanted = null,
		embedded = false,
		onrefresh,
	}: {
		sessionId: string;
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
	const data = $derived({ sessionId, previews, wanted });

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

	// Looking at a version, or saying what should change about it. Giving feedback always picks.
	type Mode = 'preview' | 'feedback' | 'compare';
	const MODES: { mode: Mode; label: string }[] = [
		{ mode: 'preview', label: m.studio_mode_preview() },
		{ mode: 'feedback', label: m.studio_mode_feedback() },
		{ mode: 'compare', label: m.studio_mode_compare() },
	];
	let mode = $state<Mode>('preview');
	const picking = $derived(selecting || mode === 'feedback');

	// Everything said about the chat's previews: what the page loaded with, and what was added here.
	let added = $state<StudioFeedback[]>([]);
	// Feedback added here that a reload has since brought in is shown once.
	const feedback = $derived([...given, ...added.filter((a) => !given.some((g) => g.id === a.id))]);
	const history = $derived([...feedback].sort((a, b) => b.number - a.number));
	const marks = $derived(current && mode === 'feedback' ? annotationsOn(feedback, current, shownPage) : []);

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

	$effect(() => {
		if (frame) tell(modeMessage(picking));
	});
	$effect(() => {
		if (frame) tell(annotateMessage(marks));
	});

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
			tell(modeMessage(picking));
			tell(annotateMessage(marks));
			if (selection && selection.page === said.page) tell(highlightMessage(selection.id));
		} else {
			selection = said.selection;
			if (mode === 'feedback') scope = 'element';
		}
	}

	function openVersion(version: number) {
		pickedVersion = version;
		confirming = false;
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
	}

	function pick(element: StudioElement) {
		selection = { id: element.id, page: shownPage, kind: element.kind, label: element.label, tag: '', text: '', point: null };
		if (mode === 'feedback') scope = 'element';
		tell(highlightMessage(element.id));
	}

	function openMode(next: Mode) {
		mode = next;
		if (next === 'feedback' && selection) scope = 'element';
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

	async function askSora(event: SubmitEvent) {
		event.preventDefault();
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
		tell(highlightMessage(element.id));
	}

	function clearSelection() {
		selection = null;
		tell(highlightMessage(null));
	}
</script>

<svelte:window onmessage={heard} />

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
				<button type="button" class="studio__tool studio__tool--wide" aria-pressed={picking} disabled={mode === 'feedback'} onclick={toggleSelecting}>
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

			<div class="studio__stage" class:studio__stage--device={device !== 'desktop'} class:studio__stage--compare={mode === 'compare'}>
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
