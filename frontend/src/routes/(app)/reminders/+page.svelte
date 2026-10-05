<script lang="ts">
	import { enhance } from '$app/forms';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import ButtonGroup from '$lib/components/m3/ButtonGroup.svelte';
	import Checkbox from '$lib/components/m3/Checkbox.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import List from '$lib/components/m3/List.svelte';
	import ListItem from '$lib/components/m3/ListItem.svelte';
	import Menu from '$lib/components/m3/Menu.svelte';
	import MenuItem from '$lib/components/m3/MenuItem.svelte';
	import Snackbar from '$lib/components/m3/Snackbar.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import IconClose from '$lib/components/icons/IconClose.svelte';
	import IconPlus from '$lib/components/icons/IconPlus.svelte';
	import type { Reminder } from '$lib/types';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	const reminders = $derived(data.reminders);
	const timezone = $derived(reminders?.timezone ?? 'UTC');

	type Repeat = 'once' | 'daily' | 'weekly' | 'monthly';
	const REPEATS: { value: Repeat; label: string }[] = [
		{ value: 'once', label: 'Once' },
		{ value: 'daily', label: 'Daily' },
		{ value: 'weekly', label: 'Weekly' },
		{ value: 'monthly', label: 'Monthly' },
	];
	const SNOOZES = [
		{ minutes: 10, label: '10 minutes' },
		{ minutes: 60, label: '1 hour' },
		{ minutes: 24 * 60, label: 'Tomorrow, same time' },
	];
	const WEEKDAYS = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];

	let sheetOpen = $state(false);
	let title = $state('');
	let notes = $state('');
	let when = $state(defaultWhen());
	let repeat = $state<Repeat>('once');
	let saving = $state(false);
	let snackbar = $state(false);
	let snackbarMessage = $state('');
	// Which row's snooze menu is open.
	let menus = $state<Record<string, boolean>>({});

	function defaultWhen(): string {
		const d = new Date(Date.now() + 60 * 60 * 1000);
		d.setMinutes(0, 0, 0);
		const pad = (n: number) => String(n).padStart(2, '0');
		return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
	}

	// The picker's local time with this browser's offset, so weekly/monthly keep the picked day.
	const dueAt = $derived.by(() => {
		if (!when) return '';
		const date = new Date(when);
		if (Number.isNaN(date.getTime())) return '';
		const offset = -date.getTimezoneOffset();
		const sign = offset >= 0 ? '+' : '-';
		const pad = (n: number) => String(Math.floor(Math.abs(n))).padStart(2, '0');
		return `${when}:00${sign}${pad(offset / 60)}:${pad(offset % 60)}`;
	});

	function openSheet() {
		title = '';
		notes = '';
		when = defaultWhen();
		repeat = 'once';
		sheetOpen = true;
	}

	function dayKey(iso: string): string {
		return new Intl.DateTimeFormat('en-CA', { timeZone: timezone, year: 'numeric', month: '2-digit', day: '2-digit' }).format(new Date(iso));
	}
	function dayHeading(iso: string): string {
		const key = dayKey(iso);
		if (key === dayKey(new Date().toISOString())) return 'Today';
		if (key === dayKey(new Date(Date.now() + 86_400_000).toISOString())) return 'Tomorrow';
		return new Intl.DateTimeFormat(undefined, { weekday: 'long', day: 'numeric', month: 'long', timeZone: timezone }).format(new Date(iso));
	}
	function time(iso: string): string {
		return new Intl.DateTimeFormat(undefined, { hour: '2-digit', minute: '2-digit', hourCycle: 'h23', timeZone: timezone }).format(new Date(iso));
	}
	function shortWhen(iso: string): string {
		return new Intl.DateTimeFormat(undefined, { weekday: 'short', day: 'numeric', month: 'short', hour: '2-digit', minute: '2-digit', hourCycle: 'h23', timeZone: timezone }).format(
			new Date(iso),
		);
	}
	function repeatText(r: { recurrence: string | null; recurrence_weekday?: number | null; recurrence_day_of_month?: number | null }): string | null {
		if (r.recurrence === 'daily') return 'Every day';
		if (r.recurrence === 'weekly') return r.recurrence_weekday != null ? `Every ${WEEKDAYS[r.recurrence_weekday]}` : 'Weekly';
		if (r.recurrence === 'monthly') return r.recurrence_day_of_month != null ? `Monthly on day ${r.recurrence_day_of_month}` : 'Monthly';
		return null;
	}
	function supporting(r: Reminder): string {
		return [repeatText(r), r.notes].filter(Boolean).join(' · ') || (r.created_by === 'agent' ? 'Set in chat' : 'Set here');
	}
	function agentName(agent: string): string {
		return agent === 'chitchat' ? 'Nomi' : agent.charAt(0).toUpperCase() + agent.slice(1);
	}

	const groups = $derived.by(() => {
		const map = new Map<string, { heading: string; items: Reminder[] }>();
		for (const r of reminders?.upcoming ?? []) {
			const key = dayKey(r.due_at);
			if (!map.has(key)) map.set(key, { heading: dayHeading(r.due_at), items: [] });
			map.get(key)!.items.push(r);
		}
		return [...map.values()];
	});

	function notify(message: string) {
		snackbarMessage = message;
		snackbar = true;
	}

	const PAST_LABEL: Record<string, string> = { fired: 'Went off', done: 'Done', cancelled: 'Cancelled' };
