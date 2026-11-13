<script lang="ts">
	import { AlarmClock, AlarmClockPlus, ArrowRight, Calendar, Check, CircleCheck, Clock, Ellipsis, ExternalLink, Inbox, MessageSquare, Repeat, X } from '@lucide/svelte';
	import { goto, invalidate } from '$app/navigation';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { clock } from '$lib/clock';
	import { later } from '$lib/chat/later';
	import { clock as clockTime, dayKey } from '$lib/chat/dates';
	import { agenda, dayName, entryCaption, entryCrew, entryTag, entryTime, filterCounts, inboxEntries, inboxFeed, type Filter, type InboxEntry } from '$lib/inbox/feed';
	import { dueAt, localNow, parseReminder } from '$lib/inbox/quickReminder';
	import { QUICK_REMINDER, inboxAction } from '$lib/inbox/actions';
	import type { EmailPreferences } from '$lib/adapters/api';
	import { toasts } from '$lib/ds/overlays/toasts.svelte';
	import type { MenuEntry } from '$lib/ds/overlays/MenuPanel.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import IconButton from '$lib/ds/actions/IconButton.svelte';
	import Menu from '$lib/ds/overlays/Menu.svelte';
	import Segmented from '$lib/ds/inputs/Segmented.svelte';
	import SectionLabel from '$lib/ds/status/SectionLabel.svelte';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';
	import InboxHeading from '$lib/ds/inbox/InboxHeading.svelte';
	import InboxCard from '$lib/ds/inbox/InboxCard.svelte';
	import InboxRow from '$lib/ds/inbox/InboxRow.svelte';
	import NowLine from '$lib/ds/inbox/NowLine.svelte';
	import EarlierToggle from '$lib/ds/inbox/EarlierToggle.svelte';
	import QuickReminder from '$lib/ds/inbox/QuickReminder.svelte';
	import Agenda from '$lib/ds/inbox/Agenda.svelte';
	import PreferenceRow from '$lib/ds/inbox/PreferenceRow.svelte';

	// The Inbox (App / Inbox BFoiZ): the feed (padded [24,32,20,32], its sections 18 apart) beside
	// the Coming up column (340, its left stroke), which goes under the feed on narrower screens.
	// The top bar (its Mark all read and New reminder) is the layout's. Cancelling a reminder or
	// scheduled work, and ticking one off before it's due, wait ten seconds for an undo, as deleting
	// a chat does; leaving the app sends them at once (the layout flushes them).
	// Left out (AGENTS.md §9): the pen's snooze on reminders still ahead (the backend's snooze moves
	// one to N minutes from now, which would bring it forward), and its Push for reminders and Quiet
	// hours settings (the backend has neither: BACKEND_TODO.md).
	let { data } = $props();

	const time = clock();
	const language = getLocale();
	const zone = $derived(data.reminders.timezone);
	const key = (entry: InboxEntry) => `${entry.kind}:${entry.id}`;

	let filter = $state<Filter>('all');
	let expanded = $state(false);
	let leaving = $state<string[]>([]);
	let busy = $state<string>();

	const entries = $derived(inboxEntries(data.notifications, data.reminders).filter((entry) => !leaving.includes(key(entry))));
	const counts = $derived(filterCounts(entries, data.counts));
	const feed = $derived(inboxFeed(entries, filter, time.now, zone));
	const coming = $derived(agenda(entries, time.now, zone));
	const today = $derived(dayKey(time.now, zone));
	const more = $derived(data.shown < data.pages);
	const nothing = $derived(!feed.needs.length && !feed.today.length && !feed.later.length && !feed.earlier.length && !more);
	const filters = $derived([
		{ value: 'all', label: m.inbox_filter_all(), count: counts.all },
		{ value: 'needs', label: m.inbox_filter_needs(), count: counts.needs },
		{ value: 'reminders', label: m.inbox_filter_reminders(), count: counts.reminders },
		{ value: 'updates', label: m.inbox_filter_updates(), count: counts.updates },
	]);
	const empty = $derived(
		{
			all: { icon: Inbox, title: m.inbox_empty_all_title(), body: m.inbox_empty_all_body() },
			needs: { icon: CircleCheck, title: m.inbox_empty_needs_title(), body: m.inbox_empty_needs_body() },
			reminders: { icon: AlarmClock, title: m.inbox_empty_reminders_title(), body: m.inbox_empty_reminders_body() },
			updates: { icon: Inbox, title: m.inbox_empty_updates_title(), body: m.inbox_empty_updates_body() },
		}[filter],
	);

	/** Runs an action on an entry; notifications change the sidebar's count, so the shell reloads. */
	async function run(entry: InboxEntry, name: string, fields: Record<string, string>, done?: string, keepalive = false): Promise<boolean> {
		busy = key(entry);
		const result = await inboxAction(name, fields, { keepalive });
		busy = undefined;
		if (!result.ok) {
			if (result.message) toasts.show({ kind: 'error', title: m.inbox_action_failed(), detail: result.message });
			return false;
		}
		if (done) toasts.show({ kind: 'success', title: done });
		await invalidate(entry.kind === 'notification' ? 'app:shell' : 'app:inbox');
		return true;
	}

	/** Takes it out of the list now and sends it in ten seconds, unless the person undoes it. */
	function withUndo(entry: InboxEntry, title: string, send: () => Promise<boolean>) {
		const id = key(entry);
		leaving = [...leaving, id];
		const back = () => (leaving = leaving.filter((one) => one !== id));
		later.schedule(id, () => void send().finally(back), 10_000);
		toasts.show({ kind: 'info', title, duration: 10_000, action: { label: m.action_undo(), onclick: () => later.undo(id) && back() } });
	}

	const inApp = (link: string | null): link is string => !!link && link.startsWith('/') && !link.startsWith('//');

	async function open(entry: Extract<InboxEntry, { kind: 'notification' }>) {
		if (!entry.read) await run(entry, 'read', { id: entry.id });
		if (inApp(entry.link)) await goto(entry.link);
	}

	const finish = (entry: InboxEntry) => run(entry, 'reminder', { id: entry.id, do: 'done' }, m.inbox_marked_done());
	const snooze = (entry: InboxEntry) => run(entry, 'reminder', { id: entry.id, do: 'snooze', minutes: '60' }, m.inbox_snoozed());
	const finishEarly = (entry: InboxEntry) => withUndo(entry, m.inbox_marked_done(), () => run(entry, 'reminder', { id: entry.id, do: 'done' }, undefined, true));

	function menu(entry: Extract<InboxEntry, { kind: 'reminder' | 'task' }>): MenuEntry[] {
		const chat = { label: m.inbox_open_chat(), icon: MessageSquare, onselect: () => void goto(`/chat/${entry.sessionId}`) };
		if (entry.kind === 'task') {
			return [chat, { label: m.inbox_cancel_task(), icon: X, danger: true, onselect: () => withUndo(entry, m.inbox_task_cancelled(), () => run(entry, 'cancelTask', { id: entry.id }, undefined, true)) }];
		}
		const cancel = () => withUndo(entry, m.inbox_reminder_cancelled(), () => run(entry, 'reminder', { id: entry.id, do: 'cancel' }, undefined, true));
		return [chat, { label: entry.repeat ? m.inbox_stop_reminder() : m.inbox_cancel_reminder(), icon: X, danger: true, onselect: cancel }];
	}

	// The quick reminder: read as it's typed, in the zone the backend reads it in.
	let line = $state('');
	let setting = $state(false);
	let failed = $state<{ line: string; message: string }>();
	const parsed = $derived(parseReminder(line, localNow(time.now, zone)));
	const quickError = $derived(failed?.line === line ? failed.message : undefined);
	const REPEATS = { daily: m.inbox_repeat_daily, weekly: m.inbox_repeat_weekly, monthly: m.inbox_repeat_monthly };
	const chips = $derived([
		...(parsed.date ? [{ icon: Calendar, label: dayName(parsed.date, language) }] : []),
		...(parsed.time ? [{ icon: Clock, label: parsed.time }] : []),
		...(parsed.repeat ? [{ icon: Repeat, label: REPEATS[parsed.repeat]() }] : []),
	]);
	const hint = $derived(setting ? m.inbox_quick_saving() : line.trim() && !parsed.time ? m.inbox_quick_needs_time() : m.inbox_quick_add());

	async function remind() {
		const reminder = parsed;
		const due = dueAt(reminder);
		if (!reminder.title) return void (failed = { line, message: m.inbox_remind_what() });
		if (!due) return void (failed = { line, message: m.inbox_remind_when() });
		setting = true;
		const result = await inboxAction('remind', { title: reminder.title, due_at: due, repeat: reminder.repeat ?? '' });
		setting = false;
		if (!result.ok) return void (failed = { line, message: result.message || m.inbox_action_failed() });
		line = '';
		toasts.show({ kind: 'success', title: m.inbox_reminder_set({ when: `${dayName(reminder.date!, language)} · ${reminder.time}` }) });
		await invalidate('app:inbox');
	}

	// Which emails: switched at once, switched back if saving fails.
	let emails = $derived<EmailPreferences | null>(data.preferences);

	async function saveEmails(change: Partial<EmailPreferences>) {
		if (!emails) return;
		const before = emails;
		emails = { ...emails, ...change };
		const result = await inboxAction('preferences', { email_account: String(emails.email_account), email_promos: String(emails.email_promos) });
		if (result.ok) return;
		emails = before;
		if (result.message) toasts.show({ kind: 'error', title: m.inbox_action_failed(), detail: result.message });
	}
