<script lang="ts">
	import { tick } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	// M3 date picker: a field that opens a dialog to pick one day or a range of days, on a
	// month calendar or on day / month / year wheels (switchable in the dialog). Values are plain
	// "YYYY-MM-DD" days, sent with the form through hidden inputs named like the old date inputs.
	import Button from './Button.svelte';
	import Dialog from './Dialog.svelte';
	import IconButton from './IconButton.svelte';
	import PickerField from './PickerField.svelte';
	import Wheel from './Wheel.svelte';
	import {
		addMonths,
		clampDate,
		dateFieldOrder,
		daysInMonth,
		formatDate,
		isoFrom,
		monthGrid,
		monthLabels,
		orderRange,
		parseISODate,
		todayISO,
		weekdayLabels,
		weekStartFor,
	} from '$lib/dates';

	let {
		id,
		label,
		mode = 'single',
		value = $bindable(''),
		start = $bindable(''),
		end = $bindable(''),
		name,
		nameStart,
		nameEnd,
		min,
		max,
		variant = 'calendar',
		placeholder,
		required = false,
		disabled = false,
		error = false,
		supportingText,
		onchange,
	}: {
		id?: string;
		label: string;
		/** One day, or a range from `start` to `end`. */
		mode?: 'single' | 'range';
		/** The picked day (single). */
		value?: string;
		/** The range's first and last days (range). */
		start?: string;
		end?: string;
		/** Form field for the day (single). */
		name?: string;
		/** Form fields for the range's ends. */
		nameStart?: string;
		nameEnd?: string;
		min?: string;
		max?: string;
		/** How the dialog opens; the person can switch. */
		variant?: 'calendar' | 'wheel';
		placeholder?: string;
		/** Hides "Clear". */
		required?: boolean;
		disabled?: boolean;
		error?: boolean;
		supportingText?: string;
		onchange?: () => void;
	} = $props();

	const uid = $props.id();
	const fieldId = $derived(id ?? `${uid}-field`);
	const locale = $derived(getLocale());
	const weekStart = $derived(weekStartFor(locale));
	const weekdays = $derived(weekdayLabels(locale, weekStart));
	const months = $derived(monthLabels(locale));
	const range = $derived(mode === 'range');

	let open = $state(false);
	let input = $state<'calendar' | 'wheel'>('calendar');
	let picking = $state<'days' | 'years'>('days');
	let draftStart = $state('');
	let draftEnd = $state('');
	/** Which end of a range the wheels are setting. */
	let edge = $state<'start' | 'end'>('start');
	let view = $state({ year: 2026, month: 0 });
	/** The day the keyboard is on in the calendar. */
	let focusDay = $state('');
	/** The day under the pointer, to preview a range before its end is picked. */
	let hoverDay = $state('');
	let grid = $state<HTMLDivElement | null>(null);
	let yearList = $state<HTMLDivElement | null>(null);

	const today = $derived(todayISO());
	const shown = $derived(
		range
			? start
				? `${formatDate(start, locale)} – ${formatDate(end || start, locale)}`
				: ''
			: formatDate(value, locale),
	);
	const fallbackDay = $derived(clampDate(today, min, max));
	const days = $derived(monthGrid(view.year, view.month, weekStart));
	const monthTitle = $derived(
		new Intl.DateTimeFormat(locale, { month: 'long', year: 'numeric' }).format(new Date(view.year, view.month, 1)),
	);
	const firstYear = $derived(min ? Number(min.slice(0, 4)) : Number(today.slice(0, 4)) - 100);
	const lastYear = $derived(max ? Number(max.slice(0, 4)) : Number(today.slice(0, 4)) + 50);
	const years = $derived(Array.from({ length: lastYear - firstYear + 1 }, (_, i) => firstYear + i));

	const headline = $derived.by(() => {
		const short = (iso: string) => formatDate(iso, locale, { month: 'short', day: 'numeric' });
		if (!range) return draftStart ? formatDate(draftStart, locale, { weekday: 'short', month: 'short', day: 'numeric' }) : m.picker_no_date();
		if (!draftStart) return m.picker_no_date();
		return `${short(draftStart)} – ${draftEnd ? short(draftEnd) : m.picker_end()}`;
	});

	const outOfBounds = (iso: string) => Boolean((min && iso < min) || (max && iso > max));

	function show() {
		if (disabled) return;
		draftStart = range ? start : value;
		draftEnd = range ? end : '';
		input = variant;
		picking = 'days';
		edge = 'start';
		hoverDay = '';
		const anchor = parseISODate(draftStart || fallbackDay) ?? new Date();
		view = { year: anchor.getFullYear(), month: anchor.getMonth() };
		focusDay = draftStart || fallbackDay;
		open = true;
	}

	function confirm() {
		if (range) {
			start = draftStart;
			end = draftStart ? draftEnd || draftStart : '';
		} else {
			value = draftStart;
		}
		open = false;
		onchange?.();
	}

	function clear() {
		value = '';
		start = '';
		end = '';
		open = false;
		onchange?.();
	}

	function pickDay(iso: string) {
		if (outOfBounds(iso)) return;
		focusDay = iso;
		if (!range) {
			draftStart = iso;
		} else if (!draftStart || draftEnd) {
			draftStart = iso;
			draftEnd = '';
		} else {
			[draftStart, draftEnd] = orderRange(draftStart, iso);
		}
	}

	function moveMonth(delta: number) {
		view = addMonths(view.year, view.month, delta);
	}

	async function moveFocus(iso: string) {
		const date = parseISODate(iso);
		if (!date) return;
		focusDay = iso;
		if (date.getFullYear() !== view.year || date.getMonth() !== view.month) view = { year: date.getFullYear(), month: date.getMonth() };
		await tick();
		grid?.querySelector<HTMLButtonElement>(`[data-day="${iso}"]`)?.focus();
	}

	function onGridKeydown(event: KeyboardEvent) {
		const date = parseISODate(focusDay);
		if (!date) return;
		const shift: Record<string, () => void> = {
			ArrowLeft: () => date.setDate(date.getDate() - 1),
			ArrowRight: () => date.setDate(date.getDate() + 1),
			ArrowUp: () => date.setDate(date.getDate() - 7),
			ArrowDown: () => date.setDate(date.getDate() + 7),
			PageUp: () => date.setMonth(date.getMonth() - 1),
			PageDown: () => date.setMonth(date.getMonth() + 1),
			Home: () => date.setDate(date.getDate() - ((date.getDay() - weekStart + 7) % 7)),
			End: () => date.setDate(date.getDate() + 6 - ((date.getDay() - weekStart + 7) % 7)),
		};
		if (!shift[event.key]) return;
		event.preventDefault();
		shift[event.key]();
		const pad = (n: number) => String(n).padStart(2, '0');
		void moveFocus(`${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`);
	}

	async function showYears() {
		picking = picking === 'years' ? 'days' : 'years';
		if (picking === 'years') {
			await tick();
			yearList?.querySelector<HTMLElement>('[aria-pressed="true"]')?.scrollIntoView({ block: 'center' });
		}
	}

	function pickYear(year: number) {
		view = { year, month: view.month };
		picking = 'days';
	}

	// The wheels show (and set) one day: the single pick, or the range end being edited.
	const wheelDay = $derived((edge === 'end' ? draftEnd || draftStart : draftStart) || fallbackDay);
	const wheelParts = $derived.by(() => {
		const date = parseISODate(wheelDay) ?? new Date();
		return { year: date.getFullYear(), month: date.getMonth(), day: date.getDate() };
	});
	const fieldOrder = $derived(dateFieldOrder(locale));

	function setWheel(part: 'day' | 'month' | 'year', next: number) {
		const parts = { ...wheelParts, [part]: next };
		const iso = clampDate(isoFrom(parts.year, parts.month, parts.day), min, max);
		if (!range || edge === 'start') {
			draftStart = iso;
			if (range && draftEnd && draftEnd < iso) draftEnd = iso;
		} else {
			draftEnd = iso;
			if (!draftStart || draftStart > iso) draftStart = iso;
		}
		const date = parseISODate(iso)!;
		view = { year: date.getFullYear(), month: date.getMonth() };
	}

	const wheelOptions = $derived({
		day: Array.from({ length: daysInMonth(wheelParts.year, wheelParts.month) }, (_, i) => ({
			value: i + 1,
			label: String(i + 1),
			disabled: outOfBounds(isoFrom(wheelParts.year, wheelParts.month, i + 1)),
		})),
		month: months.map((label, i) => ({
			value: i,
			label,
			disabled: Boolean(
				(min && isoFrom(wheelParts.year, i, daysInMonth(wheelParts.year, i)) < min) || (max && isoFrom(wheelParts.year, i, 1) > max),
			),
		})),
		year: years.map((y) => ({ value: y, label: String(y) })),
	});
	const wheelLabels = { day: m.picker_day, month: m.picker_month, year: m.picker_year };

	function dayState(iso: string) {
		const end = draftEnd || (range && draftStart && hoverDay && !draftEnd ? hoverDay : '');
		const [from, to] = end ? orderRange(draftStart, end) : [draftStart, draftStart];
		return {
			selected: iso === draftStart || (range && iso === draftEnd),
			from: range && Boolean(end) && iso === from && from !== to,
			to: range && Boolean(end) && iso === to && from !== to,
			between: range && Boolean(end) && iso > from && iso < to,
			preview: range && !draftEnd,
		};
	}