</script>

<div class="page">
	<div class="page__inner">
		<header class="head">
			<div class="head__text">
				<h1 class="md-display-small head__title">Reminders</h1>
				<p class="md-body-large head__lede">Everything Nomi will nudge you about. Ask in any chat, like “remind me to stretch at 4pm”, or add one here.</p>
			</div>
			<div class="head__actions">
				<AgentShape agent="reminders" size={64} class="head__mark" />
				<Button variant="gradient" size="m" onclick={openSheet}>
					<IconPlus size={20} />
					New reminder
				</Button>
			</div>
		</header>

		{#if !reminders}
			<p class="notice" role="alert">Couldn't load your reminders just now. Reload the page to try again.</p>
		{:else}
			<section class="surface" aria-labelledby="upcoming-heading">
				<div class="surface__head">
					<h2 id="upcoming-heading" class="md-title-large surface__title">Coming up</h2>
					<span class="count">{reminders.upcoming.length}</span>
				</div>
				{#if groups.length === 0}
					<div class="empty">
						<AgentShape agent="reminders" size={88} working />
						<div>
							<p class="md-title-medium empty__title">Nothing to remind you about</p>
							<p class="md-body-medium empty__body">Add one, or just tell Nomi in a chat.</p>
						</div>
					</div>
				{:else}
					{#each groups as group (group.heading)}
						<h3 class="day">{group.heading}</h3>
						<List>
							{#each group.items as r (r.id)}
								<ListItem headline={r.title} supportingText={supporting(r)} class="row">
									{#snippet leading()}
										<form
											method="POST"
											action="?/act"
											use:enhance={() => {
												return async ({ result, update }) => {
													await update();
													if (result.type === 'success') notify(`Ticked off “${r.title}”.`);
												};
											}}
										>
											<input type="hidden" name="id" value={r.id} />
											<input type="hidden" name="action" value="done" />
											<Checkbox aria-label="Mark “{r.title}” done" onchange={(e) => (e.currentTarget as HTMLInputElement).form?.requestSubmit()} />
										</form>
									{/snippet}
									{#snippet trailing()}
										<div class="row__trailing">
											<span class="time">{time(r.due_at)}</span>
											<Menu bind:open={() => menus[r.id] ?? false, (v) => (menus[r.id] = v)}>
												{#snippet trigger({ toggle })}
													<IconButton aria-label="Snooze or cancel “{r.title}”" onclick={toggle}>
														<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><circle cx="12" cy="13" r="8" /><path d="M12 9v4l2 2M5 3 2 6M22 6l-3-3" /></svg>
													</IconButton>
												{/snippet}
												<p class="md-label-medium menu__title">Snooze</p>
												{#each SNOOZES as s (s.minutes)}
													<form
														method="POST"
														action="?/act"
														use:enhance={() => {
															menus[r.id] = false;
															return async ({ result, update }) => {
																await update();
																if (result.type === 'success') notify(`Snoozed “${r.title}” for ${s.label.toLowerCase()}.`);
															};
														}}
													>
														<input type="hidden" name="id" value={r.id} />
														<input type="hidden" name="action" value="snooze" />
														<input type="hidden" name="minutes" value={s.minutes} />
														<MenuItem type="submit">{s.label}</MenuItem>
													</form>
												{/each}
												<form
													method="POST"
													action="?/act"
													use:enhance={() => {
														menus[r.id] = false;
														return async ({ result, update }) => {
															await update();
															if (result.type === 'success') notify(`Cancelled “${r.title}”.`);
														};
													}}
												>
													<input type="hidden" name="id" value={r.id} />
													<input type="hidden" name="action" value="cancel" />
													<MenuItem type="submit">Cancel reminder</MenuItem>
												</form>
											</Menu>
										</div>
									{/snippet}
								</ListItem>
							{/each}
						</List>
					{/each}
				{/if}
			</section>

			{#if reminders.scheduled_tasks.length > 0}
				<section class="surface surface--tonal" aria-labelledby="tasks-heading">
					<div class="surface__head">
						<h2 id="tasks-heading" class="md-title-large surface__title">Scheduled for the crew</h2>
						<span class="count">{reminders.scheduled_tasks.length}</span>
					</div>
					<p class="md-body-medium surface__lede">Work an agent will do for you later, like a weekly spending summary.</p>
					<List>
						{#each reminders.scheduled_tasks as task (task.id)}
							<ListItem
								headline={task.label}
								supportingText={[shortWhen(task.run_at), agentName(task.agent), repeatText(task)].filter(Boolean).join(' · ')}
							>
								{#snippet leading()}
									<AgentShape agent={task.agent} size={36} />
								{/snippet}
								{#snippet trailing()}
									<form
										method="POST"
										action="?/cancelTask"
										use:enhance={() => {
											return async ({ result, update }) => {
												await update();
												if (result.type === 'success') notify(`Cancelled “${task.label}”.`);
											};
										}}
									>
										<input type="hidden" name="id" value={task.id} />
										<IconButton type="submit" aria-label="Cancel “{task.label}”"><IconClose size={20} /></IconButton>
									</form>
								{/snippet}
							</ListItem>
						{/each}
					</List>
				</section>
			{/if}

			{#if reminders.past.length > 0}
				<section class="surface surface--quiet" aria-labelledby="past-heading">
					<h2 id="past-heading" class="md-title-large surface__title">Recently</h2>
					<List>
						{#each reminders.past as r (r.id)}
							<ListItem headline={r.title} supportingText="{PAST_LABEL[r.status]} · {shortWhen(r.last_fired_at ?? r.due_at)}">
								{#snippet leading()}
									<span class="status status--{r.status}" aria-hidden="true">
										{#if r.status === 'done'}
											<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round"><path d="m5 12 5 5 9-10" /></svg>
										{:else if r.status === 'cancelled'}
											<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round"><path d="M6 6l12 12M18 6 6 18" /></svg>
										{:else}
											<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9" /><path d="M10.3 21a1.94 1.94 0 0 0 3.4 0" /></svg>
										{/if}
									</span>
								{/snippet}
							</ListItem>
						{/each}
					</List>
				</section>
			{/if}
		{/if}
	</div>
</div>

<BottomSheet bind:open={sheetOpen}>
	<form
		method="POST"
		action="?/create"
		class="sheet"
		use:enhance={() => {
			saving = true;
			return async ({ result, update }) => {
				await update({ reset: false });
				saving = false;
				if (result.type === 'success') {
					sheetOpen = false;
					notify(`Reminder set for ${new Date(dueAt).toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' })}.`);
				}
			};
		}}
	>
		<div class="sheet__head">
			<AgentShape agent="reminders" size={36} />
			<h2 class="md-headline-small-emphasized sheet__title">New reminder</h2>
		</div>
		<TextField id="reminder-title" name="title" label="Remind me to…" bind:value={title} required maxlength={200} />
		<TextField id="reminder-notes" name="notes" label="Notes (optional)" bind:value={notes} maxlength={500} />
		<label class="when">
			<span class="field-label">When</span>
			<input type="datetime-local" bind:value={when} required class="when__input" />
		</label>
		<input type="hidden" name="due_at" value={dueAt} />
		<div class="repeat">
			<span class="field-label">Repeat</span>
			<ButtonGroup options={REPEATS} bind:value={repeat} name="recurrence" aria-label="Repeat" />
		</div>
		{#if form?.error}
			<p class="sheet__error" role="alert">{form.error}</p>
		{/if}
		<div class="sheet__actions">
			<Button type="button" variant="text" onclick={() => (sheetOpen = false)}>Cancel</Button>
			<Button type="submit" variant="filled" size="m" disabled={saving || !title.trim() || !dueAt}>Save reminder</Button>
		</div>
	</form>
</BottomSheet>

<Snackbar bind:open={snackbar} message={snackbarMessage} />

<style>
	.page {
		height: 100%;
		overflow-y: auto;
		padding: 32px clamp(16px, 4vw, 56px) 56px;
		box-sizing: border-box;
	}
	.page__inner {
		max-width: 920px;
		margin: 0 auto;
		display: flex;
		flex-direction: column;
		gap: 20px;
	}
	.head {
		display: flex;
		align-items: flex-end;
		justify-content: space-between;
		gap: 20px;
		flex-wrap: wrap;
	}
	.head__text {
		flex: 1 1 360px;
	}
	.head__title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
	}
	.head__lede {
		margin: 8px 0 0;
		max-width: 56ch;
		color: var(--md-sys-color-on-surface-variant);
	}
	.head__actions {
		display: flex;
		align-items: center;
		gap: 16px;
	}
	@media (max-width: 640px) {
		.head__actions :global(.head__mark) {
			display: none;
		}
	}
	.notice {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
	}

	.surface {
		padding: 20px 12px 12px;
		border-radius: var(--md-sys-shape-corner-extra-large-increased);
		background: var(--md-sys-color-surface-container-lowest);
	}
	.surface--tonal {
		background: color-mix(in srgb, var(--md-sys-color-secondary-container) 55%, var(--md-sys-color-surface-container-lowest));
	}
	.surface--quiet {
		background: var(--md-sys-color-surface-container-low);
	}
	.surface__head {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 0 12px 6px;
	}
	.surface__title {
		margin: 0;
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}
	.surface--quiet .surface__title {
		padding: 0 12px 6px;
	}
	.surface__lede {
		margin: 0;
		padding: 0 12px 8px;
		color: var(--md-sys-color-on-surface-variant);
	}
	.count {
		min-width: 24px;
		height: 24px;
		padding: 0 8px;
		box-sizing: border-box;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
		font-weight: 600;
	}
	.day {
		margin: 14px 14px 2px;
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
		font-weight: 500;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--md-sys-color-on-surface-variant);
	}
	.row__trailing {
		display: flex;
		align-items: center;
		gap: 4px;
	}
	/* The time sits in a tonal pill so the day's schedule scans at a glance. */
	.time {
		padding: 6px 12px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.875rem;
		font-weight: 600;
		font-variant-numeric: tabular-nums;
	}
	.menu__title {
		margin: 0;
		padding: 4px 8px 6px;
		color: var(--md-sys-color-on-surface-variant);
	}
	.status {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 36px;
		height: 36px;
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface-variant);
	}
	.status--done {
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}
	.empty {
		display: flex;
		align-items: center;
		gap: 20px;
		padding: 16px 14px 20px;
	}
	.empty__title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
	}
	.empty__body {
		margin: 4px 0 0;
		color: var(--md-sys-color-on-surface-variant);
	}

	.sheet {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}
	.sheet__head {
		display: flex;
		align-items: center;
		gap: 12px;
	}
	.sheet__title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
	}
	.field-label {
		font-size: 0.8125rem;
		font-weight: 600;
		color: var(--md-sys-color-on-surface-variant);
	}
	.when,
	.repeat {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.when__input {
		height: 56px;
		padding: 0 16px;
		border: 1px solid var(--md-sys-color-outline);
		border-radius: var(--md-sys-shape-corner-small);
		background: transparent;
		color: var(--md-sys-color-on-surface);
		font: inherit;
		color-scheme: light dark;
	}
	.when__input:focus {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: -1px;
	}
	.sheet__error {
		margin: 0;
		color: var(--md-sys-color-error);
		font-size: 0.875rem;
	}
	.sheet__actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
	}
</style>