</script>

<svelte:head><title>{m.nav_inbox()} · {m.app_name()}</title></svelte:head>

{#snippet row(entry: InboxEntry)}
	{@const done = entry.kind === 'reminder' && (entry.status === 'done' || entry.status === 'cancelled')}
	{@const ahead = !done && Date.parse(entry.at) > time.now.getTime()}
	<InboxRow
		data-entry={entry.id}
		time={entryTime(entry, time.now, zone, language)}
		agent={entryCrew(entry)}
		title={entry.title}
		tag={entryTag(entry, language)}
		caption={entryCaption(entry, time.now) || undefined}
		upcoming={ahead}
		finished={done}
	>
		{#snippet tools()}
			{#if entry.kind === 'notification' && inApp(entry.link)}
				<IconButton icon={ExternalLink} label={m.inbox_open()} variant="ghost" iconSize={14} onclick={() => open(entry)} />
			{:else if entry.kind === 'task' || (entry.kind === 'reminder' && entry.status === 'active')}
				{#if entry.kind === 'reminder' && !entry.repeat}
					<IconButton icon={Check} label={m.inbox_mark_done()} variant="ghost" iconSize={14} disabled={busy === key(entry)} onclick={() => finishEarly(entry)} />
				{/if}
				<Menu items={menu(entry)} align="end">
					{#snippet trigger(props)}<IconButton icon={Ellipsis} label={m.inbox_more({ title: entry.title })} variant="ghost" iconSize={14} {...props} />{/snippet}
				</Menu>
			{/if}
		{/snippet}
	</InboxRow>
{/snippet}

<div class="flex h-full flex-col overflow-y-auto xl:flex-row xl:overflow-hidden">
	<div class="flex min-w-0 flex-1 flex-col gap-4.5 px-4 pt-6 pb-5 lg:px-8 xl:overflow-y-auto">
		<header class="flex flex-wrap items-end gap-x-6 gap-y-3">
			<div class="flex min-w-0 flex-1 basis-80 flex-col gap-1.5">
				<h1 class="text-display font-medium tracking-display text-text">{m.nav_inbox()}</h1>
				<p class="text-body text-text-2">{m.inbox_intro()}</p>
			</div>
			<!-- On the header's bottom line, as drawn (Segmented keeps itself to the start otherwise). -->
			<div class="shrink-0"><Segmented label={m.inbox_filter_label()} options={filters} value={filter} onchange={(value) => (filter = value as Filter)} /></div>
		</header>

		{#if feed.needs.length}
			<section>
				<InboxHeading label={m.inbox_filter_needs()} count={feed.needs.length} />
				<div class="flex flex-col gap-2.5">
					{#each feed.needs as entry (key(entry))}
						<InboxCard
							time={entryTime(entry, time.now, zone, language)}
							agent={entryCrew(entry)}
							title={entry.title}
							tag={entryTag(entry, language)}
							caption={entryCaption(entry, time.now) || undefined}
							unread={entry.kind === 'notification'}
						>
							{#snippet actions()}
								{#if entry.kind === 'notification'}
									{#if inApp(entry.link)}<Button label={m.inbox_open()} variant="inverse" iconEnd={ArrowRight} loading={busy === key(entry)} onclick={() => open(entry)} />{/if}
									<Button label={m.inbox_mark_read()} variant="ghost" iconStart={Check} disabled={busy === key(entry)} onclick={() => run(entry, 'read', { id: entry.id })} />
								{:else}
									<Button label={m.inbox_done()} variant="inverse" iconStart={Check} loading={busy === key(entry)} onclick={() => finish(entry)} />
									<Button label={m.inbox_snooze_hour()} variant="ghost" iconStart={AlarmClockPlus} disabled={busy === key(entry)} onclick={() => snooze(entry)} />
								{/if}
							{/snippet}
						</InboxCard>
					{/each}
				</div>
			</section>
		{/if}

		{#if feed.today.length}
			<section>
				<InboxHeading label={m.inbox_today({ date: dayName(today, language) })} />
				<ul class="flex flex-col border-t border-border">
					{#each feed.today as entry, index (key(entry))}
						{#if index === feed.nowAt}<li><NowLine time={clockTime(time.now, zone)} label={m.inbox_now()} /></li>{/if}
						{@render row(entry)}
					{/each}
					{#if feed.nowAt === feed.today.length}<li><NowLine time={clockTime(time.now, zone)} label={m.inbox_now()} /></li>{/if}
				</ul>
			</section>
		{/if}

		{#each feed.later as day (day.day)}
			<section>
				<InboxHeading label={dayName(day.day, language)} />
				<ul class="flex flex-col border-t border-border">
					{#each day.entries as entry (key(entry))}{@render row(entry)}{/each}
				</ul>
			</section>
		{/each}

		{#if feed.earlier.length || more}
			<section class="flex flex-col gap-2">
				<EarlierToggle label={m.inbox_earlier()} count={feed.earlier.length} bind:expanded />
				{#if expanded}
					{#if feed.earlier.length}
						<ul class="flex flex-col border-t border-border">
							{#each feed.earlier as entry (key(entry))}{@render row(entry)}{/each}
						</ul>
					{/if}
					{#if more}
						<span><Button label={m.inbox_show_older()} variant="ghost" onclick={() => goto(`?shown=${data.shown + 1}`, { noScroll: true, keepFocus: true })} /></span>
					{/if}
				{/if}
			</section>
		{/if}

		{#if nothing}<EmptyState icon={empty.icon} title={empty.title} body={empty.body} />{/if}
	</div>

	<aside class="flex shrink-0 flex-col border-t border-border bg-bg xl:w-85 xl:overflow-y-auto xl:border-t-0 xl:border-l">
		<div class="border-b border-border px-4 pt-4 pb-3.75 xl:pr-5 xl:pl-4.75">
			<QuickReminder id={QUICK_REMINDER} label={m.inbox_quick()} placeholder={m.inbox_quick_placeholder()} bind:value={line} {chips} {hint} error={quickError} busy={setting} onsubmit={remind} />
		</div>
		<div class="border-b border-border px-4 pt-4 pb-3.75 xl:pr-5 xl:pl-4.75">
			<Agenda
				label={m.inbox_next_days()}
				count={coming.reduce((sum, day) => sum + day.entries.length, 0)}
				empty={m.inbox_agenda_empty()}
				days={coming.map((day) => ({
					label: day.day === today ? m.inbox_agenda_today() : dayName(day.day, language, false),
					today: day.day === today,
					items: day.entries.map((entry) => ({ agent: entryCrew(entry), title: entry.title, time: clockTime(entry.at, zone) })),
				}))}
			/>
		</div>
		<section class="flex flex-col gap-3 px-4 py-4 xl:pr-5 xl:pl-4.75">
			<SectionLabel label={m.inbox_delivery()} />
			{#if emails}
				<PreferenceRow title={m.inbox_email_account()} description={m.inbox_email_account_body()} checked={emails.email_account} onchange={(on) => saveEmails({ email_account: on })} />
				<PreferenceRow title={m.inbox_email_promos()} description={m.inbox_email_promos_body()} checked={emails.email_promos} onchange={(on) => saveEmails({ email_promos: on })} />
			{:else}
				<p class="text-caption text-text-3">{m.inbox_preferences_failed()}</p>
			{/if}
		</section>
	</aside>
</div>
