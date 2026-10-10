<script lang="ts" generics="T extends string | number">
	// One column of a wheel picker: scroll (or drag, or use the arrow keys) and the row in the
	// middle band is the pick. Used for days, months, years, hours and minutes.
	let {
		options,
		value = $bindable(),
		label,
		onchange,
	}: {
		options: { value: T; label: string; disabled?: boolean }[];
		value: T;
		/** What the column is, for screen readers ("Hour"). */
		label: string;
		onchange?: (value: T) => void;
	} = $props();

	const ROW = 40;
	const uid = $props.id();
	let list = $state<HTMLDivElement | null>(null);
	let settle: ReturnType<typeof setTimeout> | undefined;
	// Set while the wheel scrolls itself, so that scroll isn't read back as a pick.
	let steering = false;

	const index = $derived(Math.max(0, options.findIndex((o) => o.value === value)));

	// Keep the picked row in the middle band when the value (or the options) change. A wheel in
	// a dialog that is still opening has no height yet; it lines up once it does.
	$effect(() => {
		const target = index * ROW;
		const el = list;
		if (!el) return;
		let frame = 0;
		const align = () => {
			if (el.clientHeight === 0) {
				frame = requestAnimationFrame(align);
				return;
			}
			if (Math.abs(el.scrollTop - target) < 1) return;
			steering = true;
			el.scrollTo({ top: target, behavior: 'instant' });
			frame = requestAnimationFrame(() => (steering = false));
		};
		align();
		return () => {
			cancelAnimationFrame(frame);
			steering = false;
		};
	});

	function pickAt(i: number) {
		let at = Math.min(options.length - 1, Math.max(0, i));
		// Skip rows that can't be picked, towards the nearest one that can.
		if (options[at]?.disabled) {
			const next = options.findIndex((o, j) => j > at && !o.disabled);
			const prev = options.findLastIndex((o, j) => j < at && !o.disabled);
			at = next === -1 ? prev : prev === -1 ? next : next - at <= at - prev ? next : prev;
			if (at === -1) return;
		}
		if (options[at].value !== value) {
			value = options[at].value;
			onchange?.(value);
		}
		if (list && Math.abs(list.scrollTop - at * ROW) >= 1) list.scrollTo({ top: at * ROW, behavior: 'smooth' });
	}

	function onScroll() {
		if (steering || !list) return;
		clearTimeout(settle);
		settle = setTimeout(() => list && pickAt(Math.round(list.scrollTop / ROW)), 90);
	}

	function onKeydown(event: KeyboardEvent) {
		const step = { ArrowUp: -1, ArrowDown: 1, PageUp: -5, PageDown: 5 }[event.key];
		if (step !== undefined) pickAt(index + step);
		else if (event.key === 'Home') pickAt(0);
		else if (event.key === 'End') pickAt(options.length - 1);
		else return;
		event.preventDefault();
	}
</script>

<div class="wheel">
	<div class="wheel__band" aria-hidden="true"></div>
	<div
		bind:this={list}
		class="wheel__list"
		role="listbox"
		tabindex="0"
		aria-label={label}
		aria-activedescendant="{uid}-{index}"
		onscroll={onScroll}
		onkeydown={onKeydown}
	>
		{#each options as option, i (option.value)}
			<div
				id="{uid}-{i}"
				class="wheel__row"
				class:wheel__row--picked={i === index}
				class:wheel__row--disabled={option.disabled}
				role="option"
				aria-selected={i === index}
				aria-disabled={option.disabled}
				tabindex="-1"
				onclick={() => pickAt(i)}
				onkeydown={() => {}}
			>
				{option.label}
			</div>
		{/each}
	</div>
</div>

<style>
	.wheel {
		position: relative;
		flex: 1;
		min-width: 56px;
		height: 200px;
		/* Rows fade out towards the edges, like a drum turning away. */
		mask-image: linear-gradient(transparent, #000 30%, #000 70%, transparent);
	}
	.wheel__band {
		position: absolute;
		top: 80px;
		right: 4px;
		left: 4px;
		height: 40px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-secondary-container);
		pointer-events: none;
	}
	.wheel__list {
		position: relative;
		height: 100%;
		padding: 80px 0;
		overflow-y: auto;
		box-sizing: border-box;
		scroll-snap-type: y mandatory;
		scrollbar-width: none;
		overscroll-behavior: contain;
		outline: none;
	}
	.wheel__list::-webkit-scrollbar {
		display: none;
	}
	.wheel:has(.wheel__list:focus-visible) .wheel__band {
		outline: 2px solid var(--md-sys-color-primary);
	}
	.wheel__row {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 40px;
		scroll-snap-align: center;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 1.0625rem;
		font-variant-numeric: tabular-nums;
		cursor: pointer;
		user-select: none;
		transition:
			color var(--nomi-motion-effects-fast),
			font-weight var(--nomi-motion-effects-fast);
	}
	.wheel__row--picked {
		color: var(--md-sys-color-on-secondary-container);
		font-weight: 700;
	}
	.wheel__row--disabled {
		opacity: 0.38;
		cursor: default;
	}
</style>
