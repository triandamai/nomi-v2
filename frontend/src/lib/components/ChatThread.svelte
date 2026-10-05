<script lang="ts">
	import { enhance } from '$app/forms';
	import { invalidateAll } from '$app/navigation';
	import { page } from '$app/state';
	import { onMount, type Snippet } from 'svelte';
	import CrewPanel from '$lib/components/CrewPanel.svelte';
	import MessageBubble from '$lib/components/MessageBubble.svelte';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import LoadingIndicator from '$lib/components/m3/LoadingIndicator.svelte';
	import SendButton from '$lib/components/m3/SendButton.svelte';
	import MicButton from '$lib/components/MicButton.svelte';
	import VoiceNoteRecorder from '$lib/components/VoiceNoteRecorder.svelte';
	import Menu from '$lib/components/m3/Menu.svelte';
	import MenuItem from '$lib/components/m3/MenuItem.svelte';
	import ThinkingMenu from '$lib/components/ThinkingMenu.svelte';
	import { deserialize } from '$app/forms';
	import {
		composeMessage,
		formatBytes,
		isAudioFile,
		isTextFile,
		voiceNoteName,
		MAX_ATTACHMENTS,
		MAX_FILE_BYTES,
		MAX_TOTAL_BYTES,
		type Attachment,
	} from '$lib/attachments';
	import { buildMessageFetchUrl } from '$lib/buildMessageFetchUrl';
	import { agentTypeFallbackLabel, delegationStatusLabel, toolActivityLabel } from '$lib/agentLabels';
	import { buildCrew } from '$lib/crew';
	import { groupReasoning } from '$lib/reasoning';
	import type { AgentStatus, RenderedMessage, ThinkingLevel } from '$lib/types';

	interface AgentActivityItem {
		id: string;
		target_agent_type: string;
		status: string;
		task: string;
		result: string | null;
		error: string | null;
	}

	let {
		sessionId,
		messages,
		agentActivity,
		agentStatus = null,
		sendError = null,
		title = 'Chat',
		context,
		extraControls,
		thinkingLevel = null,
	}: {
		sessionId: string;
		messages: RenderedMessage[];
		agentActivity: AgentActivityItem[];
		agentStatus?: AgentStatus | null;
		sendError?: string | null;
		/** Shown in the app bar. */
		title?: string;
		/** Optional lead-in for the app bar's meta line, e.g. a project name. */
		context?: string;
		/** App-bar actions (model / personality menus). */
		extraControls?: Snippet;
		/** The chat's thinking level; null hides the picker. The page must have a setThinking action. */
		thinkingLevel?: ThinkingLevel | null;
	} = $props();

	// Composer state: the typed draft, attached files, and the chat's thinking level.
	let draft = $state('');
	let attachments = $state<(Attachment & { size: number })[]>([]);
	let attachError = $state<string | null>(null);
	let fileInput: HTMLInputElement | undefined = $state();
	let attachMenuOpen = $state(false);
	let recordingVoice = $state(false);
	let speechSupported = $state(false);
	onMount(() => {
		const w = window as unknown as { SpeechRecognition?: unknown; webkitSpeechRecognition?: unknown };
		speechSupported = Boolean(w.SpeechRecognition ?? w.webkitSpeechRecognition);
	});

	function saveVoiceNote(transcript: string, seconds: number) {
		recordingVoice = false;
		if (attachments.length >= MAX_ATTACHMENTS) {
			attachError = `Up to ${MAX_ATTACHMENTS} files per message.`;
			return;
		}
		attachments = [...attachments, { name: voiceNoteName(seconds), text: transcript, kind: 'voice', size: new Blob([transcript]).size }];
		messageInput?.focus();
	}
	let dictationBase = '';
	let level = $state<ThinkingLevel>('medium');
	$effect(() => {
		if (thinkingLevel) level = thinkingLevel;
	});

	const canSend = $derived(draft.trim().length > 0 || attachments.length > 0);
	const composedText = $derived(composeMessage(draft, attachments));

	async function addFiles(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const files = [...(input.files ?? [])];
		input.value = '';
		attachError = null;
		const refused: string[] = [];
		const audio: string[] = [];
		for (const file of files) {
			if (attachments.length >= MAX_ATTACHMENTS) {
				attachError = `Up to ${MAX_ATTACHMENTS} files per message.`;
				break;
			}
			if (isAudioFile(file.name, file.type)) {
				audio.push(file.name);
				continue;
			}
			if (!isTextFile(file.name, file.type)) {
				refused.push(file.name);
				continue;
			}
			const total = attachments.reduce((sum, a) => sum + a.size, 0);
			if (file.size > MAX_FILE_BYTES || total + file.size > MAX_TOTAL_BYTES) {
				attachError = `${file.name} is too big (files up to ${formatBytes(MAX_FILE_BYTES)}, ${formatBytes(MAX_TOTAL_BYTES)} per message).`;
				continue;
			}
			attachments = [...attachments, { name: file.name, text: await file.text(), kind: 'file', size: file.size }];
		}
		if (audio.length > 0) {
			attachError = `${audio.join(', ')}: audio files can't be read yet. Record a voice note instead, from the attach menu.`;
		}
		if (refused.length > 0) {
			attachError = `${refused.join(', ')}: only text files (notes, CSV, JSON, code) can be attached for now, not images or PDFs.`;
		}
	}

	function removeAttachment(index: number) {
		attachments = attachments.filter((_, i) => i !== index);
		attachError = null;
	}

	function onTranscript(text: string, final: boolean) {
		draft = dictationBase ? `${dictationBase} ${text}` : text;
		if (final) dictationBase = draft;
	}

	async function setThinking(next: ThinkingLevel) {
		const previous = level;
		level = next;
		const body = new FormData();
		body.set('level', next);
		try {
			const response = await fetch('?/setThinking', { method: 'POST', body, headers: { 'x-sveltekit-action': 'true' } });
			if (deserialize(await response.text()).type !== 'success') level = previous;
		} catch {
			level = previous;
		}
	}

	// Turns streaming into this chat right now, by turn id: the user's own turn plus any
	// hand-off or scheduled run. Nomi is working while any is open; each closes on its own
	// TurnCompleted/TurnFailed, so one finishing never hides (or strands) another.
	let openTurns = $state<string[]>([]);
	const pendingReply = $derived(openTurns.length > 0);
	// Set once the supervisor has stopped this chat's crew: the stopped turn's last streamed
	// deltas must not bring the working indicator back. Cleared when that turn ends or the user
	// sends something new.
	let stopRequested = $state(false);
	let stopping = $state(false);
	let submitting = $state(false);
	let turnError = $state(false);
	let connectionLost = $state(false);
	let activitySheetOpen = $state(false);
	let messagesContainer: HTMLDivElement | undefined = $state();
	let messageInput: HTMLTextAreaElement | undefined = $state();

	let localMessages = $state(messages);

	let currentPhase = $state<{ phase: string; detail: string | null } | null>(
		agentStatus ? { phase: agentStatus.current_phase, detail: agentStatus.current_phase_detail } : null,
	);

	function phaseText(phase: string, detail: string | null): string {
		if (phase === 'thinking') return 'Nomi is thinking…';
		if (phase === 'writing_reply') return 'Nomi is finalizing a reply…';
		if (phase === 'calling_tool') return detail ? `Nomi is ${toolActivityLabel(detail)}…` : 'Nomi is using a tool…';
		return 'Nomi is working…';
	}

	// Resync whenever the page's own `messages` prop changes — navigating to a different
	// session, or a full invalidateAll() (still used for AgentDelegationUpdated and on
	// WS-reconnect-after-drop, see below).
	$effect(() => {
		localMessages = messages;
	});

	// Enter sends, Shift+Enter breaks the line — the textarea grows with its content.
	function onComposerKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' && !event.shiftKey && !event.isComposing) {
			event.preventDefault();
			const field = event.currentTarget as HTMLTextAreaElement;
			// While the crew works, only a stop command goes through (anything else would queue
			// behind the running turn).
			if (canSend && (!isWorking || STOP_COMMAND.test(draft))) field.form?.requestSubmit();
		}
	}

	const NIL_UUID = '00000000-0000-0000-0000-000000000000';
	const STOP_COMMAND = /^\W*(stop|cancel|abort|halt|berhenti|hentikan|batalkan)\b/i;

	/** The supervisor answered a stop command: no turn is coming, and any running one is ending. */
	function settleAfterStop() {
		stopRequested = true;
		submitting = false;
		openTurns = [];
		currentPhase = null;
	}

	/** A turn ended. Also ends "sending": a turn can finish without streaming anything (an approval, a stop). */
	function closeTurn(turnId: string) {
		openTurns = openTurns.filter((id) => id !== turnId);
		submitting = false;
		if (openTurns.length === 0) currentPhase = null;
	}

	async function fetchAndUpsertMessage(id: string) {
		try {
			// Plain string concatenation via buildMessageFetchUrl, not a bare relative reference —
			// this route never renders with a trailing slash, so `fetch('message/id')` would resolve
			// against the current URL by dropping the sessionId segment (WHATWG relative-URL
			// resolution rules), not appending to it. See buildMessageFetchUrl's own doc comment.
			const response = await fetch(buildMessageFetchUrl(page.url.pathname, id));
			if (!response.ok) return;
			const message: RenderedMessage = await response.json();
			const index = localMessages.findIndex((m) => m.id === message.id);
			if (index === -1) {
				localMessages = [...localMessages, message];
			} else {
				localMessages = [...localMessages.slice(0, index), message, ...localMessages.slice(index + 1)];
			}
		} catch {
			// Best-effort — a dropped connection triggers a full invalidateAll() on reconnect
			// (see the socket 'open' handler below), which reconciles anything missed here.
		}
	}

	const TERMINAL_CLOSE_CODES = new Set([4401, 4404]);
	const INITIAL_RETRY_DELAY_MS = 1000;
	const MAX_RETRY_DELAY_MS = 30000;

	// Consecutive messages from the same sender in the same minute chain into one visual group
	// (no repeated "You"/"Nomi" label, tighter spacing) — same idea as Slack/iMessage grouping.
	function sameMinute(a: string, b: string): boolean {
		const da = new Date(a);
		const db = new Date(b);
		return (
			da.getFullYear() === db.getFullYear() &&
			da.getMonth() === db.getMonth() &&
			da.getDate() === db.getDate() &&
			da.getHours() === db.getHours() &&
			da.getMinutes() === db.getMinutes()
		);
	}

	function isChained(list: RenderedMessage[], index: number): boolean {
		if (index === 0) return false;
		const previous = list[index - 1];
		const current = list[index];
		if (previous.sender !== current.sender) return false;
		// For assistant messages, only chain when the exact same agent produced both — e.g. a
		// money-agent reply immediately followed by a chitchat reply are both "assistant" but
		// must never chain into one unlabeled group.
		if (previous.agent_display_name !== current.agent_display_name) return false;
		return sameMinute(previous.created_at, current.created_at);
	}

	// The sender label shows on the first bubble of a chain (announcing who's talking), but the
	// timestamp shows on the last one instead — same convention as iMessage/Slack: what matters
	// for a timestamp is when the group of messages finished, not when it started.
	function isLastInChain(list: RenderedMessage[], index: number): boolean {
		return index === list.length - 1 || !isChained(list, index + 1);
	}


	// `submitting` covers the gap between hitting Send and the first streamed token arriving —
	// `pendingReply` alone only flips once a WS "Delta" event lands, which leaves a brief window
	// right after sending where neither the button nor the "Typing…" bubble showed any feedback.
	const isWorking = $derived(submitting || pendingReply);

	// Thinking steps fold into the reply they led to (see $lib/reasoning).
	const threadItems = $derived(groupReasoning(localMessages));
	const threadMessages = $derived(threadItems.map((item) => item.message));

	const crew = $derived(
		buildCrew({
			activity: agentActivity,
			messageAuthors: localMessages.filter((m) => m.sender === 'assistant').map((m) => m.agent_display_name),
			nomiWorking: isWorking,
			roster: page.data.crew ?? [],
			nomiStatus: currentPhase
				? phaseText(currentPhase.phase, currentPhase.detail).replace(/^Nomi is /, '')
				: isWorking
					? 'working…'
					: null,
		}),
	);
	const involvedCrew = $derived(crew.filter((m) => m.involved));
	const workingCrew = $derived(crew.filter((m) => m.working));
	// App-bar meta line: "TRAVEL · NOMI + MONEY + PLANNING"
	const metaLine = $derived(
		[context, involvedCrew.map((m) => m.name).join(' + ')].filter(Boolean).join(' · '),
	);

	$effect(() => {
		if (pendingReply || turnError) submitting = false;
	});

	// -1 (not 0) so the very first run — the initial mount — always counts as "grew" and scrolls
	// to the bottom of whatever history loaded, regardless of how many messages that is.
	let lastMessageCount = -1;

	// Keep the latest message (and the "Typing…" indicator) in view, but only when a message was
	// actually appended or a turn is in progress — not on every fetchAndUpsertMessage call. Under
	// Svelte 5, reassigning `localMessages` (even to replace one entry in place, e.g. a todo-list
	// step flipping or an approval being decided after the turn has already finished) re-triggers
	// any effect that reads `.length`, even though the length didn't change. Comparing against the
	// previous count so an in-place `MessageUpdated` patch — which this task's whole point is to
	// apply without visibly jumping the viewport — doesn't yank the scroll position, while a
	// genuinely new message (or live progress while `isWorking`) still does.
	$effect(() => {
		const currentCount = localMessages.length;
		const grew = currentCount > lastMessageCount;
		lastMessageCount = currentCount;
		if (grew || isWorking) {
			messagesContainer?.scrollTo({ top: messagesContainer.scrollHeight });
		}
	});

	onMount(() => {
		messageInput?.focus();

		let socket: WebSocket | undefined;
		let retryDelay = INITIAL_RETRY_DELAY_MS;
		let retryTimeout: ReturnType<typeof setTimeout> | undefined;
		let intentionallyClosed = false;
		let hasConnectedBefore = false;

		function connect() {
			socket = new WebSocket(`/chat/${sessionId}/ws`);

			socket.addEventListener('open', () => {
				retryDelay = INITIAL_RETRY_DELAY_MS;
				connectionLost = false;
				if (hasConnectedBefore) {
					// Reconnected after a drop; neither leg replays missed events, so re-fetch to
					// reconcile anything that happened while disconnected. A turn that ended in the
					// gap would otherwise stay "working"; one still running streams again.
					openTurns = [];
					invalidateAll();
				}
				hasConnectedBefore = true;
			});

			socket.addEventListener('message', (event) => {
				let envelope: { kind: string; turn_job_id?: string; message_id?: string; phase?: string; detail?: string | null };
				try {
					envelope = JSON.parse(event.data);
				} catch {
					return;
				}
				const turnId = envelope.turn_job_id ?? NIL_UUID;
				if (envelope.kind === 'Delta') {
					if (!stopRequested && !openTurns.includes(turnId)) openTurns = [...openTurns, turnId];
				} else if (envelope.kind === 'TurnCompleted') {
					stopRequested = false;
					closeTurn(turnId);
					turnError = false;
					if (envelope.message_id && envelope.message_id !== NIL_UUID) {
						fetchAndUpsertMessage(envelope.message_id);
					}
				} else if (envelope.kind === 'TurnFailed') {
					stopRequested = false;
					closeTurn(turnId);
					turnError = true;
				} else if (envelope.kind === 'AgentDelegationUpdated') {
					invalidateAll();
				} else if (envelope.kind === 'MessageCreated' || envelope.kind === 'MessageUpdated') {
					if (envelope.message_id) fetchAndUpsertMessage(envelope.message_id);
				} else if (envelope.kind === 'AgentPhaseChanged') {
					if (typeof envelope.phase === 'string' && !stopRequested) {
						currentPhase = envelope.phase === 'waiting' ? null : { phase: envelope.phase, detail: envelope.detail ?? null };
					}
				}
			});

			socket.addEventListener('close', (event) => {
				if (intentionallyClosed) return;
				if (TERMINAL_CLOSE_CODES.has(event.code)) {
					connectionLost = true;
					return;
				}
				const jitter = Math.random() * 250;
				retryTimeout = setTimeout(connect, retryDelay + jitter);
				retryDelay = Math.min(retryDelay * 2, MAX_RETRY_DELAY_MS);
			});
		}

		connect();

		return () => {
			intentionallyClosed = true;
			clearTimeout(retryTimeout);
			socket?.close();
		};
	});
