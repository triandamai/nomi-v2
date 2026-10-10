<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	// M3 time picker: a field that opens a dialog to pick a time on a clock dial (hours, then
	// minutes) or on hour / minute wheels, switchable in the dialog. 12- or 24-hour as the
	// person's language writes it. The value is "HH:MM" (24-hour), sent through a hidden input.
	import Button from './Button.svelte';
	import Dialog from './Dialog.svelte';
	import IconButton from './IconButton.svelte';
	import PickerField from './PickerField.svelte';
	import Wheel from './Wheel.svelte';
	import { displayTime, formatTime, from12Hour, parseTime, to12Hour, uses12Hour } from '$lib/dates';

	let {
		id,
		label,
		value = $bindable(''),
		name,
		variant = 'dial',
		step = 1,
		hour12,
		placeholder,
		required = false,
		disabled = false,
		error = false,
		supportingText,
		onchange,
	}: {
		id?: string;
		label: string;
		/** "HH:MM", 24-hour. */
		value?: string;
		name?: string;
		/** How the dialog opens; the person can switch. */
		variant?: 'dial' | 'wheel';
		/** Minutes between picks on the wheel (the dial snaps to it too). */
		step?: number;
		/** Defaults to how the language writes the time. */
		hour12?: boolean;
		placeholder?: string;
		required?: boolean;
		disabled?: boolean;
		error?: boolean;
		supportingText?: string;
		onchange?: () => void;
	} = $props();

	const uid = $props.id();
	const fieldId = $derived(id ?? `${uid}-field`);
	const locale = $derived(getLocale());
	const twelve = $derived(hour12 ?? uses12Hour(locale));
	const pad = (n: number) => String(n).padStart(2, '0');

	let open = $state(false);
	let input = $state<'dial' | 'wheel'>('dial');
	let picking = $state<'hour' | 'minute'>('hour');
	let hour = $state(9);
	let minute = $state(0);
	let dial = $state<HTMLDivElement | null>(null);
	let dragging = false;

	const shown = $derived(displayTime(value, locale));
	const pm = $derived(hour >= 12);
	const hourText = $derived(twelve ? String(to12Hour(hour).hour) : pad(hour));

	function show() {
		if (disabled) return;
		const time = parseTime(value);
		const now = new Date();
		hour = time?.hour ?? now.getHours();
		minute = time?.minute ?? Math.round(now.getMinutes() / step) * step % 60;
		input = variant;
		picking = 'hour';
		open = true;
	}

	function confirm() {
		value = formatTime(hour, minute);
		open = false;
		onchange?.();
	}

	function clear() {
		value = '';
		open = false;
		onchange?.();
	}

	function setPeriod(afternoon: boolean) {
		if (afternoon !== pm) hour = (hour + 12) % 24;
	}

	// The dial: 256px across, numbers on a ring 100px out (and, for 24-hour hours, 13–00 on an
	// inner ring 64px out).
	const SIZE = 256;
	const OUTER = 100;
	const INNER = 64;
	type Mark = { value: number; label: string; angle: number; radius: number };
	const marks = $derived.by((): Mark[] => {
		if (picking === 'minute') return Array.from({ length: 12 }, (_, i) => ({ value: i * 5, label: pad(i * 5), angle: i * 30, radius: OUTER }));
		if (twelve) return Array.from({ length: 12 }, (_, i) => ({ value: i, label: String(i === 0 ? 12 : i), angle: i * 30, radius: OUTER }));
		return [
			...Array.from({ length: 12 }, (_, i) => ({ value: i === 0 ? 12 : i, label: String(i === 0 ? 12 : i), angle: i * 30, radius: OUTER })),
			...Array.from({ length: 12 }, (_, i) => ({ value: i === 0 ? 0 : i + 12, label: i === 0 ? '00' : String(i + 12), angle: i * 30, radius: INNER })),
		];
	});
	const hand = $derived.by(() => {
		if (picking === 'minute') return { angle: minute * 6, radius: OUTER };
		if (twelve) return { angle: (hour % 12) * 30, radius: OUTER };
		return { angle: (hour % 12) * 30, radius: hour === 0 || hour > 12 ? INNER : OUTER };
	});
	const isPicked = (mark: Mark) =>
		picking === 'minute' ? mark.value === minute : twelve ? mark.value === hour % 12 : mark.value === hour;
	const at = (angle: number, radius: number) => ({
		x: SIZE / 2 + radius * Math.sin((angle * Math.PI) / 180),
		y: SIZE / 2 - radius * Math.cos((angle * Math.PI) / 180),
	});

	function pickFromPointer(event: PointerEvent) {
		if (!dial) return;
		const box = dial.getBoundingClientRect();
		const dx = event.clientX - (box.left + box.width / 2);
		const dy = event.clientY - (box.top + box.height / 2);
		const angle = (Math.atan2(dx, -dy) * 180) / Math.PI;
		const turn = (angle + 360) % 360;
		const scale = box.width / SIZE;
		if (picking === 'minute') {
			minute = (Math.round(turn / 6 / step) * step) % 60;
			return;
		}
		const slot = Math.round(turn / 30) % 12;
		if (twelve) hour = from12Hour(slot === 0 ? 12 : slot, pm);
		else {
			const inner = Math.hypot(dx, dy) < ((OUTER + INNER) / 2) * scale;
			hour = inner ? (slot === 0 ? 0 : slot + 12) : slot === 0 ? 12 : slot;
		}
	}

	function onDialDown(event: PointerEvent) {
		dragging = true;
		(event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
		pickFromPointer(event);
	}
	function onDialMove(event: PointerEvent) {
		if (dragging) pickFromPointer(event);
	}
	function onDialUp() {
		if (!dragging) return;
		dragging = false;
		if (picking === 'hour') picking = 'minute';
	}
	function onDialKeydown(event: KeyboardEvent) {
		const delta = { ArrowUp: 1, ArrowRight: 1, ArrowDown: -1, ArrowLeft: -1 }[event.key];
		if (delta === undefined) {
			if (event.key === 'Enter' && picking === 'hour') picking = 'minute';
			return;
		}
		event.preventDefault();
		if (picking === 'minute') minute = (minute + delta * step + 60) % 60;
		else hour = (hour + delta + 24) % 24;
	}

	const hourOptions = $derived(
		twelve
			? Array.from({ length: 12 }, (_, i) => ({ value: i === 0 ? 12 : i, label: String(i === 0 ? 12 : i) }))
			: Array.from({ length: 24 }, (_, i) => ({ value: i, label: pad(i) })),
	);
	const minuteOptions = $derived(Array.from({ length: Math.ceil(60 / step) }, (_, i) => ({ value: i * step, label: pad(i * step) })));
	const periodLabels = $derived.by(() => {
		const part = (h: number) =>
			new Intl.DateTimeFormat(locale, { hour: 'numeric', hour12: true }).formatToParts(new Date(2023, 0, 1, h)).find((p) => p.type === 'dayPeriod')?.value;
		return { am: part(9) ?? 'AM', pm: part(15) ?? 'PM' };
	});
	const periodOptions = $derived([
		{ value: 'am', label: periodLabels.am },
		{ value: 'pm', label: periodLabels.pm },
	]);
	// The wheels can't land between steps; the dial can, so snap when switching to them.
	const wheelMinute = $derived(minuteOptions.some((o) => o.value === minute) ? minute : Math.floor(minute / step) * step);
</script>

<PickerField id={fieldId} {label} text={shown} placeholder={placeholder ?? m.picker_pick_time()} {error} {disabled} {supportingText} onclick={show}>
	{#snippet icon()}
		<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9" /><path d="M12 7v5l3 2" /></svg>
	{/snippet}
</PickerField>
{#if name}<input type="hidden" {name} {value} />{/if}

<Dialog bind:open class="picker-dialog">
	{#if open}
		<div class="tp">
			<span class="tp__supporting">{m.picker_select_time()}</span>
			<div class="tp__readout">
				<button type="button" class="tp__unit" aria-pressed={input === 'dial' && picking === 'hour'} aria-label={m.picker_hour()} onclick={() => { input = 'dial'; picking = 'hour'; }}>{hourText}</button>
				<span class="tp__colon" aria-hidden="true">:</span>
				<button type="button" class="tp__unit" aria-pressed={input === 'dial' && picking === 'minute'} aria-label={m.picker_minute()} onclick={() => { input = 'dial'; picking = 'minute'; }}>{pad(minute)}</button>
				{#if twelve}
					<div class="tp__period" role="radiogroup" aria-label={m.picker_period()}>
						<button type="button" role="radio" aria-checked={!pm} onclick={() => setPeriod(false)}>{periodLabels.am}</button>
						<button type="button" role="radio" aria-checked={pm} onclick={() => setPeriod(true)}>{periodLabels.pm}</button>
					</div>
				{/if}
			</div>

			{#if input === 'dial'}
				{@const tip = at(hand.angle, hand.radius)}
				<div
					bind:this={dial}
					class="tp__dial"
					role="slider"
					tabindex="0"
					aria-label={picking === 'hour' ? m.picker_hour() : m.picker_minute()}
					aria-valuemin={picking === 'hour' ? 0 : 0}
					aria-valuemax={picking === 'hour' ? 23 : 59}
					aria-valuenow={picking === 'hour' ? hour : minute}
					aria-valuetext={displayTime(formatTime(hour, minute), locale)}
					onpointerdown={onDialDown}
					onpointermove={onDialMove}
					onpointerup={onDialUp}
					onpointercancel={onDialUp}
					onkeydown={onDialKeydown}
				>
					<svg class="tp__hand" viewBox="0 0 {SIZE} {SIZE}" aria-hidden="true">
						<line x1={SIZE / 2} y1={SIZE / 2} x2={tip.x} y2={tip.y} />
						<circle cx={SIZE / 2} cy={SIZE / 2} r="4" />
						<circle class="tp__knob" cx={tip.x} cy={tip.y} r="20" />
					</svg>
					{#each marks as mark (`${mark.radius}-${mark.value}`)}
						{@const spot = at(mark.angle, mark.radius)}
						<span
							class="tp__mark"
							class:tp__mark--inner={mark.radius === INNER}
							class:tp__mark--picked={isPicked(mark)}
							style:left="{(spot.x / SIZE) * 100}%"
							style:top="{(spot.y / SIZE) * 100}%"
							aria-hidden="true">{mark.label}</span
						>
					{/each}
				</div>
			{:else}
				<div class="tp__wheels">
					<Wheel
						options={hourOptions}
						value={twelve ? to12Hour(hour).hour : hour}
						label={m.picker_hour()}
						onchange={(next) => (hour = twelve ? from12Hour(next, pm) : next)}
					/>
					<Wheel options={minuteOptions} value={wheelMinute} label={m.picker_minute()} onchange={(next) => (minute = next)} />
					{#if twelve}
						<Wheel options={periodOptions} value={pm ? 'pm' : 'am'} label={m.picker_period()} onchange={(next) => setPeriod(next === 'pm')} />
					{/if}
				</div>
			{/if}

			<div class="tp__actions">
				<IconButton
					type="button"
					aria-label={input === 'dial' ? m.picker_use_wheel() : m.picker_use_dial()}
					onclick={() => {
						if (input === 'dial') minute = wheelMinute;
						input = input === 'dial' ? 'wheel' : 'dial';
					}}
				>
					{#if input === 'dial'}
						<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M7 4v16M12 4v16M17 4v16M4 12h16" /></svg>
					{:else}
						<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9" /><path d="M12 7v5l3 2" /></svg>
					{/if}
				</IconButton>
				{#if !required}<Button type="button" variant="text" size="s" onclick={clear}>{m.picker_clear()}</Button>{/if}
				<span class="tp__spacer"></span>
				<Button type="button" variant="text" size="s" onclick={() => (open = false)}>{m.common_cancel()}</Button>
				<Button type="button" variant="filled" size="s" onclick={confirm}>{m.picker_ok()}</Button>
			</div>
		</div>
	{/if}
</Dialog>

<style>
	.tp {
		display: flex;
		flex-direction: column;
		gap: 16px;
		padding: 20px 20px 12px;
	}
	.tp__supporting {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
		font-weight: 500;
	}
	.tp__readout {
		display: flex;
		align-items: stretch;
		justify-content: center;
		gap: 6px;
	}
	.tp__unit {
		width: 96px;
		height: 80px;
		border: none;
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface-container-highest);
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-ref-typeface-brand);
		font-size: 3.25rem;
		font-weight: 600;
		font-variant-numeric: tabular-nums;
		line-height: 1;
		cursor: pointer;
		transition:
			background-color var(--nomi-motion-effects-fast),
			border-radius var(--nomi-motion-spatial-fast);
	}
	.tp__unit[aria-pressed='true'] {
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}
	.tp__colon {
		display: flex;
		align-items: center;
		color: var(--md-sys-color-on-surface);
		font-size: 3rem;
		font-weight: 600;
	}
	.tp__period {
		display: flex;
		flex-direction: column;
		margin-left: 6px;
		overflow: hidden;
		border: 1px solid var(--md-sys-color-outline);
		border-radius: var(--md-sys-shape-corner-small);
	}
	.tp__period button {
		flex: 1;
		min-width: 52px;
		border: none;
		background: none;
		color: var(--md-sys-color-on-surface-variant);
		font: inherit;
		font-size: 0.875rem;
		font-weight: 600;
		cursor: pointer;
	}
	.tp__period button + button {
		border-top: 1px solid var(--md-sys-color-outline);
	}
	.tp__period button[aria-checked='true'] {
		background: var(--md-sys-color-tertiary-container);
		color: var(--md-sys-color-on-tertiary-container);
	}
	.tp__dial {
		position: relative;
		width: min(256px, 100%);
		aspect-ratio: 1;
		margin: 0 auto;
		border-radius: 50%;
		background: var(--md-sys-color-surface-container-highest);
		touch-action: none;
		cursor: pointer;
		user-select: none;
	}
	.tp__dial:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 3px;
	}
	.tp__hand {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		pointer-events: none;
	}
	.tp__hand line {
		stroke: var(--md-sys-color-primary);
		stroke-width: 2;
	}
	.tp__hand circle {
		fill: var(--md-sys-color-primary);
	}
	.tp__mark {
		position: absolute;
		display: flex;
		align-items: center;
		justify-content: center;
		width: 40px;
		height: 40px;
		translate: -50% -50%;
		color: var(--md-sys-color-on-surface);
		font-size: 1rem;
		font-variant-numeric: tabular-nums;
		pointer-events: none;
	}
	.tp__mark--inner {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.tp__mark--picked {
		color: var(--md-sys-color-on-primary);
		font-weight: 700;
	}
	.tp__wheels {
		display: flex;
		gap: 4px;
	}
	.tp__actions {
		display: flex;
		align-items: center;
		gap: 4px;
	}
	.tp__spacer {
		flex: 1;
	}
</style>
