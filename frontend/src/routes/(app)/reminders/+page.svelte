<script lang="ts">
	import { enhance } from '$app/forms';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import Chip from '$lib/components/m3/Chip.svelte';
	import Snackbar from '$lib/components/m3/Snackbar.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
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
	const WEEKDAYS = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];

	let label = $state('');
	let when = $state(defaultWhen());
	let repeat = $state<Repeat>('once');
	let saving = $state(false);
	let snackbar = $state(false);
	let snackbarMessage = $state('');

	function defaultWhen(): string {
		const d = new Date(Date.now() + 60 * 60 * 1000);
		d.setMinutes(0, 0, 0);
		const pad = (n: number) => String(n).padStart(2, '0');
		return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
	}

	// The picker's local time with this browser's UTC offset, so a weekly or monthly reminder
	// keeps the weekday and date the user picked.
	const runAt = $derived.by(() => {
		if (!when) return '';
		const date = new Date(when);
		if (Number.isNaN(date.getTime())) return '';
		const offset = -date.getTimezoneOffset();
		const sign = offset >= 0 ? '+' : '-';
		const pad = (n: number) => String(Math.floor(Math.abs(n))).padStart(2, '0');
		return `${when}:00${sign}${pad(offset / 60)}:${pad(offset % 60)}`;
	});

	function dayKey(iso: string): string {
		return new Intl.DateTimeFormat('en-CA', { timeZone: timezone, year: 'numeric', month: '2-digit', day: '2-digit' }).format(new Date(iso));
	}
	function dayHeading(iso: string): string {
		const key = dayKey(iso);
		const today = dayKey(new Date().toISOString());
		const tomorrow = dayKey(new Date(Date.now() + 86_400_000).toISOString());
		if (key === today) return 'Today';
		if (key === tomorrow) return 'Tomorrow';
		return new Intl.DateTimeFormat(undefined, { weekday: 'long', day: 'numeric', month: 'long', timeZone: timezone }).format(new Date(iso));
	}
	function time(iso: string): string {
		return new Intl.DateTimeFormat(undefined, { hour: '2-digit', minute: '2-digit', hourCycle: 'h23', timeZone: timezone }).format(new Date(iso));
	}
	function shortDate(iso: string): string {
		return new Intl.DateTimeFormat(undefined, { day: 'numeric', month: 'short', timeZone: timezone }).format(new Date(iso));
	}
	function repeatText(r: Reminder): string | null {
		if (r.recurrence === 'daily') return 'Every day';
		if (r.recurrence === 'weekly') return `Every ${WEEKDAYS[r.recurrence_weekday ?? 0]}`;
		if (r.recurrence === 'monthly') return `Monthly on the ${r.recurrence_day_of_month}${ordinal(r.recurrence_day_of_month ?? 1)}`;
		return null;
	}
	function ordinal(n: number): string {
		if (n % 100 >= 11 && n % 100 <= 13) return 'th';
		return ['th', 'st', 'nd', 'rd'][n % 10] ?? 'th';
	}
	function agentName(agent: string): string {
		return agent === 'chitchat' ? 'Nomi' : agent.charAt(0).toUpperCase() + agent.slice(1);
	}

	const groups = $derived.by(() => {
		const map = new Map<string, { heading: string; items: Reminder[] }>();
		for (const r of reminders?.upcoming ?? []) {
			const key = dayKey(r.run_at);
			if (!map.has(key)) map.set(key, { heading: dayHeading(r.run_at), items: [] });
			map.get(key)!.items.push(r);
		}
		return [...map.values()];
	});

	function notify(message: string) {
		snackbarMessage = message;
		snackbar = true;
	}
</script>

