<script lang="ts">
	import { onMount, tick, untrack, type Snippet } from 'svelte';
	import { ArrowUpRight, Check, Copy, Cpu, Download, FilePlus2, FolderCheck, FolderInput, Hourglass, ImageOff, Info, KeyRound, List, LogIn, Maximize2, Minimize2, Pencil, Pin, PinOff, RefreshCw, RotateCcw, RotateCw, Share, Sparkles, TextSelect, ThumbsDown, ThumbsUp, Trash2, Undo2, Unplug, X } from '@lucide/svelte';
	import { goto, invalidate } from '$app/navigation';
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import type { AgentStatus, MessageItem, MessageVersion, RenderedMessage, RerunResult, ThinkingLevel } from '$lib/adapters/api';
	import { chatCrewRows, crewIdOf, failureView, mentionOptions, modelMenu, type FailureAction } from '$lib/adapters/chat';
	import { clock as appClock } from '$lib/clock';
	import { callJson } from '$lib/chat/client';
	import { clock, elapsedLabel } from '$lib/chat/dates';
	import { connectChat, type ChatEvent } from '$lib/chat/events';
	import { chatFileName, chatMarkdown } from '$lib/chat/exportChat';
	import { chatHeader, type ChatAction } from '$lib/chat/header.svelte';
	import { later } from '$lib/chat/later';
	import { mergeReloaded, prependOlder, upsert } from '$lib/chat/mergeMessages';
	import * as outbox from '$lib/chat/outbox';
	import { afterRerun } from '$lib/chat/rerun';
	import { threadEntries, turnDuration } from '$lib/chat/thread';
	import { applyChatEvent, idleTurn, isWorking, retryInfo, turnSent, turnStopped, turnWithAgentActivity, type LiveTurn } from '$lib/chat/turn';
	import { toasts } from '$lib/ds/overlays/toasts.svelte';
	import MessageUser from '$lib/ds/conversation/MessageUser.svelte';
	import TeammateMessage from '$lib/ds/build/TeammateMessage.svelte';
	import { initials, peerFor } from '$lib/projects/peers';
	import LiveStep from '$lib/ds/conversation/LiveStep.svelte';
	import Reasoning from '$lib/ds/chat/Reasoning.svelte';
	import ReplyText from '$lib/ds/chat/ReplyText.svelte';
	import FollowUps from '$lib/ds/chat/FollowUps.svelte';
	import MessageEditor from '$lib/ds/chat/MessageEditor.svelte';
	import DateDivider from '$lib/ds/chat/DateDivider.svelte';
	import NewDivider from '$lib/ds/chat/NewDivider.svelte';
	import JumpToLatest from '$lib/ds/chat/JumpToLatest.svelte';
	import LoadingOlder from '$lib/ds/chat/LoadingOlder.svelte';
	import Banner from '$lib/ds/chat/Banner.svelte';
	import MoveDialog from '$lib/ds/chat/MoveDialog.svelte';
	import PhoneMoveSheet from '$lib/ds/thread/PhoneMoveSheet.svelte';
	import ShareDialog from '$lib/ds/chat/ShareDialog.svelte';
	import Dialog from '$lib/ds/overlays/Dialog.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import AgentReply from '$lib/ds/thread/AgentReply.svelte';
	import Prose from '$lib/ds/thread/Prose.svelte';
	import ReplyBar, { type Dislike, type RegenerateStyle } from '$lib/ds/thread/ReplyBar.svelte';
	import FailedReply from '$lib/ds/thread/FailedReply.svelte';
	import TurnNote from '$lib/ds/thread/TurnNote.svelte';
	import ChatComposer from '$lib/ds/thread/ChatComposer.svelte';
	import PhoneComposer from '$lib/ds/thread/PhoneComposer.svelte';
	import PhoneMessageEditor from '$lib/ds/thread/PhoneMessageEditor.svelte';
	import PhoneDislikeSheet from '$lib/ds/thread/PhoneDislikeSheet.svelte';
	import PhoneShareSheet from '$lib/ds/thread/PhoneShareSheet.svelte';
	import SheetTiles from '$lib/ds/overlays/SheetTiles.svelte';
	import ActionSheet from '$lib/ds/overlays/ActionSheet.svelte';
	import { longPress } from '$lib/longPress';
	import NavBar from '$lib/ds/thread/PhoneChatNavBar.svelte';
	import Mascot from '$lib/ds/agents/Mascot.svelte';
	import PhoneUserMessage from '$lib/ds/thread/PhoneMessageBubble.svelte';
	import ActionBar from '$lib/ds/mobile/ActionBar.svelte';
	import SheetRow from '$lib/ds/mobile/SheetRow.svelte';
	import BottomSheet from '$lib/ds/overlays/BottomSheet.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import { phoneView } from '$lib/phone.svelte';
	import { replyBlocks } from '$lib/adapters/blocks';
	import ReplyBlocks from '$lib/chat/ReplyBlocks.svelte';
	import CodeQuote from '$lib/ds/build/CodeQuote.svelte';
	import PhoneErrorCard from '$lib/ds/thread/PhoneErrorCard.svelte';
	import PhoneCutNotice from '$lib/ds/thread/PhoneCutNotice.svelte';
	import PhonePendingMessage from '$lib/ds/thread/PhonePendingMessage.svelte';
	import PhoneCrewWorking from '$lib/ds/thread/PhoneCrewWorking.svelte';
	import PhoneButton from '$lib/ds/mobile/PhoneButton.svelte';
	import { CREW } from '$lib/ds/agents/crew';
	import { crewOf } from '$lib/adapters/shell';
	import { countdown, phoneFailureView, resetsOn, type PhoneFailureAction } from '$lib/adapters/phoneChat';
	import PhoneChatFlows from './PhoneChatFlows.svelte';
	import { PhoneFlows } from './phoneFlows.svelte';
	import type { ChatPageData } from '$lib/server/chatPage';
	import type { DeletePreview } from '$lib/server/chat';
	import DeletedReply from '$lib/ds/chat/DeletedReply.svelte';
	import { blockedReason, composeMessage, extensionOf, formatBytes, hiddenLine, kindLabel, progressParts, readyCount, seesImages, splitAttachments, visionSplit, type SentFile } from '$lib/chat/attachments';
	import { ComposerUploads, type Chip } from '$lib/chat/uploads.svelte';
	import { clearDraft, loadDraft, saveDraft } from '$lib/chat/draft';
	import ComposerFile from '$lib/ds/thread/ComposerFile.svelte';
	import ModelFix from '$lib/ds/thread/ModelFix.svelte';
	import VisionModelMenu, { type VisionGroup } from '$lib/ds/thread/VisionModelMenu.svelte';
	import SentFiles, { type SentFileItem } from '$lib/ds/thread/SentFiles.svelte';
	import FileNote from '$lib/ds/thread/FileNote.svelte';
	import FileViewer from '$lib/ds/thread/FileViewer.svelte';
	import PhotoViewer from '$lib/ds/thread/PhotoViewer.svelte';
	import FileSheetHeader from '$lib/ds/thread/FileSheetHeader.svelte';
	import type { Shell } from '$lib/server/shell';

	// A chat, as its page and a project's workspace show it.
	// Flow / Chat — Ask → think → answer (ENb07), Message controls (Cf6GO), Like & dislike (HKYo1),
	// Manage chat & long threads (J3ks0) and Errors & recovery (TpuRc): a chat, live. The thread
	// scrolls in a 720 column padded [40,48,0,48]; the composer stays at the bottom, padded
	// [0,48,24,48]. Everything comes from the backend: the messages (rendered on the server), the
	// chat's live feed (/chat/:id/ws, reloaded on every (re)connect since nothing missed is
	// replayed), and what each call answers. Rich reply blocks, attachments and voice come later
	// and slot in where a reply's prose goes and in the composer's attach and voice.
	type Props = {
		data: ChatPageData & Pick<Shell, 'crew' | 'usage'>;
		/** In a project's workspace: the 520 chat pane, 20 in, with the project's own placeholder. */
		pane?: boolean;
		/** Lines picked in the code view, quoted above the composer and sent ahead of the message. */
		quote?: { file: string; range: string; snippet: string; text: string } | null;
		onunquote?: () => void;
		/**
		 * The chat moved on: a message came in or was sent, a turn started, ended or was moved up. The
		 * workspace reads its line of asks again (Invite & build together TnwQT, step 9).
		 */
		onactivity?: () => void;
		/** What sits at the end of the thread, after the replies: the workspace's asks in line and requests to publish. */
		after?: Snippet;
	};
	let { data, pane = false, quote = null, onunquote, onactivity, after }: Props = $props();

	// On a phone (Flow / Mobile Chat — Start & answer, steps 2–7) the chat is the pen's phone screen:
	// the nav bar (back to Chats, the title, who answers, the chat's sheet), the thread padded
	// [16,16,20,16] with messages 24 apart, your messages in the 18-round bubble, each reply under the
	// Mobile byline with its prose at 16 and the round action bar (More: regenerate), and the phone's
	// composer, whose model chip and "@ Crew" open sheets. In a project's workspace on a phone (Mobile
	// Build) the chat is the Chat tab: the same, under the workspace's own nav, so without this one's.
	const phoneGuess = () => Boolean(page.data.phone);
	const phoneWidth = phoneView(phoneGuess());
	const phone = $derived(phoneWidth.current);
	type Sheet =
		| { kind: 'chat' }
		| { kind: 'rename' }
		| { kind: 'reply'; message: RenderedMessage; latest: boolean; asked?: RenderedMessage }
		| { kind: 'regen'; message: RenderedMessage }
		| { kind: 'dislike'; message: RenderedMessage }
		| { kind: 'mine'; message: RenderedMessage; latest: boolean }
		| { kind: 'model'; retry?: { message: RenderedMessage; reason: string } }
		| { kind: 'crew' };
	let sheet = $state<Sheet | null>(null);
	// The phone's chat flows (UX contract phone-chat-flows): questions, the plan's bar, the crew at
	// work, kept per chat.
	const flows = new PhoneFlows();
	/** A provider's retry wait seen in this turn (seconds), for the note once it went through. */
	let retryWait: number | null = null;
	let retried = $state<{ after: string; seconds: number } | null>(null);
	// What the phone's action bar shows at once, before the reload brings the saved feedback (the
	// desktop's reply bar keeps its own): put back if saving fails.
	let feedbackShown = $state<Record<string, 'liked' | 'disliked' | null>>({});
	async function copyText(text: string) {
		const ok = await navigator.clipboard
			?.writeText(text)
			.then(() => true)
			.catch(() => false);
		toasts.show(ok ? { kind: 'success', title: m.copy_done() } : { kind: 'error', title: m.action_failed() });
	}
	async function ratePhone(message: RenderedMessage, next: 'liked' | 'disliked' | null) {
		const before = feedbackShown[message.id];
		feedbackShown = { ...feedbackShown, [message.id]: next };
		if (!(await rate(message, next === 'liked' ? 'up' : next === 'disliked' ? 'down' : null))) feedbackShown = { ...feedbackShown, [message.id]: before };
	}
	let newName = $state('');

	const header = chatHeader();
	const time = appClock();
	const language = getLocale();
	const NIL = '00000000-0000-0000-0000-000000000000';

	type Pending = { key: number; text: string; status: 'sending' | 'queued' | 'failed'; reason?: string };

	function crewNameOf(agentType: string): string {
		const crewId = crewOf(agentType);
		return CREW.find((member) => member.id === crewId)?.name ?? (agentType ? agentType.charAt(0).toUpperCase() + agentType.slice(1) : 'Nomi');
	}

	function initialTurn(status: AgentStatus | null | undefined): LiveTurn {
		if (!status) return idleTurn();
		return turnWithAgentActivity(
			idleTurn(),
			{
				agent_session_id: status.agent_session_id,
				agent_name: crewNameOf(status.agent_type),
				phase: status.current_phase,
				detail: status.current_phase_detail,
			},
			Date.now(),
		);
	}

	const chatId = $derived(data.chat.id);
	const api = $derived(`/chat/${data.chat.id}/api`);
	const outboxKey = $derived(`/chat/${data.chat.id}`);

	let messages = $state<RenderedMessage[]>(untrack(() => data.messages));
	let hasOlder = $state(untrack(() => data.hasOlder));
	let turn = $state<LiveTurn>(untrack(() => initialTurn(data.agentStatus)));
	let tickNow = $state(Date.now());
	let pending = $state<Pending[]>([]);
	let nextKey = 1;
	let draft = $state('');
	let field: HTMLTextAreaElement | undefined = $state();
	let editing = $state<string | null>(null);
	let versions = $state<Record<string, MessageVersion[]>>({});
	let viewing = $state<Record<string, number>>({});
	/** Notes kept on messages for this visit: sent after signing in again. */
	let lateNotes = $state<Record<string, string>>({});
	/** Replies whose thinking was timed live, in seconds. */
	let thoughtFor = $state<Record<string, number>>({});
	/** A failure retried here, folded above the reply that came after it. */
	let recovered = $state<{ after: string; reason: string; credits?: boolean } | null>(null);
	/** The reply continued after the stream was cut off, and when it was. */
	let continued = $state<{ after: string; elapsed: string } | null>(null);
	/** The stop exchange sent from here: the stopped reply says it, so its messages wait for a reload. */
	const hidden = new Set<string>();
	let online = $state(true);
	let backOnline = $state(false);
	let modelKey = $state<string | null>(untrack(() => data.model?.key ?? data.models.current?.key ?? null));
	let thinking = $state<ThinkingLevel | null>(untrack(() => data.thinking));
	let moveOpen = $state(false);
	let connectedNote = $state<string | null>(null);
	let deleteOpen = $state(false);

	// Long threads: where the person is, what came in while they read further up, earlier pages.
	let scroller: HTMLElement | undefined = $state();
	let atLatest = $state(true);
	let unseen = $state(0);
	let newFromId = $state<string | null>(null);
	let loadingOlder = $state(false);
	let olderFailed = $state(false);

	const working = $derived(isWorking(turn));
	const entries = $derived(threadEntries(messages, { locale: language, now: time.now, timeZone: time.timeZone, newFromId }));
	const crew = $derived(mentionOptions(data.crew.rows));
	const models = $derived(modelMenu(data.models, modelKey));
	const modelLabel = $derived(models.flatMap((group) => group.models).find((option) => option.chosen)?.label ?? data.model?.label ?? data.models.current?.label);
	const canUseCredits = $derived(data.models.groups.some((group) => group.source === 'nomi' && group.models.some((option) => option.enabled)));
	const LEVELS: Record<ThinkingLevel, () => string> = { low: m.thinking_low, medium: m.thinking_medium, high: m.thinking_high, max: m.thinking_max };
	const thinkingChoice = $derived(
		thinking ? { label: LEVELS[thinking](), value: thinking, levels: (Object.keys(LEVELS) as ThinkingLevel[]).map((value) => ({ value, label: LEVELS[value]() })) } : undefined,
	);
	const quota = $derived(data.usage && data.usage.card.used >= 0.8 ? Math.round(data.usage.card.used * 100) : null);
	/** The failure notice the cut-off turn posted: the cut-off text is drawn in its place. */
	const cutOff = $derived.by(() => {
		if (turn.end?.status !== 'interrupted' || turn.startedAt === null) return null;
		const since = new Date(turn.startedAt - 1000).toISOString();
		return entries.findLast((entry) => entry.type === 'reply' && entry.failure && entry.message.created_at >= since)?.type === 'reply'
			? (entries.findLast((entry) => entry.type === 'reply' && entry.failure)?.type === 'reply' ? (entries.findLast((entry) => entry.type === 'reply' && entry.failure) as { message: RenderedMessage }).message.id : null)
			: null;
	});

	// The shell's top bar and crew panel follow this chat. Attaching reads what the header holds,
	// which detaching changes: untracked, or each would set off the other.
	$effect(() => {
		const id = chatId;
		untrack(() => header.attach(id, { pick: onMenu, rename, share }));
		return () => header.detach(id);
	});
	$effect(() => {
		const replied = [...new Set(messages.filter((message) => message.sender === 'assistant' && message.agent_display_name).map((message) => message.agent_display_name!))];
		header.crew = { replied, working: working ? (turn.agent ?? 'Nomi') : null };
	});

	// Another chat starts clean; the same chat reloaded keeps what came in live meanwhile.
	let shownId = untrack(() => data.chat.id);
	$effect.pre(() => {
		const incoming = data.messages;
		const id = data.chat.id;
		const older = data.hasOlder;
		untrack(() => {
			if (id !== shownId) {
				shownId = id;
				messages = incoming;
				hasOlder = older;
				turn = initialTurn(data.agentStatus);
				void readCrewAtWork();
				pending = [];
				editing = null;
				versions = {};
				viewing = {};
				recovered = null;
				continued = null;
				newFromId = null;
				unseen = 0;
				atLatest = true;
				modelKey = data.model?.key ?? data.models.current?.key ?? null;
				thinking = data.thinking;
				hidden.clear();
				uploads.clear();
				removedLine = null;
				viewer = null;
				photo = null;
				restoreDraft(id);
				flows.reset();
				retried = null;
				retryWait = null;
				return;
			}
			messages = mergeReloaded(incoming, messages, hasOlder);
		});
	});

	// The live rail's clock, while the crew is at it.
	$effect(() => {
		if (!working) return;
		tickNow = Date.now();
		const timer = setInterval(() => (tickNow = Date.now()), 1000);
		return () => clearInterval(timer);
	});
	const elapsed = $derived(turn.startedAt === null ? '' : elapsedLabel(tickNow - turn.startedAt));

	async function fetchMessage(id: string) {
		if (hidden.has(id)) return;
		const result = await callJson<RenderedMessage>(`${api}/messages/${id}`);
		if (!result.ok || hidden.has(id)) return;
		messages = upsert(messages, result.data);
	}

	const ACTIVITY = new Set<ChatEvent['kind']>(['QueueChanged', 'MessageCreated', 'TurnCompleted', 'TurnFailed', 'ChatSettled']);

	function onEvent(event: ChatEvent) {
		const before = turn;
		turn = applyChatEvent(turn, event, Date.now());
		if (ACTIVITY.has(event.kind)) onactivity?.();
		switch (event.kind) {
			case 'AgentSessionStarted':
				flows.sessionStarted(event.agent_session_id, event.agent_display_name);
				break;
			case 'AgentSessionEnded':
				flows.sessionEnded(event.agent_session_id);
				break;
			case 'AgentPhaseChanged':
				if (event.phase === 'retrying') retryWait ??= retryInfo(event.detail, Date.now()).seconds;
				break;
			case 'TurnFailed':
				retryWait = null;
				toasts.show({
					kind: 'error',
					title: event.error || m.action_failed(),
				});
				break;
			case 'TurnCompleted':
				if (retryWait !== null) retried = { after: new Date(before.startedAt ?? Date.now()).toISOString(), seconds: retryWait };
				retryWait = null;
				if (event.message_id !== NIL) {
					if (before.thoughtFor !== null) thoughtFor = { ...thoughtFor, [event.message_id]: Math.max(1, Math.round(before.thoughtFor / 1000)) };
					void fetchMessage(event.message_id);
				}
				break;
			case 'MessageCreated':
			case 'MessageUpdated':
				void fetchMessage(event.message_id);
				break;
			case 'MessagesDeleted': {
			const gone = new Set(event.message_ids);
			// A reply deleted here keeps its placeholder in the thread (Share & delete, step 5).
			messages = messages.filter((message) => !gone.has(message.id) || message.id in deletedReplies);
				break;
			}
			case 'SessionRenamed':
				header.title = event.title;
				void invalidate('app:shell');
				break;
			case 'SessionDeleted':
				if (!later.pending(chatId)) void goto('/chats', { replaceState: true });
				break;
		}
	}

	$effect(() => {
		const id = chatId;
		return connectChat({
			sessionId: id,
			handlers: {
				opened(resumed) {
					// Nothing missed is replayed: what happened meanwhile comes with a reload.
					if (resumed && !turn.end) turn = idleTurn();
				void readCrewAtWork();
					void invalidate('chat:page');
					onactivity?.();
				},
				event: onEvent,
				lost(code) {
					if (code === 4401) header.signedOutDraft = '';
					else void invalidate('chat:page');
				},
			},
		});
	});

	// Sending. Offline, it waits in the outbox; signed out, it waits for the sign-in; refused, it
	// stays where it was written with Retry and Edit.
	async function send(text: string, options: { late?: boolean; note?: string } = {}) {
		const local: Pending = { key: nextKey++, text, status: 'sending' };
		pending = [...pending, local];
		flows.notice = null;
		retried = null;
		newFromId = null;
		recovered = null;
		jumpToLatest();
		if (!navigator.onLine) {
			outbox.keep(outboxKey, text);
			setPending(local.key, { status: 'queued' });
			online = false;
			return;
		}
		turn = turnSent(turn, Date.now());
		const result = await callJson<{ message: MessageItem; reply: MessageItem | null; stopped: boolean }>(`${api}/messages`, { method: 'POST', body: { text } });
		if (result.ok) {
			pending = pending.filter((item) => item.key !== local.key);
			messages = upsert(messages, { ...result.data.message, content_html: '' });
			onactivity?.();
			if (options.late) lateNotes = { ...lateNotes, [result.data.message.id]: m.sent_after_signin({ time: clock(new Date(), time.timeZone) }) };
			if (options.note) sentNotes = { ...sentNotes, [result.data.message.id]: options.note };
			if (result.data.stopped) {
				// The supervisor answered on the spot: no turn is coming.
				hidden.add(result.data.message.id);
				if (result.data.reply) hidden.add(result.data.reply.id);
				messages = messages.filter((message) => !hidden.has(message.id));
				turn = turnStopped(turn, Date.now());
			}
			return;
		}
		turn = idleTurn();
		if (result.offline) {
			outbox.keep(outboxKey, text);
			setPending(local.key, { status: 'queued' });
			online = false;
		} else if (result.signedOut) {
			pending = pending.filter((item) => item.key !== local.key);
			outbox.keep(`${outboxKey}#signin`, text);
			header.signedOutDraft = text;
		} else {
			const reason = result.status === 413 ? m.send_failed_long() : result.status === 429 ? m.send_failed_busy() : m.send_failed_server({ status: String(result.status) });
			setPending(local.key, { status: 'failed', reason });
		}
	}

	function setPending(key: number, change: Partial<Pending>) {
		pending = pending.map((item) => (item.key === key ? { ...item, ...change } : item));
	}

	function retryPending(item: Pending) {
		pending = pending.filter((one) => one.key !== item.key);
		void send(item.text);
	}

	function editPending(item: Pending) {
		pending = pending.filter((one) => one.key !== item.key);
		// Its files come back as chips (the backend reads each tag against its own records).
		const parts = splitAttachments(item.text);
		draft = parts.text;
		uploads.restore(parts.files.map((file) => ({ id: file.id, name: file.name, kind: file.kind, mime: file.mime ?? '', size: file.size ?? 0, status: 'ready', reference: `<attachment id="${file.id}" name="${file.name}" kind="${file.kind}" mime="${file.mime ?? ''}" size="${file.size ?? 0}"/>` })));
		field?.focus();
	}

	function cancelQueued(item: Pending) {
		outbox.take(outboxKey);
		pending = pending.filter((one) => one.key !== item.key);
	}

	/** Sends what waits on this device: from the outbox once online, or after signing in again. */
	async function flushOutbox(): Promise<boolean> {
		if (!navigator.onLine) return false;
		const late = outbox.take(`${outboxKey}#signin`);
		if (late !== null) await send(late, { late: true });
		const queued = outbox.take(outboxKey);
		if (queued === null) return late !== null;
		pending = pending.filter((item) => item.status !== 'queued');
		await send(queued);
		return true;
	}

	/** Stop: the same "stop" a person could type, answered by the supervisor (there's no endpoint). */
	async function stop() {
		if (!working) return;
		const result = await callJson<{ message: MessageItem; reply: MessageItem | null; stopped: boolean }>(`${api}/messages`, { method: 'POST', body: { text: 'stop' } });
		if (!result.ok) {
			toasts.show({ kind: 'error', title: result.message || m.rerun_failed() });
			return;
		}
		hidden.add(result.data.message.id);
		if (result.data.reply) hidden.add(result.data.reply.id);
		messages = messages.filter((message) => !hidden.has(message.id));
		turn = turnStopped(turn, Date.now());
	}

	/** Stop all (the crew-working banner): the same "stop", even while no reply of this turn streams. */
	async function stopAll() {
		if (working) return stop();
		const result = await callJson<{ message: MessageItem; reply: MessageItem | null; stopped: boolean }>(`${api}/messages`, { method: 'POST', body: { text: 'stop' } });
		if (!result.ok) toasts.show({ kind: 'error', title: result.message || m.rerun_failed() });
	}

	/** Who of the crew is at work in the chat already (the feed only says who starts from now). */
	async function readCrewAtWork() {
		const result = await callJson<AgentStatus | null>(`${api}/agent-status`);
		if (!result.ok) return;
		if (result.data) {
			const agentName = crewNameOf(result.data.agent_type);
			flows.sessionStarted(result.data.agent_session_id, agentName);
			if (!turn.stopRequested) {
				turn = turnWithAgentActivity(
					turn,
					{
						agent_session_id: result.data.agent_session_id,
						agent_name: agentName,
						phase: result.data.current_phase,
						detail: result.data.current_phase_detail,
					},
					Date.now(),
				);
			}
			header.crew = { replied: header.crew.replied, working: agentName };
			onactivity?.();
		} else if (!turn.pending && turn.openTurns.length > 0 && !turn.reply.text.trim()) {
			turn = idleTurn();
			header.crew = { replied: header.crew.replied, working: null };
		}
	}

	function continueReply() {
		if (turn.end?.status === 'interrupted') continued = { after: new Date().toISOString(), elapsed: `${Math.round(turn.end.elapsedMs / 1000)}s` };
		void send(m.reply_continue_message());
	}

	// The latest turn running again: the person's message edited, or the reply asked for again.
	function rerunStarted(done: RerunResult, update: Partial<RenderedMessage>) {
		messages = afterRerun(messages, done.message_id, { versions: done.total - 1, ...update });
		const { [done.message_id]: _dropped, ...rest } = versions;
		versions = rest;
		viewing = { ...viewing, [done.message_id]: 0 };
		turn = turnSent(turn, Date.now());
		jumpToLatest();
	}

	async function regenerate(replyId: string, style: RegenerateStyle) {
		const result = await callJson<RerunResult>(`${api}/messages/${replyId}/regenerate`, { method: 'POST', body: { style } });
		if (!result.ok) return void toasts.show({ kind: 'error', title: m.rerun_failed(), detail: result.message || undefined });
		rerunStarted(result.data, {});
	}

	async function saveEdit(message: RenderedMessage, text: string) {
		const result = await callJson<RerunResult>(`${api}/messages/${message.id}`, { method: 'PATCH', body: { text } });
		if (!result.ok) return void toasts.show({ kind: 'error', title: m.rerun_failed(), detail: result.message || undefined });
		editing = null;
		rerunStarted(result.data, { content: text, edited_at: result.data.edited_at });
	}

	/** Steps between the takes of a person's message; the earlier ones load once. */
	async function step(message: RenderedMessage, to: number) {
		const total = (message.versions ?? 0) + 1;
		if (to < 1 || to > total) return;
		if (to < total && !versions[message.id]) {
			const result = await callJson<MessageVersion[]>(`${api}/messages/${message.id}/versions`);
			if (!result.ok) return void toasts.show({ kind: 'error', title: m.versions_failed() });
			versions = { ...versions, [message.id]: result.data };
		}
		viewing = { ...viewing, [message.id]: to === total ? 0 : to };
	}

	async function rate(message: RenderedMessage, rating: 'up' | 'down' | null): Promise<boolean> {
		const result = rating
			? await callJson(`${api}/messages/${message.id}/feedback`, { method: 'PUT', body: { rating } })
			: await callJson(`${api}/messages/${message.id}/feedback`, { method: 'DELETE' });
		if (!result.ok) toasts.show({ kind: 'error', title: m.feedback_failed() });
		return result.ok;
	}

	async function dislike(message: RenderedMessage, feedback: Dislike): Promise<boolean> {
		const result = await callJson(`${api}/messages/${message.id}/feedback`, { method: 'PUT', body: { rating: 'down', ...feedback } });
		if (!result.ok) toasts.show({ kind: 'error', title: m.feedback_failed() });
		return result.ok;
	}

	async function retryRun(message: RenderedMessage, reason: string, credits = false) {
		const result = await callJson(`${api}/messages/${message.id}/retry`, { method: 'POST' });
		if (!result.ok) return void toasts.show({ kind: 'error', title: m.retry_failed(), detail: result.message || undefined });
		messages = messages.filter((one) => one.id !== message.id);
		recovered = { after: new Date().toISOString(), reason, credits };
		turn = turnSent(turn, Date.now());
	}

	async function useModel(key: string): Promise<boolean> {
		const result = await callJson(`${api}/model`, { method: 'PUT', body: { key } });
		if (!result.ok) {
			toasts.show({ kind: 'error', title: m.model_failed() });
			return false;
		}
		modelKey = key;
		return true;
	}

	async function useThinking(level: string) {
		const before = thinking;
		thinking = level as ThinkingLevel;
		const result = await callJson(`${api}/thinking`, { method: 'PUT', body: { level } });
		if (!result.ok) {
			thinking = before;
			toasts.show({ kind: 'error', title: m.thinking_failed() });
		}
	}

	/** The model picker in the composer, opened from a failure's "Choose a model". */
	function openModelPicker() {
		const chip = [...document.querySelectorAll<HTMLButtonElement>('[data-composer] button[aria-haspopup="menu"]')].find((button) => button.textContent?.trim() === modelLabel);
		chip?.click();
	}

	function failureActions(message: RenderedMessage, failure: { kind: Parameters<typeof failureView>[0]; text: string; retry: boolean }) {
		const view = failureView(failure.kind, { retry: failure.retry, canUseCredits });
		const ACTIONS: Record<FailureAction, { label: string; icon?: typeof RotateCcw; onclick?: () => void; href?: string }> = {
			retry: { label: m.action_retry(), icon: RotateCcw, onclick: () => retryRun(message, failure.text) },
			credits: {
				label: m.action_switch_credits(),
				icon: Sparkles,
				onclick: async () => {
					const credits = data.models.groups.find((group) => group.source === 'nomi')?.models.find((option) => option.enabled);
					if (credits && (await useModel(credits.key)) && failure.retry) await retryRun(message, failure.text);
				},
			},
			models: { label: m.action_choose_model(), icon: Cpu, onclick: openModelPicker },
			own_key: { label: m.action_own_key(), icon: KeyRound, href: '/settings/models' },
			upgrade: { label: m.action_upgrade(), icon: ArrowUpRight, href: '/settings/plan' },
			new_chat: { label: m.action_new_chat(), icon: FilePlus2, href: '/chat/new' },
		};
		return { title: view.title, actions: view.actions.map((action) => ACTIONS[action]) };
	}

	/** A failure on a phone (Errors & recovery AyrHG): the phone's title and labels, the desktop's moves. */
	function phoneFailure(message: RenderedMessage, failure: { kind: Parameters<typeof failureView>[0]; text: string; retry: boolean }) {
		const choices = data.models.groups.flatMap((group) => group.models.filter((option) => option.enabled).map((option) => ({ key: option.key, label: option.label, context: option.context })));
		const view = phoneFailureView(failure.kind, { retry: failure.retry, canUseCredits, models: choices, current: modelKey });
		const credits = data.models.groups.find((group) => group.source === 'nomi')?.models.find((option) => option.enabled);
		const toAction = (action: PhoneFailureAction) => {
			switch (action.action) {
				case 'retry':
					return { label: action.label, icon: RotateCcw, onclick: () => void retryRun(message, failure.text) };
				case 'credits':
					// No icon: the pen's sparkles are banned by Nomi's UI rules (AGENTS.md §9).
					return { label: action.label, onclick: async () => void (credits && (await useModel(credits.key)) && failure.retry && (await retryRun(message, failure.text, true))) };
				case 'switch':
					return { label: action.label, onclick: async () => void (action.model && (await useModel(action.model)) && failure.retry && (await retryRun(message, failure.text))) };
				case 'models':
					return { label: action.label, onclick: () => (sheet = { kind: 'model', ...(failure.retry && { retry: { message, reason: failure.text } }) }) };
				case 'own_key':
					return { label: action.label, icon: KeyRound, href: '/settings/models' };
				case 'upgrade':
					return { label: action.label, icon: ArrowUpRight, href: '/settings/plan' };
				case 'new_chat':
					return { label: action.label, icon: FilePlus2, href: '/chat/new' };
			}
		};
		const [primary, secondary] = view.actions.map(toAction);
		const usage = failure.kind === 'quota' ? data.usage : null;
		return {
			title: view.title,
			primary,
			secondary,
			tag: usage ? usage.card.plan : undefined,
			meta: usage ? m.phone_credits_out_meta({ used: usage.tokens.used, allowance: usage.tokens.total, date: resetsOn(time.now, language, time.timeZone) }) : modelLabel,
		};
	}

	// The chat itself: its menu, rename, pin, move, export, share, delete.
	function onMenu(action: ChatAction) {
		if (action === 'rename') header.renaming = true;
		else if (action === 'pin') void pin();
		else if (action === 'move') moveOpen = true;
		else if (action === 'export') exportChat();
		else if (action === 'share') void share();
		else if (action === 'delete') deleteOpen = true;
	}

	const shownTitle = $derived(header.shown(chatId, 'title', data.chat.title));
	// Who answers in this chat, for the phone's nav bar: whoever is at it, else the last to reply.
	const answering = $derived.by(() => {
		const name = working ? (turn.agent ?? 'Nomi') : (messages.findLast((message) => message.sender === 'assistant' && message.agent_display_name)?.agent_display_name ?? 'Nomi');
		// On a phone the nav bar also says who waits on the person, or how many work in the background.
		const asking = phone && !working ? latestApproval : null;
		if (asking) return { agent: crewIdOf(asking), name: m.phone_approval_nav({ agent: asking }) };
		if (phone && !working && flows.working.length > 0) return { agent: crewIdOf(flows.working[0]), name: m.phone_crew_working({ count: String(flows.working.length) }) };
		// A chat in a project names the project under the title (Manage chat & long threads, step 4 NCfNM).
		if (phone && !working && projectName) return { agent: crewIdOf(name), name: projectName };
		return { agent: crewIdOf(name), name: working ? m.chat_nav_working({ name }) : name };
	});
	/** The project the chat is filed under: what the page loaded, or where it was just moved. */
	const projectName = $derived(header.shown(chatId, 'meta', data.projects.find((one) => one.id === data.chat.inProjectId)?.name ?? null));
	/** Who of the crew waits on an approval in the latest reply. */
	const latestApproval = $derived.by(() => {
		const reply = messages.findLast((message) => message.sender === 'assistant');
		const waiting = (reply?.content_blocks ?? []).some((block) => block.kind === 'approval_request' && (block as { status?: string }).status === 'pending');
		return waiting ? nameOf(reply!) : null;
	});
	const pinned = $derived(header.shown(chatId, 'pinned', data.chat.pinned));
	// The chat's sheet says how many messages it holds and who answers (W3a8j G8cFz: "14 messages ·
	// Dana"); the count only once every message is loaded, so it's never short.
	const chatSubtitle = $derived.by(() => {
		const name = working ? (turn.agent ?? 'Nomi') : (messages.findLast((message) => message.sender === 'assistant' && message.agent_display_name)?.agent_display_name ?? 'Nomi');
		return hasOlder ? name : m.chat_sheet_subtitle({ count: messages.length, name });
	});
	// A reply asked for again on another model (Message controls lVDqs, "With another model"): the
	// chat moves to that model (the composer's chip says so), then the turn runs again on it.
	const otherModels = $derived(models.flatMap((group) => group.models).filter((option) => !option.chosen));
	async function regenerateWith(replyId: string, key: string) {
		if (await useModel(key)) await regenerate(replyId, 'again');
	}
	/**
	 * Select text (Message controls sKD62): your message's text, selected in place for the phone's own
	 * handles, once its sheet is gone (and the focus it gives back, which would take the selection).
	 */
	function selectText(id: string) {
		void tick().then(() =>
			requestAnimationFrame(() => {
				const text = document.querySelector(`[data-message-id="${id}"] p`);
				if (!text) return;
				if (document.activeElement instanceof HTMLElement) document.activeElement.blur();
				getSelection()?.selectAllChildren(text);
			}),
		);
	}

	async function patch(body: Record<string, unknown>): Promise<boolean> {
		const result = await callJson(`${api}/session`, { method: 'PATCH', body });
		if (!result.ok) toasts.show({ kind: 'error', title: m.chat_change_failed(), detail: result.message || undefined });
		else void invalidate('app:shell');
		return result.ok;
	}

	async function rename(name: string) {
		header.renaming = false;
		const before = shownTitle;
		if (!name.trim() || name.trim() === before) return;
		header.title = name.trim();
		if (!(await patch({ title: name.trim() }))) header.title = before;
	}

	async function pin() {
		const next = !pinned;
		if (!(await patch({ pinned: next }))) return;
		header.pinned = next;
		toasts.show({ kind: 'success', title: next ? m.chat_pinned_toast() : m.chat_unpinned_toast() });
	}

	async function move(projectId: string) {
		const before = data.chat.inProjectId;
		const project = data.projects.find((one) => one.id === projectId);
		if (!(await patch({ in_project_id: projectId }))) return;
		header.meta = project?.name ?? null;
		toasts.show({
			kind: 'success',
			icon: FolderCheck,
			title: m.chat_moved({ project: project?.name ?? '' }),
			// The pen's NCfNM says a shared workspace's members can see it now; the backend keeps a filed
			// chat private (moveTargets), so the detail says that instead (AGENTS.md §9).
			detail: m.chat_moved_detail(),
			duration: 10_000,
			action: {
				label: m.action_undo(),
				onclick: async () => {
					if (await patch({ in_project_id: before })) header.meta = data.projects.find((one) => one.id === before)?.name ?? data.meta;
				},
			},
		});
	}

	function exportChat() {
		const url = URL.createObjectURL(new Blob([chatMarkdown(shownTitle, messages, m.export_you())], { type: 'text/markdown;charset=utf-8' }));
		const anchor = document.createElement('a');
		anchor.href = url;
		anchor.download = chatFileName(shownTitle);
		anchor.click();
		URL.revokeObjectURL(url);
	}

	// Share & delete (TnscX): a link to the whole chat (the top bar's Share) or one reply (the reply's
	// Share), masked and open to anyone for 7 days at first; the dialog changes what it shares, who
	// may open it and for how long, on the same link.
	type ShareLink = { id: string; token: string; scope: 'chat' | 'reply'; message_id: string | null; mask: boolean; anyone: boolean; expires_at: string | null; preview: string; preview_by: string | null; chat_messages: number };
	const EXPIRY = [7, 30, null, 1] as const;
	let shareOpen = $state(false);
	let shareLink = $state<ShareLink | null>(null);
	let shareDays = $state<(typeof EXPIRY)[number]>(7);
	let shareReply = $state<string | null>(null);
	const shareUrl = $derived(shareLink ? `${page.url.origin}/shared/${shareLink.token}` : '');

	async function saveShare(settings: { scope: 'chat' | 'reply'; mask: boolean; anyone: boolean }, days: (typeof EXPIRY)[number]) {
		const body = { ...settings, message_id: settings.scope === 'reply' ? shareReply : null, expires_in_days: days };
		const result = shareLink ? await callJson<ShareLink>(`${api}/share/${shareLink.id}`, { method: 'PUT', body }) : await callJson<ShareLink>(`${api}/share`, { method: 'POST', body });
		if (!result.ok) return void toasts.show({ kind: 'error', title: m.link_failed(), detail: result.message || undefined });
		shareLink = result.data;
		shareDays = days;
	}

	async function share(replyId?: string) {
		shareLink = null;
		shareReply = replyId ?? null;
		await saveShare({ scope: replyId ? 'reply' : 'chat', mask: true, anyone: true }, 7);
		if (shareLink) shareOpen = true;
	}

	async function copyShare() {
		await navigator.clipboard?.writeText(shareUrl).catch(() => undefined);
		toasts.show({ kind: 'success', title: m.link_copied(), detail: shareUrl.replace(/^https?:\/\//, '') });
	}

	// Delete reply (Share & delete yoVD6, steps 4–5): asked first in an action sheet that says what
	// goes with it (the backend's preview: what the reply's cards made); then a placeholder stands
	// in the thread and the toast offers Undo for ten seconds before the reply goes for good, as a
	// deleted chat does (the backend can't bring a reply back). Leaving the page deletes it at once.
	let replyDelete = $state<{ message: RenderedMessage; body: string } | null>(null);
	let deletedReplies = $state<Record<string, 'waiting' | 'gone'>>({});
	async function askDeleteReply(message: RenderedMessage) {
		const result = await callJson<DeletePreview>(`${api}/messages/${message.id}/delete`);
		if (!result.ok) return void toasts.show({ kind: 'error', title: m.reply_delete_failed(), detail: result.message || undefined });
		const made = [...result.data.reminders, ...result.data.plans, ...result.data.tasks, ...result.data.transactions, ...result.data.files].map((item) => item.label);
		replyDelete = { message, body: made.length ? m.delete_reply_body_phone_items({ items: made.join(', ') }) : m.delete_reply_body_phone() };
	}
	function deleteReplyLater(message: RenderedMessage) {
		const id = message.id;
		const path = `${api}/messages/${id}`;
		replyDelete = null;
		deletedReplies = { ...deletedReplies, [id]: 'waiting' };
		later.schedule(
			`reply:${id}`,
			() => {
				void fetch(path, { method: 'DELETE', keepalive: true })
					.then((response) => {
						if (!response.ok) throw new Error(String(response.status));
						deletedReplies = { ...deletedReplies, [id]: 'gone' };
					})
					.catch(() => {
						const { [id]: _back, ...rest } = deletedReplies;
						deletedReplies = rest;
						toasts.show({ kind: 'error', title: m.reply_delete_failed() });
					});
			},
			10_000,
		);
		toasts.show({ kind: 'info', icon: Trash2, title: m.reply_deleted(), detail: m.reply_deleted_detail(), duration: 10_000, action: { label: m.action_undo(), onclick: () => undoDeleteReply(id) } });
	}
	function undoDeleteReply(id: string) {
		if (!later.undo(`reply:${id}`)) return;
		const { [id]: _back, ...rest } = deletedReplies;
		deletedReplies = rest;
	}

	/** Deleted after ten seconds to undo, from Home; leaving the page deletes it at once. */
	async function deleteChat() {
		deleteOpen = false;
		const id = chatId;
		const count = messages.length;
		header.leaving = [...header.leaving, id];
		later.schedule(
			id,
			() => {
				void fetch(`/chat/${id}/api/session`, { method: 'DELETE', keepalive: true })
					.then((response) => {
						if (!response.ok) throw new Error(String(response.status));
						return invalidate('app:shell');
					})
					.catch(() => {
						header.leaving = header.leaving.filter((one) => one !== id);
						toasts.show({ kind: 'error', title: m.chat_delete_failed() });
					})
					.finally(() => (header.leaving = header.leaving.filter((one) => one !== id)));
			},
			10_000,
		);
		await goto('/');
		toasts.show({
			kind: 'info',
			icon: Trash2,
			title: m.chat_deleted(),
			detail: m.chat_deleted_detail({ count }),
			duration: 10_000,
			action: {
				label: m.action_undo(),
				onclick: () => {
					if (later.undo(id)) {
						header.leaving = header.leaving.filter((one) => one !== id);
						void goto(`/chat/${id}`);
					}
				},
			},
		});
	}

	// Scrolling: new things follow while the person is at the latest; scrolled up, they're counted
	// on the pill and the NEW divider marks where they start. The person's own message brings them back.
	function onscroll() {
		if (!scroller) return;
		if (scroller.scrollTop < 240) void loadOlder();
		atLatest = scroller.scrollHeight - scroller.scrollTop - scroller.clientHeight < 80;
		if (atLatest) unseen = 0;
	}

	function jumpToLatest(smooth = false) {
		atLatest = true;
		unseen = 0;
		void tick().then(() => scroller?.scrollTo({ top: scroller.scrollHeight, behavior: smooth ? 'smooth' : 'instant' }));
	}

	let seenCount = -1;
	let prepending = false;
	$effect(() => {
		const count = messages.length;
		const added = seenCount < 0 ? 0 : count - seenCount;
		seenCount = count;
		void turn.reply.text;
		void pending.length;
		if (prepending) return;
		untrack(() => {
			if (atLatest) jumpToLatest();
			else if (added > 0) {
				if (newFromId === null) newFromId = messages[count - added]?.id ?? null;
				unseen += added;
			}
		});
	});

	async function loadOlder() {
		const first = messages[0];
		if (loadingOlder || !hasOlder || !first) return;
		loadingOlder = true;
		olderFailed = false;
		const result = await callJson<{ messages: RenderedMessage[]; hasOlder: boolean }>(`${api}/messages?before=${encodeURIComponent(first.id)}`);
		if (result.ok && scroller) {
			const fromBottom = scroller.scrollHeight - scroller.scrollTop;
			prepending = true;
			messages = prependOlder(result.data.messages, messages);
			hasOlder = result.data.hasOlder;
			await tick();
			scroller.scrollTop = scroller.scrollHeight - fromBottom;
			prepending = false;
		} else olderFailed = true;
		loadingOlder = false;
	}

	function onkeydown(event: KeyboardEvent) {
		if (event.defaultPrevented || document.querySelector('dialog[open], [role="dialog"], [role="menu"], [role="listbox"]')) return;
		if (event.key === 'Escape' && working) {
			event.preventDefault();
			void stop();
		} else if (event.key === 'F2') {
			event.preventDefault();
			header.renaming = true;
		}
	}

	onMount(() => {
		// Back from connecting an app the crew asked for: say so once, then drop the mark from the address.
		const url = new URL(location.href);
		if (url.searchParams.get('connected') === 'google') {
			connectedNote = m.chat_connected_google();
			url.searchParams.delete('connected');
			history.replaceState(history.state, '', url);
		}
		online = navigator.onLine;
		let hide: ReturnType<typeof setTimeout> | undefined;
		const offline = () => {
			online = false;
			backOnline = false;
		};
		const back = async () => {
			online = true;
			backOnline = await flushOutbox();
			clearTimeout(hide);
			hide = setTimeout(() => (backOnline = false), 4000);
		};
		window.addEventListener('offline', offline);
		window.addEventListener('online', back);
		// Signed back in (or reloaded) with a message waiting: it goes now.
		void flushOutbox();
		restoreDraft(chatId);
		void readCrewAtWork();
		const onVisible = () => {
			if (document.visibilityState === 'visible') void readCrewAtWork();
		};
		document.addEventListener('visibilitychange', onVisible);
		if (!matchMedia('(pointer: coarse)').matches) field?.focus();
		jumpToLatest();
		return () => {
			clearTimeout(hide);
			window.removeEventListener('offline', offline);
			window.removeEventListener('online', back);
			document.removeEventListener('visibilitychange', onVisible);
		};
	});

	const nameOf = (message: { agent_display_name: string | null }) => message.agent_display_name ?? 'Nomi';
	const replyMeta = (message: RenderedMessage, turnStart: string | null, extra?: string) =>
		[clock(message.created_at, time.timeZone), turnDuration(turnStart, message.created_at), extra].filter(Boolean).join(' · ');
	const reasoningLabel = (id: string) => (thoughtFor[id] ? m.reasoning_thought_for({ seconds: thoughtFor[id] }) : m.reasoning_thinking());
	/** What the crew is doing, in words, while nothing is written yet. */
	const phaseLabel = $derived.by(() => {
		const phase = turn.phase;
		if (!phase) return m.phase_working();
		if (phase.phase === 'retrying') return m.phase_retrying_in({ time: elapsedLabel(retryInfo(phase.detail, tickNow).seconds * 1000) });
		if (phase.phase === 'calling_tool') return m.phase_calling_tool({ tool: [phase.detail, phase.subject].filter(Boolean).join(' · ') });
		if (phase.phase === 'writing_reply') return m.phase_writing();
		if (phase.phase === 'thinking') return m.reasoning_thinking();
		return m.phase_working();
	});
	const retrying = $derived(turn.phase?.phase === 'retrying' ? retryInfo(turn.phase.detail, tickNow) : null);

	// Files (UX contract audit-chat-files: G3, G4, G6, G12, built on what the backend has). Each file
	// uploads the moment it's picked; a message carries the ready ones as reference tags after its
	// text. A picture with a model that can't see it waits on the fix line (Switch model, Send text
	// only); a file that failed or was refused keeps Send off until it's retried or removed. What's in
	// the composer is kept for a trip to Settings and back (draft.ts).
	let picker: HTMLInputElement | undefined = $state();
	let liveText = $state('');
	let alertText = $state('');
	/** "bca-oct-2026.pdf removed · 1 file ready", after a file that didn't make it is taken out. */
	let removedLine = $state<string | null>(null);
	/** Notes on messages sent from here ("Sent without …"): nothing in the backend keeps them. */
	let sentNotes = $state<Record<string, string>>({});
	let visionOpen = $state(false);
	let switchButton: HTMLButtonElement | undefined = $state();
	/** The failed file whose sheet is open, on a phone. */
	let fileSheet = $state<Chip | null>(null);
	const fixId = $props.id();

	const uploads = new ComposerUploads({
		announce: ({ kind, name }) => {
			if (kind === 'failed') alertText = m.files_live_failed({ name });
			else liveText = kind === 'uploaded' ? m.files_live_uploaded({ name }) : m.files_live_rejected({ name });
		},
		discard: (id) => void fetch(`/attachments/${id}`, { method: 'DELETE', keepalive: true }).catch(() => undefined),
	});

	const currentModel = $derived(data.models.groups.flatMap((group) => group.models).find((option) => option.key === modelKey));
	const canSee = $derived(seesImages(currentModel));
	/** The chips as they show: a ready picture the model can't see is a warning. */
	const chips = $derived(uploads.chips.map((chip): Chip => (chip.status === 'ready' && chip.uploaded?.kind === 'image' && !canSee ? { ...chip, status: 'warning' } : chip)));
	const blocked = $derived(blockedReason(chips));
	const warning = $derived(blocked?.kind === 'warning');
	const blockedText = $derived<string | true | undefined>(
		!blocked ? undefined : blocked.kind === 'rejected' ? m.files_hint_rejected({ name: blocked.name }) : blocked.kind === 'failed' ? (blocked.count === 1 ? m.files_hint_failed_one() : m.files_hint_failed_many({ count: String(blocked.count) })) : true,
	);
	const REASONS = { connection_lost: m.files_reason_connection_lost, server_error: m.files_reason_server_error, storage_full: m.files_reason_storage_full, too_large: m.files_reason_too_large, rejected: m.files_reason_server_error };

	function chipDetail(chip: Chip): string {
		if (chip.status === 'uploading') {
			const progress = m.files_progress(progressParts(chip.progress ?? 0, chip.size));
			return chip.again ? m.files_again({ progress }) : progress;
		}
		if (chip.status === 'failed') return m.files_upload_failed({ reason: REASONS[chip.failure ?? 'server_error']() });
		if (chip.status === 'rejected') return extensionOf(chip.name) ? m.files_rejected({ ext: extensionOf(chip.name) }) : m.files_rejected_plain();
		if (chip.status === 'warning') return m.files_model_warning({ model: modelLabel ?? '' });
		return `${kindLabel({ name: chip.name, kind: chip.uploaded?.kind ?? '' })} · ${formatBytes(chip.size)}`;
	}

	// The warning is said once as it appears; focus moves to Switch model (ModelFix).
	let warned = false;
	$effect(() => {
		if (warning && !warned) liveText = m.files_live_warning({ model: modelLabel ?? '' });
		warned = warning;
	});

	function pickFiles(event: Event & { currentTarget: HTMLInputElement }) {
		const files = event.currentTarget.files;
		if (files?.length) {
			removedLine = null;
			uploads.add([...files]);
		}
		event.currentTarget.value = '';
	}

	/**
	 * Takes a file out; focus goes to the next chip, or the field (the contract's rule). When that
	 * clears the way for the fix line (a picture the model can't see, behind a failed upload), focus
	 * goes to Switch model instead, as when the warning first appears (gate 3, answer 2).
	 */
	async function removeChip(chip: Chip) {
		const at = chips.findIndex((one) => one.key === chip.key);
		const warnedBefore = warning;
		const unfinished = chip.status === 'failed' || chip.status === 'rejected';
		uploads.remove(chip.key);
		if (unfinished) {
			const ready = readyCount(uploads.chips);
			removedLine = ready === 0 ? m.files_removed_none({ name: chip.name }) : ready === 1 ? m.files_removed_one({ name: chip.name }) : m.files_removed_many({ name: chip.name, count: String(ready) });
			liveText = removedLine;
		}
		await tick();
		if (warning && !warnedBefore) return void switchButton?.focus();
		const next = document.querySelectorAll<HTMLElement>('[data-file-chip]')[at];
		(next ?? field)?.focus();
	}

	/** The message as it goes: the text (after any quoted lines), then each ready file's tag. */
	function submit(text: string, note?: string) {
		const quoted = quote;
		if (quoted) onunquote?.();
		const body = composeMessage(quoted ? `${quoted.text}\n\n${text}` : text, uploads.uploaded.map((file) => file.reference));
		if (!body) return;
		uploads.clear();
		removedLine = null;
		clearDraft(chatId);
		void send(body, { note });
	}

	/** Sends without the pictures the model can't see; they're deleted, never sent. */
	function sendTextOnly() {
		visionOpen = false;
		const left = chips.filter((chip) => chip.status === 'warning');
		for (const chip of left) uploads.remove(chip.key);
		const text = draft;
		draft = '';
		submit(text, m.files_sent_without({ name: left.map((chip) => chip.name).join(', '), model: modelLabel ?? '' }));
	}

	const vision = $derived.by(() => {
		const split = visionSplit(data.models.groups, modelKey);
		const groups: VisionGroup[] = [];
		for (const source of ['connection', 'nomi'] as const) {
			const models = split.groups
				.filter((group) => group.source === source)
				.flatMap((group) => group.models.map((model) => ({ key: model.key, label: model.label, meta: m.files_reads_images({ source: source === 'nomi' ? m.models_nomi_credits() : group.label }) })));
			if (models.length) groups.push({ group: source === 'nomi' ? m.models_nomi_credits() : m.models_your_keys(), models });
		}
		const line = split.hidden.length ? hiddenLine(split.hidden, modelLabel ?? '') : null;
		return { groups, hidden: line ? (line.others ? m.files_hidden_many({ model: line.name, count: String(line.others) }) : m.files_hidden_one({ model: line.name })) : undefined };
	});

	async function pickVisionModel(key: string) {
		visionOpen = false;
		if (await useModel(key)) field?.focus();
	}

	function closeVision() {
		visionOpen = false;
		switchButton?.focus();
	}

	function restoreDraft(id: string) {
		const kept = loadDraft(id);
		if (!kept) return;
		if (!draft.trim()) draft = kept.text;
		uploads.restore(kept.files);
	}

	// What's in the composer is kept as it changes (an empty one keeps nothing).
	$effect(() => saveDraft(chatId, { text: draft, files: uploads.uploaded }));

	// The viewer (G12): a sent file opened beside the chat (a PDF at page 1, the browser's own
	// reader drawing it), a photo in the lightbox; anything else downloads.
	type Viewer = { file: SentFile; trigger: HTMLElement | null; status: 'loading' | 'ready' | 'error'; pages: number | null; started: number; code: string | null; attempt: number };
	let viewer = $state<Viewer | null>(null);
	let photo = $state<{ file: SentFile; trigger: HTMLElement | null; index: number; count: number; at: string } | null>(null);
	let viewerNow = $state(Date.now());
	const PREVIEW_TIMEOUT_MS = 30_000;

	function openFile(file: SentFile, all: SentFile[], at: string) {
		const trigger = document.activeElement instanceof HTMLElement ? document.activeElement : null;
		if (file.kind === 'image') {
			const photos = all.filter((one) => one.kind === 'image');
			photo = { file, trigger, index: photos.indexOf(file) + 1, count: photos.length, at };
		} else if (file.kind === 'pdf') {
			viewer = { file, trigger, status: 'loading', pages: null, started: Date.now(), code: null, attempt: 0 };
			void loadViewer();
		} else window.location.assign(`/attachments/${file.id}/content?download`);
	}

	async function loadViewer() {
		const opened = viewer;
		if (!opened) return;
		const attempt = opened.attempt + 1;
		viewer = { ...opened, status: 'loading', code: null, started: Date.now(), attempt };
		const response = await fetch(`/attachments/${opened.file.id}`).catch(() => null);
		if (viewer?.attempt !== attempt) return;
		if (!response?.ok) {
			viewer = { ...viewer, status: 'error', code: `preview_failed · ${response?.status ?? 'offline'}` };
			liveText = m.files_viewer_error_title();
			return;
		}
		const details = ((await response.json().catch(() => null)) as { details?: { pages?: number } } | null)?.details;
		viewer = { ...viewer, pages: details?.pages || null };
		setTimeout(() => {
			if (viewer?.attempt === attempt && viewer.status === 'loading') {
				viewer = { ...viewer, status: 'error', code: 'preview_timeout · 30 s' };
				liveText = m.files_viewer_error_title();
			}
		}, PREVIEW_TIMEOUT_MS);
	}

	function viewerLoaded() {
		if (!viewer || viewer.status !== 'loading') return;
		viewer = { ...viewer, status: 'ready' };
		liveText = viewer.pages ? m.files_live_page({ page: '1', pages: String(viewer.pages) }) : '';
	}

	async function closeViewer() {
		const trigger = viewer?.trigger ?? photo?.trigger;
		viewer = null;
		photo = null;
		await tick();
		trigger?.focus();
	}

	$effect(() => {
		if (viewer?.status !== 'loading') return;
		viewerNow = Date.now();
		const timer = setInterval(() => (viewerNow = Date.now()), 1000);
		return () => clearInterval(timer);
	});

	/** "0:02", as the viewer's step says it. */
	const shortElapsed = (ms: number) => {
		const seconds = Math.max(0, Math.floor(ms / 1000));
		return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, '0')}`;
	};
	const pagesText = (pages: number | null) => (pages === null ? null : pages === 1 ? m.files_pages_one() : m.files_pages_many({ count: String(pages) }));
	const fileMeta = (file: SentFile) => [kindLabel(file), file.size !== undefined ? formatBytes(file.size) : null].filter(Boolean).join(' · ');

	/** A sent message's files, as the chips over its bubble. */
	function sentItems(files: SentFile[], at: string): SentFileItem[] {
		return files.map((file) => ({
			name: file.name,
			meta: fileMeta(file),
			image: file.kind === 'image' ? `/attachments/${file.id}/content` : undefined,
			onopen: () => openFile(file, files, at),
		}));
	}
</script>

<svelte:window {onkeydown} />


{#snippet userMessage(message: RenderedMessage, latest: boolean)}
	{@const total = (message.versions ?? 0) + 1}
	{@const shown = viewing[message.id] ? versions[message.id]?.[viewing[message.id] - 1] : undefined}
	{@const parts = splitAttachments(shown?.text ?? message.content)}
	{#if editing === message.id && phone}
		<PhoneMessageEditor value={message.content} version={m.edit_saves_as({ version: total + 1 })} onsave={(text) => saveEdit(message, text)} oncancel={() => (editing = null)} />
	{:else if editing === message.id}
		<MessageEditor value={message.content} version="{total} / {total + 1}" onsave={(text) => saveEdit(message, text)} oncancel={() => (editing = null)} />
	{:else if phone}
		{#if parts.files.length}<SentFiles phone files={sentItems(parts.files, clock(message.created_at, time.timeZone))} />{/if}
		{#if parts.text}
			<!-- Long-pressing your message opens its sheet (Message controls, step 3), lifting the bubble
			     over the scrim: its place in the thread stays, empty, until the sheet closes (nzT7f). -->
			<!-- A message with files isn't edited in place (editing with files is P1), so its sheet has no Edit. -->
			<div class={[sheet?.kind === 'mine' && sheet.message.id === message.id && 'invisible']} use:longPress={() => (sheet = { kind: 'mine', message, latest: latest && !working && !shown && !parts.files.length })}>
				<PhoneUserMessage text={parts.text} />
			</div>
		{/if}
		{#if sentNotes[message.id]}<FileNote icon={ImageOff} mono align="end" text={sentNotes[message.id]} />{/if}
		{#if lateNotes[message.id]}<TurnNote icon={LogIn} align="end" text={lateNotes[message.id]} />{/if}
	{:else}
		{#if parts.files.length}<SentFiles files={sentItems(parts.files, clock(message.created_at, time.timeZone))} />{/if}
		{#if parts.text}<MessageUser
			text={parts.text}
			time={sentNotes[message.id] ? undefined : clock(message.edited_at ?? message.created_at, time.timeZone)}
			edited={Boolean(message.edited_at) && !shown}
			version={total > 1 && message.edited_at ? { current: viewing[message.id] || total, total, onprev: () => step(message, (viewing[message.id] || total) - 1), onnext: () => step(message, (viewing[message.id] || total) + 1) } : undefined}
			onedit={latest && !working && !shown && !parts.files.length && !sentNotes[message.id] ? () => (editing = message.id) : undefined}
		/>{/if}
		{#if sentNotes[message.id]}<FileNote icon={ImageOff} mono align="end" text={sentNotes[message.id]} />{/if}
		{#if lateNotes[message.id]}<TurnNote icon={LogIn} align="end" text={lateNotes[message.id]} />{/if}
	{/if}
{/snippet}

{#snippet composerFiles()}
	<div class={['flex flex-col', phone ? 'gap-2' : 'gap-1.5']}>
		{#if chips.length}
			<div class={['flex', phone ? 'flex-col items-start gap-1.5' : 'flex-wrap items-center gap-2']}>
				{#each chips as chip (chip.key)}
					<ComposerFile
						name={chip.name}
						status={chip.status}
						detail={chipDetail(chip)}
						progress={chip.progress}
						thumb={chip.thumb}
						{phone}
						onretry={() => uploads.retry(chip.key)}
						onremove={() => void removeChip(chip)}
						onmore={() => (fileSheet = chip)}
					/>
				{/each}
			</div>
		{/if}
		<!-- A phone's Notice is padded [0,4] (S61SO), as the failed-upload Notice is; a desktop's helper sits flush. -->
		{#if removedLine}<div class={[phone && 'px-1']}><FileNote icon={phone ? Check : undefined} mono text={removedLine} /></div>{/if}
	</div>
{/snippet}

{#snippet modelFix()}
	<div class="relative">
		<ModelFix {phone} focus labelId="{fixId}-fix" bind:switchButton onswitch={() => (visionOpen = !visionOpen)} ontextonly={sendTextOnly} />
		{#if visionOpen && !phone}
			<div class="absolute right-0 bottom-full z-20 mb-2">
				<VisionModelMenu groups={vision.groups} hidden={vision.hidden} settingsHref="/settings/models" onpick={(key) => void pickVisionModel(key)} ontextonly={sendTextOnly} onclose={closeVision} />
			</div>
		{/if}
	</div>
{/snippet}

{#snippet fileViewer(open: Viewer)}
	{@const pages = pagesText(open.pages)}
	<FileViewer
		name={open.file.name}
		meta={[kindLabel(open.file), pages ?? (open.file.size !== undefined ? formatBytes(open.file.size) : null), phone ? m.files_yours_phone() : m.files_yours()].filter(Boolean).join(' · ')}
		status={open.status}
		step={open.pages ? m.files_opening({ page: '1', pages: String(open.pages) }) : m.files_opening_page({ page: '1' })}
		elapsed={shortElapsed(viewerNow - open.started)}
		note={phone ? m.files_viewer_note_phone() : m.files_viewer_note()}
		error={open.code ? { body: m.files_viewer_error_body({ page: '1' }), code: open.code } : undefined}
		{phone}
		downloadHref="/attachments/{open.file.id}/content?download"
		onretry={() => void loadViewer()}
		onclose={() => void closeViewer()}
	>
		{#key open.attempt}
			<iframe title={open.file.name} src="/attachments/{open.file.id}/content#page=1" class="min-h-0 w-full flex-1 rounded-sm bg-surface outline -outline-offset-1 outline-border" onload={viewerLoaded}></iframe>
		{/key}
	</FileViewer>
{/snippet}

<div class="flex h-full">
<div class="flex h-full min-w-0 flex-1 flex-col">
	{#if phone && !pane}<NavBar title={shownTitle ?? m.nav_new_chat()} who={answering} onback={() => void goto('/chats')} onmenu={() => (sheet = { kind: 'chat' })} />{/if}
	{#if phone && (!online || backOnline || quota !== null || flows.working.length > 0)}
		<!-- On a phone the banners stay under the nav bar, wherever the person is in the thread (UX
		     contract S3.4, S4 5.1, 6.2): the connection or the month's credits, then who's at work
		     (Approvals & background HsE8N) with Stop all. -->
		<div data-phone-banners class="flex flex-col gap-2 px-4 pt-3">
			{#if !online}
				<Banner kind="offline" text={m.phone_net_offline()} action={{ label: m.banner_retry_now(), onclick: () => void flushOutbox() }} />
			{:else if backOnline}
				<Banner kind="online" text={pending.length === 0 ? m.banner_online_sent() : m.banner_online()} />
			{:else if quota !== null}
				<Banner kind="quota" text={m.phone_credits_low({ percent: String(quota), date: resetsOn(time.now, language, time.timeZone) })} action={{ label: m.phone_credits_usage(), href: '/settings/plan' }} />
			{/if}
			{#if flows.working.length > 0}<PhoneCrewWorking crew={flows.working.map((name) => crewIdOf(name))} onstop={() => void stopAll()} />{/if}
		</div>
	{/if}
	<!-- The thread scrolls up and down only: nothing in it may slide it sideways on a phone. -->
	<div bind:this={scroller} {onscroll} class={['min-h-0 flex-1 overflow-x-hidden overflow-y-auto', phone ? 'px-4 pt-4' : pane ? 'px-5 pt-5' : 'px-4 pt-6 lg:px-12 lg:pt-10']}>
		<!-- On a phone a short thread sits at the bottom, over the composer, as the pen draws it. -->
		<div class={['mx-auto flex w-full max-w-180 flex-col wrap-anywhere', phone ? 'min-h-full justify-end gap-6 pb-5' : 'gap-8 pb-8']}>
			{#if !phone}
				{#if !online}
					<Banner kind="offline" text={m.banner_offline()} action={{ label: m.banner_retry_now(), onclick: () => void flushOutbox() }} />
				{:else if backOnline}
					<Banner kind="online" text={pending.length === 0 ? m.banner_online_sent() : m.banner_online()} />
				{:else if quota !== null}
					<Banner kind="quota" text={m.banner_quota({ percent: quota })} action={{ label: m.banner_see_usage(), href: '/settings/plan' }} />
				{/if}
			{/if}

			{#if hasOlder}
				{#if loadingOlder}
					<LoadingOlder />
				{:else}
					<button type="button" class="cursor-pointer self-center rounded-xs font-mono text-label text-text-3 outline-offset-2 hover:text-text pointer-coarse:min-h-11" onclick={loadOlder}>
						{olderFailed ? m.chat_older_failed() : m.chat_load_older()}
					</button>
				{/if}
			{/if}

			{#each entries as entry, index (entry.type === 'day' ? `day-${entry.key}` : entry.type === 'new' ? 'new' : entry.message.id)}
				{#if entry.type === 'day'}
					<!-- On a phone a thread of one day has no divider (Answer yr0gh). -->
					{#if !phone || entries.filter((one) => one.type === 'day').length > 1}<DateDivider label={entry.label} />{/if}
				{:else if entry.type === 'new'}
					<NewDivider count={entry.count} />
				{:else if entry.type === 'user' && entry.message.author_name}
					<!-- A teammate's message in a shared project chat (Invite & build together TnwQT, step 6): hers, in her colour, with whom she asked. -->
					{@const author = entry.message.author_name}
					{@const next = entries.slice(index + 1).find((one) => one.type !== 'day' && one.type !== 'new')}
					<div data-message-id={entry.message.id}>
						<TeammateMessage
							who={{ name: author, initials: initials(author), peer: peerFor(author) }}
							time={clock(entry.message.created_at, time.timeZone)}
							to={next && next.type !== 'user' ? (next.message.agent_display_name ?? undefined) : undefined}
							text={entry.message.content}
						/>
					</div>
				{:else if entry.type === 'user'}
					<div class="flex flex-col gap-1.5" data-message-id={entry.message.id}>{@render userMessage(entry.message, entry.latest)}</div>
				{:else}
					{@const message = entry.message}
					{@const start = entries.slice(0, index).findLast((one) => one.type === 'user')}
					{@const asked = start?.type === 'user' ? start.message : undefined}
					{@const takes = asked ? (asked.versions ?? 0) + 1 : 1}
					{@const earlier = asked && viewing[asked.id] ? versions[asked.id]?.[viewing[asked.id] - 1] : undefined}
					{@const firstOfTurn = entries[index - 1]?.type !== 'reply'}
					{#if earlier}
						{#if firstOfTurn}
							{#each earlier.replies as reply, at (at)}
								<AgentReply agent={crewIdOf(reply.agent_display_name)} name={reply.agent_display_name ?? 'Nomi'} meta={m.version_earlier({ current: viewing[asked!.id], total: takes })} version={!asked!.edited_at && at === 0 ? { current: viewing[asked!.id], total: takes, onprev: () => step(asked!, viewing[asked!.id] - 1), onnext: () => step(asked!, viewing[asked!.id] + 1) } : undefined}>
									<Prose html={reply.content_html} />
								</AgentReply>
							{/each}
						{/if}
					{:else if message.id in deletedReplies}
					<div data-message-id={message.id} class="flex flex-col">
						<DeletedReply name={nameOf(message)} onundo={deletedReplies[message.id] === 'waiting' ? () => undoDeleteReply(message.id) : undefined} />
					</div>
					{:else}
					<div data-message-id={message.id} class="flex flex-col" use:longPress={() => phone && (sheet = { kind: 'reply', message, latest: entry.latest, asked })}>
							<AgentReply
								{phone}
								agent={crewIdOf(message.agent_display_name)}
								name={nameOf(message)}
								meta={replyMeta(
									message,
									entry.turnStart,
									cutOff === message.id
										? m.meta_stopped()
										: continued && message.created_at > continued.after
											? m.meta_continued()
											: phone && firstOfTurn && recovered?.credits && message.created_at > recovered.after
												? m.phone_meta_credits()
												: phone && firstOfTurn && retried && message.created_at > retried.after
													? m.phone_meta_retried()
													: undefined,
								)}
								version={asked && takes > 1 && !asked.edited_at && firstOfTurn ? { current: takes, total: takes, onprev: () => step(asked, takes - 1), onnext: () => {} } : undefined}
								dimmed={editing !== null && asked?.id === editing}
							>
								{#if recovered && firstOfTurn && message.created_at > recovered.after}
									<TurnNote kind="box" icon={RotateCcw} text={phone && recovered.credits ? m.phone_recovered_credits({ reason: recovered.reason }) : m.recovered_note({ reason: recovered.reason })} />
								{:else if phone && retried && firstOfTurn && message.created_at > retried.after}
									<TurnNote kind="box" icon={Hourglass} text={m.phone_retried_note({ seconds: String(retried.seconds) })} />
								{/if}
								{#if entry.reasoning.length > 0}
									<Reasoning label={reasoningLabel(message.id)} text={entry.reasoning.join('\n\n')} />
								{/if}
								{#if entry.thinkingOnly}
									<!-- Thinking that never reached a reply (stopped, or still on its way). -->
								{:else if cutOff === message.id && turn.end && phone}
									<!-- Errors & recovery, scenario 5 step 3 (UTrnr): what was written stays, then when it was cut off. -->
									<p class="text-phone leading-phone whitespace-pre-wrap wrap-anywhere text-text">{turn.reply.text}</p>
									<PhoneCutNotice seconds={Math.round(turn.end.elapsedMs / 1000)} oncontinue={continueReply} onrestart={entry.failure?.retry ? () => entry.failure && retryRun(message, entry.failure.text) : undefined} />
								{:else if cutOff === message.id && turn.end}
									<ReplyText text={turn.reply.text} status="interrupted" reason={turn.end.reason} oncontinue={continueReply} onrestart={() => entry.failure?.retry && retryRun(message, entry.failure.text)} />
								{:else if entry.failure && phone}
									{@const view = phoneFailure(message, entry.failure)}
									<PhoneErrorCard sentence={entry.failure.text} {...view} />
								{:else if entry.failure}
									{@const view = failureActions(message, entry.failure)}
									<FailedReply text={entry.failure.text} title={view.title} actions={view.actions} />
								{:else}
									{#if !entry.quiet}<Prose {phone} html={message.content_html} />{/if}
									{#each entry.cards as card (card.id)}
										<div data-message-id={card.id} class="contents">
											<ReplyBlocks
												blocks={replyBlocks(card, { language })}
												{api}
												messageId={card.id}
												agentName={nameOf(card)}
												{language}
												timeZone={time.timeZone}
												now={time.now}
												{working}
												latest={entry.latest}
												onsend={(text) => send(text)}
												{phone}
												{flows}
												onreload={() => void fetchMessage(card.id)}
												chatId={chatId}
												onanswer={() => field?.focus()}
											/>
										</div>
									{/each}
									<ReplyBlocks
										blocks={replyBlocks(message, { language })}
										{api}
										messageId={message.id}
										agentName={nameOf(message)}
										{language}
										timeZone={time.timeZone}
										now={time.now}
										{working}
										latest={entry.latest}
										onsend={(text) => send(text)}
										{phone}
										{flows}
										onreload={() => void fetchMessage(message.id)}
										chatId={chatId}
										onanswer={() => field?.focus()}
									/>
									{#if continued && message.created_at > continued.after}<TurnNote icon={Unplug} text={phone ? m.phone_net_continued() : m.continued_note({ elapsed: continued.elapsed })} />{/if}
									{#if phone}
										{@const feedback = message.id in feedbackShown ? feedbackShown[message.id] : message.my_feedback === 'up' ? 'liked' : message.my_feedback === 'down' ? 'disliked' : null}
										<ActionBar
											text={message.content}
											{feedback}
											onlike={() => void ratePhone(message, feedback === 'liked' ? null : 'liked')}
											ondislike={() => (feedback === 'disliked' ? void ratePhone(message, null) : (sheet = { kind: 'dislike', message }))}
											onshare={() => void share(message.id)}
											onmore={() => (sheet = { kind: 'reply', message, latest: entry.latest, asked })}
										/>
									{:else}
										<ReplyBar
											text={message.content}
											feedback={message.my_feedback === 'up' ? 'liked' : message.my_feedback === 'down' ? 'disliked' : null}
											onfeedback={(rating) => rate(message, rating)}
											ondislike={(feedback) => dislike(message, feedback)}
											onregenerate={entry.latest && !working ? (style) => regenerate(message.id, style) : undefined}
											onshare={() => share(message.id)}
										/>
									{/if}
									{#if entry.followUps.length > 0 && !working}<FollowUps items={entry.followUps} onpick={(item) => send(item)} />{/if}
								{/if}
							</AgentReply>
						</div>
					{/if}
				{/if}
			{/each}

			{#each pending as item (item.key)}
				{@const parts = splitAttachments(item.text)}
				{#if parts.files.length}<SentFiles {phone} files={parts.files.map((file) => ({ name: file.name, meta: fileMeta(file) }))} />{/if}
				{#if phone && item.status === 'queued'}
					<PhonePendingMessage text={parts.text} status="queued" note={m.phone_net_queued()} action={{ label: m.action_cancel(), onclick: () => cancelQueued(item) }} />
				{:else if phone && item.status === 'failed'}
					<!-- Not sent (TJhp2): the reason under the bubble with Retry; a long press puts it back to edit. -->
					<div use:longPress={() => editPending(item)}>
						<PhonePendingMessage text={parts.text} status="failed" note={m.phone_send_failed({ reason: item.reason ?? '' })} action={{ label: m.action_retry(), onclick: () => retryPending(item) }} />
					</div>
				{:else if phone}
					<PhoneUserMessage text={parts.text} />
				{:else}
					<MessageUser
						text={parts.text}
						status={item.status}
						oncancel={item.status === 'queued' ? () => cancelQueued(item) : undefined}
						onretry={item.status === 'failed' ? () => retryPending(item) : undefined}
						onedit={item.status === 'failed' ? () => editPending(item) : undefined}
					/>
				{/if}
				{#if item.reason && !phone}<TurnNote icon={Hourglass} align="end" tone="danger" text={item.reason} />{/if}
			{/each}

			{#if connectedNote}
				<!-- Back from Google's sign-in (Crew, memory & connections, step 6 mfedO): one line says it's
				     connected; the crew's answer follows as the chat carries on. -->
				<TurnNote kind="box" icon={Check} text={connectedNote} />
			{/if}
			{#if phone && flows.working.length > 0 && !working}
				<TurnNote kind="box" icon={Info} text={m.phone_crew_leave({ agent: flows.working.at(-1) ?? 'Nomi' })} />
			{/if}

			{#if (working || turn.end) && !cutOff && (turn.reply.text || turn.reply.thinking || working)}
				<AgentReply {phone} agent={crewIdOf(turn.agent)} name={turn.agent ?? 'Nomi'} meta={[clock(new Date(turn.startedAt ?? Date.now()), time.timeZone), turn.end?.status === 'stopped' ? m.meta_stopped() : undefined].filter(Boolean).join(' · ')}>
					{#if retrying && phone}
						<!-- Errors & recovery, scenario 2 step 1 (Obkjq): the wait counts down; another model is a tap away. -->
						<PhoneErrorCard
							title={m.failure_rate_limited()}
							meta={modelLabel}
							primary={{ label: retrying.seconds > 0 ? m.phone_retrying_in({ time: countdown(retrying.seconds) }) : m.phone_retrying_now(), disabled: true }}
							secondary={{ label: m.phone_action_another_model(), onclick: () => (sheet = { kind: 'model' }) }}
						/>
					{:else if retrying}
						<TurnNote kind="box" icon={Hourglass} text={m.retry_note({ attempt: retrying.attempt, max: retrying.max })} />
					{/if}
					{#if turn.reply.thinking}
						<Reasoning live={!turn.reply.text && working} label={turn.reply.text ? (turn.thoughtFor ? m.reasoning_thought_for({ seconds: Math.max(1, Math.round(turn.thoughtFor / 1000)) }) : m.reasoning_thinking()) : m.reasoning_thinking()} elapsed={elapsed} text={turn.reply.thinking} />
					{:else if !turn.reply.text && working}
						<LiveStep step={phaseLabel} {elapsed} />
					{/if}
					{#if turn.reply.text}
						<ReplyText
							text={turn.reply.text}
							status={turn.end?.status ?? 'streaming'}
							elapsed={turn.end ? elapsedLabel(turn.end.elapsedMs) : undefined}
							reason={turn.end?.reason}
							oncontinue={turn.end ? continueReply : undefined}
						/>
					{/if}
				</AgentReply>
			{/if}

			{@render after?.()}
		</div>
	</div>

	<div class={['relative', !phone && 'px-4 pb-4', !phone && !pane && 'lg:px-12 lg:pb-6']}>
		{#if !atLatest && entries.length > 0}
			<div class="absolute -top-12 left-1/2 -translate-x-1/2"><JumpToLatest count={unseen} onclick={() => jumpToLatest(true)} /></div>
		{/if}
		{#if phone}
			{#if quote}<div class="px-4 pb-2"><CodeQuote file={quote.file} range={quote.range} snippet={quote.snippet} onremove={() => onunquote?.()} /></div>{/if}
			<PhoneChatFlows {flows} {entries} chat={shownTitle ?? m.nav_new_chat()} {language} {working} onsend={(text) => void send(text)} onfocus={() => requestAnimationFrame(() => field?.focus())} />
			<PhoneComposer
				notice={flows.notice ?? undefined}
				bind:value={draft}
				bind:field
				placeholder={pane ? m.workspace_chat_placeholder() : m.mobile_message()}
				streaming={working}
				{crew}
				model={modelLabel}
				attached={uploads.uploaded.length > 0}
				blocked={blockedText}
				files={chips.length || removedLine ? composerFiles : undefined}
				fix={warning ? modelFix : undefined}
				onattach={() => picker?.click()}
				onsend={(text) => submit(text)}
				onstop={stop}
				onmodel={models.length ? () => (sheet = { kind: 'model' }) : undefined}
				oncrew={crew.length ? () => (sheet = { kind: 'crew' }) : undefined}
			/>
		{:else}
		<div class="mx-auto flex w-full max-w-180 flex-col gap-2">
			{#if quote}<CodeQuote file={quote.file} range={quote.range} snippet={quote.snippet} onremove={() => onunquote?.()} />{/if}
			<ChatComposer
				bind:value={draft}
				bind:field
				placeholder={pane ? m.workspace_chat_placeholder() : m.chat_placeholder()}
				busy={working}
				pickerAbove
				enterSends={page.data.chat?.enterSends ?? true}
				{crew}
				model={modelLabel}
				{models}
				thinking={thinkingChoice}
				attached={uploads.uploaded.length > 0}
				blocked={blockedText}
				describedBy={warning ? `${fixId}-fix` : undefined}
				files={chips.length || removedLine ? composerFiles : undefined}
				notice={warning ? modelFix : undefined}
				onattach={() => picker?.click()}
				onsend={(text) => submit(text)}
				onstop={stop}
				onmodel={(key) => void useModel(key)}
				onthinking={(level) => void useThinking(level)}
			/>
		</div>
		{/if}
	</div>
</div>
{#if viewer && !phone}{@render fileViewer(viewer)}{/if}
</div>

{#if phone}
	<!-- The phone's sheets: the chat's (from the nav bar's More), renaming it, a reply's More, the
	     composer's model and crew. -->
	{@const close = () => (sheet = null)}
	<!-- A sheet closing clears the state only while it's still the one showing (Rename follows the chat's). -->
	{@const closed = (kind: Sheet['kind']) => () => sheet?.kind === kind && (sheet = null)}
	<BottomSheet open={sheet?.kind === 'chat'} title={shownTitle ?? m.nav_new_chat()} subtitle={chatSubtitle} onclose={closed('chat')}>
		<SheetRow icon={Pencil} label={m.chat_rename()} onclick={() => ((newName = shownTitle ?? ''), (sheet = { kind: 'rename' }))} />
		<SheetRow icon={pinned ? PinOff : Pin} label={pinned ? m.chat_unpin_top() : m.chat_pin_top()} onclick={() => (close(), onMenu('pin'))} />
		{#if data.projects.length}<SheetRow icon={FolderInput} label={m.chat_move()} more onclick={() => (close(), onMenu('move'))} />{/if}
		<SheetRow icon={Download} label={m.chat_export()} meta="md" more onclick={() => (close(), onMenu('export'))} />
		<SheetRow icon={Share} label={m.chat_share_whole()} onclick={() => (close(), onMenu('share'))} />
		<span class="h-px bg-border"></span>
		<SheetRow icon={Trash2} label={m.chat_delete()} danger onclick={() => (close(), onMenu('delete'))} />
	</BottomSheet>
	<BottomSheet open={sheet?.kind === 'rename'} title={m.chat_rename()} onclose={closed('rename')}>
		<form
			class="flex flex-col gap-3 px-4 pt-1 pb-2"
			onsubmit={(event) => {
				event.preventDefault();
				close();
				void rename(newName);
			}}
		>
			<TextField label={m.chat_rename_field()} bind:value={newName} required />
			<Button type="submit" variant="inverse" label={m.chat_rename_save()} />
		</form>
	</BottomSheet>
	{#if sheet?.kind === 'reply'}
		{@const reply = sheet}
		{@const liked = (reply.message.id in feedbackShown ? feedbackShown[reply.message.id] : reply.message.my_feedback === 'up' ? 'liked' : reply.message.my_feedback === 'down' ? 'disliked' : null) === 'liked'}
		<!-- A reply's sheet (Like & dislike, step 1): the quick tiles, then what else it can do. -->
		<BottomSheet open title={m.reply_sheet_title()} subtitle={`${nameOf(reply.message)} · ${clock(reply.message.created_at, time.timeZone)}`} onclose={closed('reply')}>
			<SheetTiles
				tiles={[
					{ label: m.reply_good(), icon: ThumbsUp, pressed: liked, onclick: () => (void ratePhone(reply.message, liked ? null : 'liked'), close()) },
					{ label: m.reply_bad(), icon: ThumbsDown, onclick: () => (sheet = { kind: 'dislike', message: reply.message }) },
					{ label: m.reply_copy_short(), icon: Copy, onclick: () => (void copyText(reply.message.content), close()) },
					{ label: m.reply_share_short(), icon: Share, onclick: () => (void share(reply.message.id), close()) },
				]}
			/>
			{#if reply.latest && !working}
				<SheetRow icon={RefreshCw} label={m.reply_regenerate()} onclick={() => (sheet = { kind: 'regen', message: reply.message })} />
				{#if otherModels.length}
					<!-- Try another model (Wl0VM): opens Regenerate's sheet, whose last group is the other models. -->
					<SheetRow icon={Cpu} label={m.reply_try_model()} meta={otherModels[0].label} more onclick={() => (sheet = { kind: 'regen', message: reply.message })} />
				{/if}
				{#if reply.asked}
					{@const asked = reply.asked}
					<SheetRow icon={Pencil} label={m.reply_edit_question()} onclick={() => ((editing = asked.id), close())} />
				{/if}
			{/if}
			{#if !(reply.latest && working)}
				<!-- Delete reply (SwByh XPHtc): under its own line, in danger; asked first. -->
				<span class="h-px bg-border"></span>
				<SheetRow
					icon={Trash2}
					label={m.reply_delete()}
					danger
					onclick={() => {
						// Read before closing: the sheet (and its {@const}) is gone once it closes.
						const target = reply.message;
						close();
						void askDeleteReply(target);
					}}
				/>
			{/if}
		</BottomSheet>
	{/if}
	{#if sheet?.kind === 'regen'}
		{@const regen = sheet}
		<BottomSheet open title={m.reply_regenerate()} subtitle={m.regen_sheet_subtitle()} onclose={closed('regen')}>
			{#each [{ style: 'again', label: m.regen_again(), icon: RefreshCw }, { style: 'shorter', label: m.regen_shorter(), icon: Minimize2 }, { style: 'longer', label: m.regen_longer(), icon: Maximize2 }, { style: 'list', label: m.regen_list(), icon: List }] as const as option (option.style)}
				<SheetRow icon={option.icon} label={option.label} onclick={() => (void regenerate(regen.message.id, option.style), close())} />
			{/each}
			{#if otherModels.length}
				<p class="px-5 pt-3 pb-1 font-mono text-label tracking-label text-text-3 uppercase">{m.regen_models()}</p>
				{#each otherModels as option (option.key)}
					<!-- The model, and what pays for it in mono (lVDqs: claude-sonnet · your key, deepseek-flash ✦ Nomi credits). -->
					<SheetRow
						icon={option.source === m.models_nomi_credits() ? Sparkles : Cpu}
						label={option.label}
						meta={option.source === m.models_nomi_credits() ? option.source : m.regen_source_key()}
						onclick={() => (void regenerateWith(regen.message.id, option.key), close())}
					/>
				{/each}
			{/if}
		</BottomSheet>
	{/if}
	{#if sheet?.kind === 'dislike'}
		{@const disliked = sheet.message}
		<PhoneDislikeSheet
			open
			onsend={(feedback) => {
				// Read now: the sheet (and its {@const}) is gone by the time the answer comes.
				const target = disliked;
				feedbackShown = { ...feedbackShown, [target.id]: 'disliked' };
				void dislike(target, feedback).then((ok) => (ok ? toasts.show({ kind: 'success', title: m.feedback_thanks() }) : (feedbackShown = { ...feedbackShown, [target.id]: null })));
			}}
			onclose={closed('dislike')}
		/>
	{/if}
	{#if sheet?.kind === 'mine'}
		{@const mine = sheet}
		<!-- Your message's sheet (Message controls, step 3). -->
		<BottomSheet open title={m.mine_sheet_title()} subtitle={clock(mine.message.created_at, time.timeZone)} onclose={closed('mine')}>
			{#snippet lifted()}<PhoneUserMessage text={splitAttachments(mine.message.content).text} lifted />{/snippet}
			{#if mine.latest}<SheetRow icon={Pencil} label={m.mine_edit()} onclick={() => ((editing = mine.message.id), close())} />{/if}
			<SheetRow icon={Copy} label={m.mine_copy()} onclick={() => (void copyText(mine.message.content), close())} />
			<SheetRow icon={TextSelect} label={m.mine_select()} onclick={() => (selectText(mine.message.id), close())} />
		</BottomSheet>
	{/if}
	<!-- Opened from a failure (Errors & recovery, scenario 1 step 2: MLk2U), picking a model retries on it. -->
	{@const retry = sheet?.kind === 'model' ? sheet.retry : undefined}
	{@const credits = data.models.groups.find((group) => group.source === 'nomi')?.models.find((option) => option.enabled)}
	{@const pick = (key: string, onCredits: boolean) => {
		// Read before closing: the sheet (and its retry) is gone once it closes.
		const again = retry;
		close();
		void useModel(key).then((ok) => ok && again && retryRun(again.message, again.reason, onCredits));
	}}
	<BottomSheet open={sheet?.kind === 'model'} title={m.composer_model_title()} onclose={closed('model')}>
		{#each models as group (group.group)}
			<p class="px-5 pt-3 pb-1 font-mono text-label tracking-label text-text-3 uppercase">{group.group}</p>
			{#each group.models as option (option.key)}
				<SheetRow icon={option.chosen ? Check : option.source === m.models_nomi_credits() ? Sparkles : Cpu} label={option.label} meta={option.source} onclick={() => pick(option.key, option.source === m.models_nomi_credits())} />
			{/each}
		{/each}
		{#if retry && credits}
			<div class="px-4 pt-3"><PhoneButton label={m.phone_action_credits_retry()} onclick={() => pick(credits.key, true)} /></div>
		{/if}
	</BottomSheet>
	<BottomSheet open={sheet?.kind === 'crew'} title={m.composer_crew_title()} onclose={closed('crew')}>
		{#each crew as option (option.name)}
			{#if option.kind === 'crew'}
				<button
					type="button"
					class="flex h-13 w-full cursor-pointer items-center gap-3.5 px-5 text-left -outline-offset-2"
					onclick={() => {
						close();
						draft = `${draft.trim() ? `${draft.trimEnd()} ` : ''}@${option.name} `;
						requestAnimationFrame(() => field?.focus());
					}}
				>
					<Mascot agent={option.agent} size={20} />
					<span class="text-phone text-text">{option.name}</span>
					<span class="min-w-0 flex-1 truncate text-phone-sm text-text-3">{option.role}</span>
				</button>
			{/if}
		{/each}
	</BottomSheet>
{/if}

{#if shareLink && phone}
	<PhoneShareSheet
		bind:open={shareOpen}
		excerpt={shareLink.preview}
		chatMessages={shareLink.chat_messages}
		url={shareUrl}
		expiry={shareDays === null ? m.share_expiry_never() : shareDays === 1 ? m.share_expiry_day() : m.share_expiry_days({ days: String(shareDays) })}
		settings={{ scope: shareLink.scope, mask: shareLink.mask, anyone: shareLink.anyone }}
		onchange={(settings) => void saveShare(settings, shareDays)}
		onexpiry={() => shareLink && void saveShare({ scope: shareLink.scope, mask: shareLink.mask, anyone: shareLink.anyone }, EXPIRY[(EXPIRY.indexOf(shareDays) + 1) % EXPIRY.length])}
		oncopy={copyShare}
	/>
{:else if shareLink}
	<ShareDialog
		bind:open={shareOpen}
		preview={{ agent: crewIdOf(shareLink.preview_by), name: shareLink.preview_by ?? m.export_you(), time: clock(new Date(), time.timeZone), excerpt: shareLink.preview }}
		chatMessages={shareLink.chat_messages}
		url={shareUrl}
		expiry={shareDays === null ? m.share_expiry_never() : shareDays === 1 ? m.share_expiry_day() : m.share_expiry_days({ days: String(shareDays) })}
		settings={{ scope: shareLink.scope, mask: shareLink.mask, anyone: shareLink.anyone }}
		onchange={(settings) => void saveShare(settings, shareDays)}
		onexpiry={() => shareLink && void saveShare({ scope: shareLink.scope, mask: shareLink.mask, anyone: shareLink.anyone }, EXPIRY[(EXPIRY.indexOf(shareDays) + 1) % EXPIRY.length])}
		oncopy={copyShare}
		onclose={() => (shareOpen = false)}
	/>
{/if}

{#if phone}
	<PhoneMoveSheet bind:open={moveOpen} projects={data.projects} current={data.chat.inProjectId ?? undefined} onmove={move} />
{:else}
	<MoveDialog bind:open={moveOpen} projects={data.projects} initialChosen={data.chat.inProjectId ?? undefined} onmove={move} />
{/if}

{#if phone}
<!-- Delete reply's question (Share & delete, step 4 EQUZo). -->
<ActionSheet
	open={replyDelete !== null}
	title={m.delete_reply_title()}
	body={replyDelete?.body}
	actions={[{ label: m.reply_delete(), danger: true, onclick: () => replyDelete && deleteReplyLater(replyDelete.message) }]}
	cancel={m.action_cancel()}
	onclose={() => (replyDelete = null)}
/>
<ActionSheet
	bind:open={deleteOpen}
		title={m.chat_delete_title()}
		body={m.chat_delete_body_phone({ count: messages.length })}
		actions={[
			{ label: m.chat_delete_export(), onclick: exportChat },
			{ label: m.chat_delete(), danger: true, onclick: () => void deleteChat() },
		]}
		cancel={m.action_cancel()}
	/>
{:else}
<Dialog bind:open={deleteOpen} title={m.chat_delete_title()} description={m.chat_delete_body({ count: messages.length })}>
	{#snippet actions()}
		<Button label={m.chat_delete_export()} variant="ghost" iconStart={Download} onclick={exportChat} />
		<Button label={m.action_cancel()} variant="secondary" onclick={() => (deleteOpen = false)} />
		<Button label={m.chat_delete()} variant="danger" iconStart={Trash2} onclick={deleteChat} />
	{/snippet}
</Dialog>
{/if}

<!-- Files: the picker, what's said to a screen reader as uploads finish or fail, a phone's sheets
     (a failed file's Retry or Remove, the models that can see a picture), and the viewers. -->
<input bind:this={picker} type="file" multiple hidden onchange={pickFiles} />
<p class="sr-only" aria-live="polite">{liveText}</p>
{#if alertText}<p role="alert" class="sr-only">{alertText}</p>{/if}

{#if phone && fileSheet}
	{@const failed = fileSheet}
	<BottomSheet open title={failed.name} onclose={() => (fileSheet = null)}>
		{#snippet header(close)}<FileSheetHeader name={failed.name} reason={chipDetail(failed)} onclose={close} />{/snippet}
		<SheetRow
			icon={RotateCw}
			label={m.files_retry()}
			meta={m.files_sheet_retry_meta()}
			onclick={() => {
				uploads.retry(failed.key);
				fileSheet = null;
			}}
		/>
		<SheetRow
			icon={X}
			danger
			label={m.files_remove()}
			meta={m.files_sheet_remove_meta()}
			onclick={() => {
				void removeChip(failed);
				fileSheet = null;
			}}
		/>
	</BottomSheet>
{/if}

{#if phone && visionOpen}
	<VisionModelMenu phone groups={vision.groups} hidden={vision.hidden} settingsHref="/settings/models" onpick={(key) => void pickVisionModel(key)} ontextonly={sendTextOnly} onclose={closeVision} />
{/if}

{#if viewer && phone}{@render fileViewer(viewer)}{/if}

{#if photo}
	{@const shown = photo}
	<PhotoViewer
		name={shown.file.name}
		meta={[fileMeta(shown.file), phone ? m.files_yours_phone() : m.files_yours(), phone ? null : m.files_sent_at({ time: shown.at })].filter(Boolean).join(' · ')}
		src="/attachments/{shown.file.id}/content"
		where={phone
			? shown.count === 1
				? m.files_photo_where_phone_one()
				: m.files_photo_where_phone({ index: String(shown.index), count: String(shown.count) })
			: shown.count === 1
				? m.files_photo_where_one()
				: m.files_photo_where({ index: String(shown.index), count: String(shown.count) })}
		{phone}
		downloadHref="/attachments/{shown.file.id}/content?download"
		onclose={() => void closeViewer()}
	/>
{/if}