</script>

<PickerField id={fieldId} {label} text={shown} placeholder={placeholder ?? m.picker_pick_date()} {error} {disabled} {supportingText} onclick={show}>
	{#snippet icon()}
		<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="5" width="18" height="16" rx="3" /><path d="M3 10h18M8 3v4M16 3v4" /></svg>
	{/snippet}
</PickerField>
{#if !range && name}<input type="hidden" {name} {value} />{/if}
{#if range && nameStart}<input type="hidden" name={nameStart} value={start} />{/if}
{#if range && nameEnd}<input type="hidden" name={nameEnd} value={end} />{/if}

<Dialog bind:open class="picker-dialog">
	{#if open}
		<div class="dp">
			<div class="dp__head">
				<span class="dp__supporting">{range ? m.picker_select_range() : m.picker_select_date()}</span>
				<div class="dp__headline-row">
					<span class="dp__headline" aria-live="polite">{headline}</span>
					<IconButton
						type="button"
						aria-label={input === 'calendar' ? m.picker_use_wheel() : m.picker_use_calendar()}
						onclick={() => (input = input === 'calendar' ? 'wheel' : 'calendar')}
					>
						{#if input === 'calendar'}
							<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M7 4v16M12 4v16M17 4v16M4 12h16" /></svg>
						{:else}
							<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="5" width="18" height="16" rx="3" /><path d="M3 10h18M8 3v4M16 3v4" /></svg>
						{/if}
					</IconButton>
				</div>
			</div>

			{#if input === 'calendar'}
				<div class="dp__nav">
					<button type="button" class="dp__month" aria-expanded={picking === 'years'} onclick={showYears}>
						{monthTitle}
						<svg class="dp__caret" class:dp__caret--up={picking === 'years'} width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m6 9 6 6 6-6" /></svg>
					</button>
					{#if picking === 'days'}
						<IconButton type="button" aria-label={m.picker_prev_month()} onclick={() => moveMonth(-1)}>
							<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="m15 18-6-6 6-6" /></svg>
						</IconButton>
						<IconButton type="button" aria-label={m.picker_next_month()} onclick={() => moveMonth(1)}>
							<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="m9 18 6-6-6-6" /></svg>
						</IconButton>
					{/if}
				</div>

				{#if picking === 'years'}
					<div class="dp__years" bind:this={yearList}>
						{#each years as year (year)}
							<button type="button" class="dp__year" class:dp__year--current={String(year) === today.slice(0, 4)} aria-pressed={year === view.year} onclick={() => pickYear(year)}>{year}</button>
						{/each}
					</div>
				{:else}
					<div class="dp__weekdays" aria-hidden="true">
						{#each weekdays as day, i (i)}<span>{day}</span>{/each}
					</div>
					<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
					<div class="dp__grid" role="group" aria-label={monthTitle} bind:this={grid} onkeydown={onGridKeydown} onpointerleave={() => (hoverDay = '')}>
						{#each days as day (day.iso)}
							{@const state = dayState(day.iso)}
							<div
								class="dp__cell"
								class:dp__cell--between={state.between}
								class:dp__cell--from={state.from}
								class:dp__cell--to={state.to}
								class:dp__cell--preview={state.preview}
							>
								{#if day.inMonth}
									<button
										type="button"
										class="dp__day"
										class:dp__day--selected={state.selected}
										class:dp__day--today={day.iso === today}
										data-day={day.iso}
										tabindex={day.iso === focusDay ? 0 : -1}
										disabled={outOfBounds(day.iso)}
										aria-pressed={state.selected}
										aria-label={formatDate(day.iso, locale, { dateStyle: 'full' })}
										onclick={() => pickDay(day.iso)}
										onpointerenter={() => (hoverDay = day.iso)}
									>
										{day.day}
									</button>
								{/if}
							</div>
						{/each}
					</div>
				{/if}
			{:else}
				{#if range}
					<div class="dp__edges" role="radiogroup" aria-label={m.picker_select_range()}>
						<button type="button" role="radio" class="dp__edge" aria-checked={edge === 'start'} onclick={() => (edge = 'start')}>
							<span class="dp__edge-label">{m.picker_start()}</span>
							<span>{draftStart ? formatDate(draftStart, locale) : '—'}</span>
						</button>
						<button type="button" role="radio" class="dp__edge" aria-checked={edge === 'end'} onclick={() => (edge = 'end')}>
							<span class="dp__edge-label">{m.picker_end()}</span>
							<span>{draftEnd || draftStart ? formatDate(draftEnd || draftStart, locale) : '—'}</span>
						</button>
					</div>
				{/if}
				<div class="dp__wheels">
					{#each fieldOrder as part (part)}
						<Wheel options={wheelOptions[part]} value={wheelParts[part]} label={wheelLabels[part]()} onchange={(next) => setWheel(part, next)} />
					{/each}
				</div>
			{/if}

			<div class="dp__actions">
				{#if !required}<Button type="button" variant="text" size="s" onclick={clear}>{m.picker_clear()}</Button>{/if}
				<span class="dp__spacer"></span>
				<Button type="button" variant="text" size="s" onclick={() => (open = false)}>{m.common_cancel()}</Button>
				<Button type="button" variant="filled" size="s" disabled={!draftStart} onclick={confirm}>{m.picker_ok()}</Button>
			</div>
		</div>
	{/if}
</Dialog>

<style>
	:global(dialog.m3-dialog.picker-dialog) {
		width: min(360px, calc(100vw - 32px));
		min-width: 0;
		background: var(--md-sys-color-surface-container-high);
	}
	:global(dialog.m3-dialog.picker-dialog .m3-dialog__content) {
		padding: 0;
	}
	:global(dialog.m3-dialog.picker-dialog .m3-dialog__body) {
		color: var(--md-sys-color-on-surface);
	}
	.dp__head {
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: 16px 12px 12px 24px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.dp__supporting {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
		font-weight: 500;
	}
	.dp__headline-row {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.dp__headline {
		flex: 1;
		min-width: 0;
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.75rem;
		font-weight: 700;
		letter-spacing: -0.02em;
		line-height: 1.2;
	}
	.dp__nav {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 8px 12px 0 16px;
	}
	.dp__month {
		display: inline-flex;
		flex: 1;
		align-items: center;
		gap: 4px;
		height: 40px;
		padding: 0 8px;
		border: none;
		border-radius: var(--md-sys-shape-corner-full);
		background: none;
		color: var(--md-sys-color-on-surface-variant);
		font: inherit;
		font-size: 0.9375rem;
		font-weight: 600;
		text-align: left;
		cursor: pointer;
	}
	.dp__month:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
	.dp__caret {
		transition: transform var(--nomi-motion-spatial-fast);
	}
	.dp__caret--up {
		transform: rotate(180deg);
	}
	.dp__weekdays,
	.dp__grid {
		display: grid;
		grid-template-columns: repeat(7, 1fr);
		padding: 0 12px;
	}
	.dp__weekdays span {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 40px;
		color: var(--md-sys-color-on-surface);
		font-size: 0.875rem;
		font-weight: 600;
	}
	.dp__grid {
		padding-bottom: 8px;
	}
	.dp__cell {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 44px;
	}
	/* The range's band runs behind the days between its ends, and half into each end. */
	.dp__cell--between {
		background: var(--md-sys-color-secondary-container);
	}
	.dp__cell--from {
		background: linear-gradient(90deg, transparent 50%, var(--md-sys-color-secondary-container) 50%);
	}
	.dp__cell--to {
		background: linear-gradient(90deg, var(--md-sys-color-secondary-container) 50%, transparent 50%);
	}
	.dp__cell--preview.dp__cell--between,
	.dp__cell--preview.dp__cell--from,
	.dp__cell--preview.dp__cell--to {
		--md-sys-color-secondary-container: color-mix(in srgb, var(--md-sys-color-secondary-container) 55%, transparent);
	}
	.dp__day {
		width: 40px;
		height: 40px;
		padding: 0;
		border: 1px solid transparent;
		border-radius: 50%;
		background: none;
		color: var(--md-sys-color-on-surface);
		font: inherit;
		font-size: 0.875rem;
		font-variant-numeric: tabular-nums;
		cursor: pointer;
		transition:
			background-color var(--nomi-motion-effects-fast),
			border-radius var(--nomi-motion-spatial-fast);
	}
	.dp__day:hover:not(:disabled) {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
	.dp__day:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 1px;
	}
	.dp__day--today {
		border-color: var(--md-sys-color-primary);
		color: var(--md-sys-color-primary);
	}
	.dp__day--selected,
	.dp__day--selected:hover:not(:disabled) {
		border-color: transparent;
		border-radius: 14px;
		background: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
		font-weight: 700;
	}
	.dp__day:disabled {
		opacity: 0.38;
		cursor: default;
	}
	.dp__years {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 8px 4px;
		height: 300px;
		padding: 8px 16px;
		overflow-y: auto;
		box-sizing: border-box;
	}
	.dp__year {
		height: 40px;
		border: 1px solid transparent;
		border-radius: var(--md-sys-shape-corner-full);
		background: none;
		color: var(--md-sys-color-on-surface-variant);
		font: inherit;
		font-size: 0.9375rem;
		cursor: pointer;
	}
	.dp__year:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
	.dp__year--current {
		border-color: var(--md-sys-color-primary);
		color: var(--md-sys-color-primary);
	}
	.dp__year[aria-pressed='true'] {
		background: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
		font-weight: 700;
	}
	.dp__edges {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 8px;
		padding: 12px 16px 0;
	}
	.dp__edge {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 8px 12px;
		border: 1.5px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-medium);
		background: none;
		color: var(--md-sys-color-on-surface);
		font: inherit;
		font-size: 0.875rem;
		text-align: left;
		cursor: pointer;
	}
	.dp__edge[aria-checked='true'] {
		border-color: var(--md-sys-color-primary);
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.dp__edge-label {
		font-size: 0.75rem;
		font-weight: 600;
		opacity: 0.8;
	}
	.dp__wheels {
		display: flex;
		gap: 4px;
		padding: 16px;
	}
	.dp__actions {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 8px 12px 12px;
	}
	.dp__spacer {
		flex: 1;
	}
</style>