<div class="reminders">
	<div class="reminders__inner">
		<header class="reminders__head">
			<div>
				<h1 class="md-display-small reminders__title">Reminders</h1>
				<p class="md-body-large reminders__lede">
					Everything Nomi will nudge you about. Ask in any chat ("remind me to stretch at 4pm") or add one here.
				</p>
			</div>
			<AgentShape agent="planning" size={72} class="reminders__mark" />
		</header>

		{#if !reminders}
			<p class="reminders__notice" role="alert">Couldn't load your reminders just now. Reload the page to try again.</p>
		{:else}
			<div class="layout">
				<section class="upcoming" aria-labelledby="upcoming-heading">
					<h2 id="upcoming-heading" class="section-title">Coming up</h2>
					{#if groups.length === 0}
						<div class="empty">
							<AgentShape agent="planning" size={72} working />
							<p class="md-body-large">Nothing scheduled. Add a reminder, or ask Nomi in a chat.</p>
						</div>
					{:else}
						{#each groups as group (group.heading)}
							<div class="day">
								<h3 class="nomi-meta day__heading">{group.heading}</h3>
								<ul class="day__list">
									{#each group.items as r (r.id)}
										<li class="item">
											<span class="item__time">{time(r.run_at)}</span>
											<span class="item__text">
												<span class="item__label">{r.label}</span>
												<span class="item__meta">
													{#if repeatText(r)}<span class="item__repeat">{repeatText(r)}</span>{/if}
													<a href="/chat/{r.session_id}">{agentName(r.agent)} will follow up in chat</a>
												</span>
											</span>
											<form
												method="POST"
												action="?/cancel"
												use:enhance={() => {
													return async ({ result, update }) => {
														await update();
														if (result.type === 'success') notify(`Cancelled “${r.label}”.`);
													};
												}}
											>
												<input type="hidden" name="id" value={r.id} />
												<button type="submit" class="item__cancel" aria-label="Cancel reminder: {r.label}">Cancel</button>
											</form>
										</li>
									{/each}
								</ul>
							</div>
						{/each}
					{/if}
				</section>

				<aside class="side">
					<section class="new" aria-labelledby="new-heading">
						<h2 id="new-heading" class="section-title">New reminder</h2>
						<form
							method="POST"
							action="?/create"
							class="new__form"
							use:enhance={() => {
								saving = true;
								return async ({ result, update }) => {
									await update({ reset: false });
									saving = false;
									if (result.type === 'success') {
										notify(`Reminder set for ${new Date(runAt).toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' })}.`);
										label = '';
										when = defaultWhen();
										repeat = 'once';
									}
								};
							}}
						>
							<TextField id="label" name="label" label="Remind me to…" bind:value={label} required maxlength={200} />
							<label class="when">
								<span class="when__label">When</span>
								<input type="datetime-local" bind:value={when} required class="when__input" />
							</label>
							<input type="hidden" name="run_at" value={runAt} />
							<div class="repeat">
								<span class="when__label">Repeat</span>
								<div class="repeat__chips" role="radiogroup" aria-label="Repeat">
									{#each REPEATS as option (option.value)}
										<Chip
											type="button"
											variant="filter"
											selected={repeat === option.value}
											role="radio"
											aria-checked={repeat === option.value}
											onclick={() => (repeat = option.value)}
										>
											{option.label}
										</Chip>
									{/each}
								</div>
								<input type="hidden" name="recurrence" value={repeat} />
							</div>
							{#if form?.error}
								<p class="new__error" role="alert">{form.error}</p>
							{/if}
							<Button type="submit" variant="gradient" size="m" disabled={saving || !label.trim() || !runAt}>Add reminder</Button>
						</form>
					</section>

					{#if reminders.past.length > 0}
						<section class="past" aria-labelledby="past-heading">
							<h2 id="past-heading" class="section-title">Recently</h2>
							<ul class="past__list">
								{#each reminders.past as r (r.id)}
									<li class="past__item">
										<span class="past__label">{r.label}</span>
										<span class="past__meta">
											{r.status === 'cancelled' ? 'Cancelled' : 'Done'} · {shortDate(r.last_fired_at ?? r.run_at)}
										</span>
									</li>
								{/each}
							</ul>
						</section>
					{/if}
				</aside>
			</div>
		{/if}
	</div>
</div>

<Snackbar bind:open={snackbar} message={snackbarMessage} />

<style>
	.reminders {
		height: 100%;
		overflow-y: auto;
		padding: 32px clamp(16px, 4vw, 56px) 56px;
		box-sizing: border-box;
	}
	.reminders__inner {
		max-width: 1180px;
		margin: 0 auto;
		display: flex;
		flex-direction: column;
		gap: 28px;
	}
	.reminders__head {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 24px;
	}
	.reminders__title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
	}
	.reminders__lede {
		margin: 8px 0 0;
		max-width: 60ch;
		color: var(--md-sys-color-on-surface-variant);
	}
	@media (max-width: 640px) {
		.reminders__head :global(.reminders__mark) {
			display: none;
		}
	}
	.reminders__notice {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.layout {
		display: grid;
		grid-template-columns: minmax(0, 1.6fr) minmax(0, 1fr);
		gap: 24px;
		align-items: start;
	}
	@media (max-width: 900px) {
		.layout {
			grid-template-columns: minmax(0, 1fr);
		}
		.side {
			order: -1;
		}
	}
	.section-title {
		margin: 0 0 14px;
		font-size: 1.125rem;
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}
	.upcoming {
		padding: 24px;
		border-radius: var(--md-sys-shape-corner-extra-large-increased);
		background: var(--md-sys-color-surface-container-lowest);
	}
	.empty {
		display: flex;
		align-items: center;
		gap: 20px;
		padding: 12px 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.empty p {
		margin: 0;
	}
	.day + .day {
		margin-top: 18px;
	}
	.day__heading {
		margin: 0 0 4px;
	}
	.day__list {
		list-style: none;
		margin: 0;
		padding: 0;
	}
	.item {
		display: grid;
		grid-template-columns: 56px minmax(0, 1fr) auto;
		align-items: center;
		gap: 12px;
		padding: 12px 0;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.day__list .item:last-child {
		border-bottom: none;
	}
	.item__time {
		align-self: start;
		padding-top: 2px;
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.875rem;
		color: var(--md-sys-color-primary);
		font-variant-numeric: tabular-nums;
	}
	.item__text {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.item__label {
		font-weight: 600;
		color: var(--md-sys-color-on-surface);
	}
	.item__meta {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.item__meta a {
		color: inherit;
	}
	.item__repeat {
		padding: 1px 8px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
		font-weight: 600;
	}
	.item__cancel {
		height: 40px;
		padding: 0 14px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-full);
		background: transparent;
		color: var(--md-sys-color-on-surface);
		font: inherit;
		font-size: 0.8125rem;
		font-weight: 600;
		cursor: pointer;
	}
	.item__cancel:hover {
		background: var(--md-sys-color-surface-container-high);
	}
	.side {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}
	.new {
		padding: 24px;
		border-radius: 40px 40px 40px 12px;
		background: color-mix(in srgb, #7ab0ff 14%, var(--md-sys-color-surface-container-lowest));
	}
	.new__form {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.when,
	.repeat {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.repeat__chips {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}
	.when__label {
		font-size: 0.8125rem;
		font-weight: 600;
		color: var(--md-sys-color-on-surface-variant);
	}
	.when__input {
		height: 52px;
		padding: 0 14px;
		border: 1px solid var(--md-sys-color-outline);
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
		font: inherit;
		color-scheme: light dark;
	}
	.when__input:focus {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 1px;
	}
	.new__error {
		margin: 0;
		color: var(--md-sys-color-error);
		font-size: 0.875rem;
	}
	.past {
		padding: 22px 24px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-low);
	}
	.past__list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 10px;
	}
	.past__item {
		display: flex;
		flex-direction: column;
	}
	.past__label {
		color: var(--md-sys-color-on-surface);
		font-size: 0.9375rem;
	}
	.past__meta {
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
</style>
