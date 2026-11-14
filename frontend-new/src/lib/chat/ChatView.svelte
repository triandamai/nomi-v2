<script lang="ts">
	import { onMount, tick, untrack } from 'svelte';
	import { ArrowUpRight, Cpu, Download, FilePlus2, Hourglass, KeyRound, LogIn, RotateCcw, Sparkles, Trash2, Unplug, Undo2 } from '@lucide/svelte';
	import { goto, invalidate } from '$app/navigation';
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import type { MessageItem, MessageVersion, RenderedMessage, RerunResult, ThinkingLevel } from '$lib/adapters/api';
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
	import { applyChatEvent, idleTurn, isWorking, retryInfo, turnSent, turnStopped } from '$lib/chat/turn';
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
	import ShareDialog from '$lib/ds/chat/ShareDialog.svelte';
	import Dialog from '$lib/ds/overlays/Dialog.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import AgentReply from '$lib/ds/thread/AgentReply.svelte';
	import Prose from '$lib/ds/thread/Prose.svelte';
	import ReplyBar, { type Dislike, type RegenerateStyle } from '$lib/ds/thread/ReplyBar.svelte';
	import FailedReply from '$lib/ds/thread/FailedReply.svelte';
	import TurnNote from '$lib/ds/thread/TurnNote.svelte';
	import ChatComposer from '$lib/ds/thread/ChatComposer.svelte';
	import { replyBlocks } from '$lib/adapters/blocks';
	import ReplyBlocks from '$lib/chat/ReplyBlocks.svelte';
	import CodeQuote from '$lib/ds/build/CodeQuote.svelte';
	import type { ChatPageData } from '$lib/server/chatPage';
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
	};
	let { data, pane = false, quote = null, onunquote }: Props = $props();

	const header = chatHeader();
	const time = appClock();
	const language = getLocale();
	const NIL = '00000000-0000-0000-0000-000000000000';

	type Pending = { key: number; text: string; status: 'sending' | 'queued' | 'failed'; reason?: string };

	const chatId = $derived(data.chat.id);
	const api = $derived(`/chat/${data.chat.id}/api`);
	const outboxKey = $derived(`/chat/${data.chat.id}`);

	let messages = $state<RenderedMessage[]>(untrack(() => data.messages));
	let hasOlder = $state(untrack(() => data.hasOlder));
	let turn = $state(idleTurn());
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
	let recovered = $state<{ after: string; reason: string } | null>(null);
	/** The reply continued after the stream was cut off, and when it was. */
	let continued = $state<{ after: string; elapsed: string } | null>(null);
	/** The stop exchange sent from here: the stopped reply says it, so its messages wait for a reload. */
	const hidden = new Set<string>();
	let online = $state(true);
	let backOnline = $state(false);
	let modelKey = $state<string | null>(untrack(() => data.model?.key ?? data.models.current?.key ?? null));
	let thinking = $state<ThinkingLevel | null>(untrack(() => data.thinking));
	let moveOpen = $state(false);
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
				turn = idleTurn();
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

	function onEvent(event: ChatEvent) {
		const before = turn;
		turn = applyChatEvent(turn, event, Date.now());
		switch (event.kind) {
			case 'TurnCompleted':
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
				messages = messages.filter((message) => !gone.has(message.id));
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
					void invalidate('chat:page');
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
	async function send(text: string, options: { late?: boolean } = {}) {
		const local: Pending = { key: nextKey++, text, status: 'sending' };
		pending = [...pending, local];
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
			if (options.late) lateNotes = { ...lateNotes, [result.data.message.id]: m.sent_after_signin({ time: clock(new Date(), time.timeZone) }) };
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
		draft = item.text;
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

	async function retryRun(message: RenderedMessage, reason: string) {
		const result = await callJson(`${api}/messages/${message.id}/retry`, { method: 'POST' });
		if (!result.ok) return void toasts.show({ kind: 'error', title: m.retry_failed(), detail: result.message || undefined });
		messages = messages.filter((one) => one.id !== message.id);
		recovered = { after: new Date().toISOString(), reason };
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
	const pinned = $derived(header.shown(chatId, 'pinned', data.chat.pinned));

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
			title: m.chat_moved({ project: project?.name ?? '' }),
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
		if (event.defaultPrevented || document.querySelector('dialog[open], [role="menu"], [role="listbox"]')) return;
		if (event.key === 'Escape' && working) {
			event.preventDefault();
			void stop();
		} else if (event.key === 'F2') {
			event.preventDefault();
			header.renaming = true;
		}
	}

	onMount(() => {
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
		if (!matchMedia('(pointer: coarse)').matches) field?.focus();
		jumpToLatest();
		return () => {
			clearTimeout(hide);
			window.removeEventListener('offline', offline);
			window.removeEventListener('online', back);
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
</script>

<svelte:window {onkeydown} />


{#snippet userMessage(message: RenderedMessage, latest: boolean)}
	{@const total = (message.versions ?? 0) + 1}
	{@const shown = viewing[message.id] ? versions[message.id]?.[viewing[message.id] - 1] : undefined}
	{#if editing === message.id}
		<MessageEditor value={message.content} version="{total} / {total + 1}" onsave={(text) => saveEdit(message, text)} oncancel={() => (editing = null)} />
	{:else}
		<MessageUser
			text={shown?.text ?? message.content}
			time={clock(message.edited_at ?? message.created_at, time.timeZone)}
			edited={Boolean(message.edited_at) && !shown}
			version={total > 1 && message.edited_at ? { current: viewing[message.id] || total, total, onprev: () => step(message, (viewing[message.id] || total) - 1), onnext: () => step(message, (viewing[message.id] || total) + 1) } : undefined}
			onedit={latest && !working && !shown ? () => (editing = message.id) : undefined}
		/>
		{#if lateNotes[message.id]}<TurnNote icon={LogIn} align="end" text={lateNotes[message.id]} />{/if}
	{/if}
{/snippet}

<div class="flex h-full flex-col">
	<div bind:this={scroller} {onscroll} class={['min-h-0 flex-1 overflow-y-auto', pane ? 'px-5 pt-5' : 'px-4 pt-6 lg:px-12 lg:pt-10']}>
		<div class="mx-auto flex w-full max-w-180 flex-col gap-8 pb-8">
			{#if !online}
				<Banner kind="offline" text={m.banner_offline()} action={{ label: m.banner_retry_now(), onclick: () => void flushOutbox() }} />
			{:else if backOnline}
				<Banner kind="online" text={pending.length === 0 ? m.banner_online_sent() : m.banner_online()} />
			{:else if quota !== null}
				<Banner kind="quota" text={m.banner_quota({ percent: quota })} action={{ label: m.banner_see_usage(), href: '/settings/plan' }} />
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
					<DateDivider label={entry.label} />
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
					{:else}
						<div data-message-id={message.id} class="flex flex-col">
							<AgentReply
								agent={crewIdOf(message.agent_display_name)}
								name={nameOf(message)}
								meta={replyMeta(message, entry.turnStart, cutOff === message.id ? m.meta_stopped() : continued && message.created_at > continued.after ? m.meta_continued() : undefined)}
								version={asked && takes > 1 && !asked.edited_at && firstOfTurn ? { current: takes, total: takes, onprev: () => step(asked, takes - 1), onnext: () => {} } : undefined}
								dimmed={editing !== null && asked?.id === editing}
							>
								{#if recovered && firstOfTurn && message.created_at > recovered.after}
									<TurnNote kind="box" icon={RotateCcw} text={m.recovered_note({ reason: recovered.reason })} />
								{/if}
								{#if entry.reasoning.length > 0}
									<Reasoning label={reasoningLabel(message.id)} text={entry.reasoning.join('\n\n')} />
								{/if}
								{#if entry.thinkingOnly}
									<!-- Thinking that never reached a reply (stopped, or still on its way). -->
								{:else if cutOff === message.id && turn.end}
									<ReplyText text={turn.reply.text} status="interrupted" reason={turn.end.reason} oncontinue={continueReply} onrestart={() => entry.failure?.retry && retryRun(message, entry.failure.text)} />
								{:else if entry.failure}
									{@const view = failureActions(message, entry.failure)}
									<FailedReply text={entry.failure.text} title={view.title} actions={view.actions} />
								{:else}
									<Prose html={message.content_html} />
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
									/>
									{#if continued && message.created_at > continued.after}<TurnNote icon={Unplug} text={m.continued_note({ elapsed: continued.elapsed })} />{/if}
									<ReplyBar
										text={message.content}
										feedback={message.my_feedback === 'up' ? 'liked' : message.my_feedback === 'down' ? 'disliked' : null}
										onfeedback={(rating) => rate(message, rating)}
										ondislike={(feedback) => dislike(message, feedback)}
										onregenerate={entry.latest && !working ? (style) => regenerate(message.id, style) : undefined}
										onshare={() => share(message.id)}
									/>
									{#if entry.followUps.length > 0 && !working}<FollowUps items={entry.followUps} onpick={(item) => send(item)} />{/if}
								{/if}
							</AgentReply>
						</div>
					{/if}
				{/if}
			{/each}

			{#each pending as item (item.key)}
				<MessageUser
					text={item.text}
					status={item.status}
					oncancel={item.status === 'queued' ? () => cancelQueued(item) : undefined}
					onretry={item.status === 'failed' ? () => retryPending(item) : undefined}
					onedit={item.status === 'failed' ? () => editPending(item) : undefined}
				/>
				{#if item.reason}<TurnNote icon={Hourglass} align="end" tone="danger" text={item.reason} />{/if}
			{/each}

			{#if (working || turn.end) && !cutOff && (turn.reply.text || turn.reply.thinking || working)}
				<AgentReply agent={crewIdOf(turn.agent)} name={turn.agent ?? 'Nomi'} meta={[clock(new Date(turn.startedAt ?? Date.now()), time.timeZone), turn.end?.status === 'stopped' ? m.meta_stopped() : undefined].filter(Boolean).join(' · ')}>
					{#if retrying}
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
		</div>
	</div>

	<div class={['relative px-4 pb-4', !pane && 'lg:px-12 lg:pb-6']}>
		{#if !atLatest && entries.length > 0}
			<div class="absolute -top-12 left-1/2 -translate-x-1/2"><JumpToLatest count={unseen} onclick={() => jumpToLatest(true)} /></div>
		{/if}
		<div class="mx-auto flex w-full max-w-180 flex-col gap-2">
			{#if quote}<CodeQuote file={quote.file} range={quote.range} snippet={quote.snippet} onremove={() => onunquote?.()} />{/if}
			<ChatComposer
				bind:value={draft}
				bind:field
				placeholder={pane ? m.workspace_chat_placeholder() : m.chat_placeholder()}
				busy={working}
				pickerAbove
				{crew}
				model={modelLabel}
				{models}
				thinking={thinkingChoice}
				onsend={(text) => {
					const quoted = quote;
					if (quoted) onunquote?.();
					void send(quoted ? `${quoted.text}\n\n${text}` : text);
				}}
				onstop={stop}
				onmodel={(key) => void useModel(key)}
				onthinking={(level) => void useThinking(level)}
			/>
		</div>
	</div>
</div>

{#if shareLink}
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

<MoveDialog bind:open={moveOpen} projects={data.projects} initialChosen={data.chat.inProjectId ?? undefined} onmove={move} />

<Dialog bind:open={deleteOpen} title={m.chat_delete_title()} description={m.chat_delete_body({ count: messages.length })}>
	{#snippet actions()}
		<Button label={m.chat_delete_export()} variant="ghost" iconStart={Download} onclick={exportChat} />
		<Button label={m.action_cancel()} variant="secondary" onclick={() => (deleteOpen = false)} />
		<Button label={m.chat_delete()} variant="danger" iconStart={Trash2} onclick={deleteChat} />
	{/snippet}
</Dialog>