</script>

<div class="chat">
	<header class="appbar">
		<div class="appbar__titles">
			<h1 class="appbar__title">{title}</h1>
			<span class="appbar__meta">{metaLine}</span>
		</div>
		<button
			type="button"
			class="appbar__crew"
			aria-label="Your crew: {workingCrew.length > 0 ? `${workingCrew.length} working` : 'all idle'}"
			onclick={() => (activitySheetOpen = true)}
		>
			<span class="appbar__stack">
				{#each involvedCrew.slice(0, 4) as member (member.key)}
					<span class="appbar__stack-item"><AgentShape agent={member.key} size={30} working={member.working} /></span>
				{/each}
			</span>
			{#if workingCrew.length > 0}
				<span class="appbar__crew-label">{workingCrew.length} working</span>
			{/if}
		</button>
		{#if extraControls}
			<div class="appbar__actions">
				{@render extraControls()}
			</div>
		{/if}
	</header>

	<div class="chat__body">
		<div class="chat__main">
			<div bind:this={messagesContainer} class="thread">
				<div class="thread__column">
					{#each threadItems as item, i (item.message.id)}
						<MessageBubble
							message={item.message}
							reasoning={item.reasoning}
							thinkingOnly={item.thinkingOnly}
							chained={isChained(threadMessages, i)}
							first={i === 0}
							showTimestamp={isLastInChain(threadMessages, i)}
						/>
					{/each}
					{#if isWorking}
						<div class="thread__status" role="status">
							<LoadingIndicator size={28} label="Nomi is working" />
							<span>{currentPhase ? phaseText(currentPhase.phase, currentPhase.detail) : 'Nomi is working…'}</span>
						</div>
					{/if}
					{#if turnError}
						<p class="thread__notice" role="alert">Something went wrong — try sending again.</p>
					{/if}
					{#if sendError}
						<p class="thread__notice" role="alert">{sendError}</p>
					{/if}
					{#if connectionLost}
						<p class="thread__notice" role="alert">Couldn't connect to this chat — try reloading the page.</p>
					{/if}
				</div>
			</div>

			<div class="dock">
				<form
					method="POST"
					action="?/sendMessage"
					class="composer"
					use:enhance={() => {
						submitting = true;
						stopRequested = false;
						return async ({ result, update }) => {
							await update({ reset: true });
							if (result.type === 'success') {
								draft = '';
								dictationBase = '';
								attachments = [];
								attachError = null;
								if (result.data?.stopped) settleAfterStop();
							}
							messageInput?.focus();
						};
					}}
				>
					{#if attachments.length > 0 || attachError || recordingVoice}
						<div class="composer__files">
							{#each attachments as file, i (file.name + i)}
								<span class="file-chip" class:file-chip--voice={file.kind === 'voice'} title={file.kind === 'voice' ? file.text : undefined}>
									{#if file.kind === 'voice'}
										<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect x="9" y="3" width="6" height="11" rx="3" /><path d="M5 11a7 7 0 0 0 14 0M12 18v3" /></svg>
									{:else}
										<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z" /><path d="M14 3v5h5" /></svg>
									{/if}
									<span class="file-chip__name">{file.name}</span>
									<span class="file-chip__size">{formatBytes(file.size)}</span>
									<button type="button" class="file-chip__remove" aria-label="Remove {file.name}" onclick={() => removeAttachment(i)}>
										<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" aria-hidden="true"><path d="M6 6l12 12M18 6 6 18" /></svg>
									</button>
								</span>
							{/each}
							{#if recordingVoice}
								<VoiceNoteRecorder onsave={saveVoiceNote} oncancel={() => (recordingVoice = false)} />
							{/if}
							{#if attachError}
								<p class="composer__error" role="alert">{attachError}</p>
							{/if}
						</div>
					{/if}
					<div class="composer__row">
						<Menu bind:open={attachMenuOpen}>
							{#snippet trigger({ toggle })}
								<button type="button" class="composer__attach" aria-label="Attach" title="Attach files or a voice note" onclick={toggle}>
									<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m21 11.5-8.6 8.6a5 5 0 0 1-7.1-7.1l8.6-8.6a3.3 3.3 0 0 1 4.7 4.7l-8.6 8.6a1.7 1.7 0 0 1-2.4-2.4l7.9-7.9" /></svg>
								</button>
							{/snippet}
							<MenuItem
								type="button"
								onclick={() => {
									attachMenuOpen = false;
									fileInput?.click();
								}}
							>
								<span class="attach-option">
									<span class="attach-option__label">Attach files</span>
									<span class="attach-option__hint">Notes, CSV, JSON or code</span>
								</span>
							</MenuItem>
							{#if speechSupported}
								<MenuItem
									type="button"
									disabled={recordingVoice}
									onclick={() => {
										attachMenuOpen = false;
										attachError = null;
										recordingVoice = true;
									}}
								>
									<span class="attach-option">
										<span class="attach-option__label">Record a voice note</span>
										<span class="attach-option__hint">Say it; Nomi works out what to do</span>
									</span>
								</MenuItem>
							{/if}
						</Menu>
						<input bind:this={fileInput} type="file" multiple hidden onchange={addFiles} />
						<label for="chat-message" class="sr-only">Message</label>
						<textarea
							id="chat-message"
							bind:this={messageInput}
							bind:value={draft}
							rows="1"
							placeholder="Message Nomi"
							class="composer__input"
							onkeydown={onComposerKeydown}
							onfocus={() => (dictationBase = draft)}
						></textarea>
						<input type="hidden" name="text" value={composedText} />
						<MicButton ontranscript={onTranscript} />
						{#if thinkingLevel}
							<ThinkingMenu {level} onchange={setThinking} />
						{/if}
						<SendButton
							working={isWorking || stopping}
							stopForm={stopping ? undefined : 'stop-crew'}
							disabled={!canSend}
						/>
					</div>
				</form>
				<!-- The in-flight send button submits this: the same "stop" a user could type. -->
				<form
					id="stop-crew"
					method="POST"
					action="?/sendMessage"
					hidden
					use:enhance={() => {
						stopping = true;
						return async ({ result, update }) => {
							await update({ reset: false });
							stopping = false;
							if (result.type === 'success' && result.data?.stopped) settleAfterStop();
						};
					}}
				>
					<input type="hidden" name="text" value="stop" />
				</form>
			</div>
		</div>

		<aside class="chat__crew" aria-label="Crew">
			<CrewPanel members={crew} />
		</aside>
	</div>
</div>

<BottomSheet bind:open={activitySheetOpen}>
	{#snippet children()}
		<CrewPanel members={crew} />
		<h2 class="md-title-large" style="color: var(--md-sys-color-on-surface); margin: 20px 0 12px;">Recent hand-offs</h2>
		{#if agentActivity.length === 0}
			<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">No background activity yet.</p>
		{:else}
			{#each agentActivity as item (item.id)}
				<div style="padding: 8px 0; border-bottom: 1px solid var(--md-sys-color-outline-variant);">
					<p class="md-body-large" style="color: var(--md-sys-color-on-surface)">
						{agentTypeFallbackLabel(item.target_agent_type)} — {delegationStatusLabel(item.status)}
					</p>
					<p class="md-body-small" style="color: var(--md-sys-color-on-surface-variant)">{item.task}</p>
					{#if item.result}
						<p class="md-body-small" style="color: var(--md-sys-color-on-surface-variant)">{item.result}</p>
					{/if}
					{#if item.error}
						<p class="md-body-small" style="color: var(--md-sys-color-error)">{item.error}</p>
					{/if}
				</div>
			{/each}
		{/if}
	{/snippet}
</BottomSheet>

<style>
	/* The chat lays itself out by its own width (container queries), not the viewport — the same
	   component sits full-width on /chat and in a 420px column on project pages. */
	.chat {
		container-type: inline-size;
		display: flex;
		flex-direction: column;
		height: 100%;
		background: var(--md-sys-color-surface);
	}

	.appbar {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 14px clamp(12px, 3vw, 32px) 10px;
	}
	.appbar__titles {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
	}
	.appbar__title {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.75rem;
		line-height: 1.15;
		font-weight: 700;
		letter-spacing: -0.025em;
		color: var(--md-sys-color-on-surface);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.appbar__meta {
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--md-sys-color-on-surface-variant);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.appbar__crew {
		display: flex;
		align-items: center;
		gap: 8px;
		height: 44px;
		padding: 0 12px 0 6px;
		border: none;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-sys-typescale-label-large-font);
		font-size: var(--md-sys-typescale-label-large-size);
		font-weight: 600;
		cursor: pointer;
		transition: border-radius var(--nomi-motion-spatial-fast);
	}
	.appbar__crew:active {
		border-radius: var(--md-sys-shape-corner-medium);
	}
	.appbar__stack {
		display: flex;
		padding-left: 4px;
	}
	.appbar__stack-item {
		display: flex;
		margin-left: -8px;
		border-radius: 50%;
	}
	.appbar__stack-item:first-child {
		margin-left: 0;
	}
	.appbar__crew-label {
		white-space: nowrap;
	}
	.appbar__actions {
		display: flex;
		align-items: center;
	}

	.chat__body {
		flex: 1;
		min-height: 0;
		display: flex;
		gap: 24px;
		padding-right: 0;
	}
	.chat__main {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
	}
	.chat__crew {
		display: none;
	}
	/* Narrow chats (phones, the project page's side column): tighter title, crew button shows
	   just the spinning stack. */
	@container (max-width: 600px) {
		.appbar {
			gap: 4px;
		}
		.appbar__title {
			font-size: 1.375rem;
		}
		.appbar__crew-label {
			display: none;
		}
		.appbar__crew {
			padding: 0 8px 0 6px;
		}
	}

	/* Wide chats: the crew lives in its own column and the app-bar stack button steps aside. */
	@container (min-width: 1100px) {
		.chat__body {
			padding-right: 24px;
		}
		.chat__crew {
			display: block;
			flex: none;
			width: 300px;
			overflow-y: auto;
			padding-bottom: 16px;
		}
		.appbar__crew {
			display: none;
		}
	}

	.thread {
		flex: 1;
		overflow-y: auto;
		padding: 12px clamp(16px, 3vw, 40px) 8px;
	}
	.thread__column {
		max-width: 780px;
		margin: 0 auto;
	}
	.thread__status {
		display: flex;
		align-items: center;
		gap: 12px;
		margin-top: 22px;
		padding-left: 4px;
		font-family: var(--md-sys-typescale-body-large-font);
		font-size: 0.9375rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.thread__notice {
		margin: 16px 0 0;
		padding: 12px 16px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
		font-family: var(--md-sys-typescale-body-medium-font);
		font-size: var(--md-sys-typescale-body-medium-size);
	}

	.dock {
		padding: 8px clamp(12px, 3vw, 40px) 16px;
	}
	.composer {
		display: flex;
		flex-direction: column;
		gap: 6px;
		max-width: 780px;
		margin: 0 auto;
		padding: 8px;
		border-radius: var(--md-sys-shape-corner-extra-large-increased);
		background: var(--md-sys-color-surface-container-lowest);
		box-shadow:
			0 1px 0 var(--md-sys-color-outline-variant),
			0 18px 40px -28px color-mix(in srgb, var(--md-sys-color-on-surface) 45%, transparent);
	}
	.composer__input {
		flex: 1;
		min-width: 0;
		align-self: center;
		max-height: 200px;
		field-sizing: content;
		padding: 12px 4px;
		border: none;
		resize: none;
		outline: none;
		background: transparent;
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-sys-typescale-body-large-font);
		font-size: var(--md-sys-typescale-body-large-size);
		line-height: 1.5;
	}
	.composer__input::placeholder {
		color: var(--md-sys-color-on-surface-variant);
	}
	.composer__row {
		display: flex;
		align-items: flex-end;
		gap: 6px;
	}
	.composer__row > :global(*) {
		align-self: center;
	}
	.composer__attach {
		flex: none;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 44px;
		height: 44px;
		border: none;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface);
		cursor: pointer;
		transition: border-radius var(--nomi-motion-spatial-fast);
	}
	.composer__attach:hover {
		border-radius: var(--md-sys-shape-corner-medium);
	}
	.composer__attach:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.composer__files {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		padding: 4px 4px 0;
	}
	.composer__error {
		flex-basis: 100%;
		margin: 0;
		font-size: 0.8125rem;
		color: var(--md-sys-color-error);
	}
	.file-chip {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		max-width: 100%;
		height: 36px;
		padding: 0 4px 0 10px;
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface-container);
		color: var(--md-sys-color-on-surface);
		font-size: 0.8125rem;
	}
	.file-chip--voice {
		background: var(--md-sys-color-tertiary-container);
		color: var(--md-sys-color-on-tertiary-container);
	}
	.attach-option {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		text-align: left;
	}
	.attach-option__label {
		font-weight: 650;
	}
	.attach-option__hint {
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.file-chip__name {
		font-weight: 600;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		max-width: 220px;
	}
	.file-chip__size {
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.file-chip__remove {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border: none;
		border-radius: var(--md-sys-shape-corner-full);
		background: transparent;
		color: inherit;
		cursor: pointer;
	}
	.file-chip__remove:hover {
		background: var(--md-sys-color-surface-container-highest);
	}
</style>
