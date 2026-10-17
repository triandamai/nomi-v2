<!-- Things in time order: when on the left (a day once, then times under it), what on the right. -->
<script lang="ts">
	import { getLocale } from '$lib/paraglide/runtime';
	import ViewFrame from './ViewFrame.svelte';
	import { formatWhen } from '$lib/views/format';
	import type { TimelineView, ViewProps } from '$lib/views/types';

	let { view }: ViewProps<TimelineView> = $props();

	const events = $derived(
		view.events.map((event, i) => {
			const when = formatWhen(event.at, getLocale());
			const previous = i > 0 ? formatWhen(view.events[i - 1].at, getLocale()).day : null;
			// Only the first event of a day names the day.
			return { ...event, day: when.day === previous && when.time ? null : when.day, time: when.time };
		}),
	);
</script>

<ViewFrame title={view.title}>
	<ol class="timeline">
		{#each events as event, i (i)}
			<li class="event">
				<span class="event__when">
					{#if event.day}<span class="event__day">{event.day}</span>{/if}
					{#if event.time}<span class="event__time">{event.time}</span>{/if}
				</span>
				<span class="event__dot" aria-hidden="true"></span>
				<span class="event__what">
					<span class="event__title">{event.title}</span>
					{#if event.detail}<span class="event__detail">{event.detail}</span>{/if}
				</span>
			</li>
		{/each}
	</ol>
</ViewFrame>

<style>
	.timeline {
		display: flex;
		flex-direction: column;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.event {
		position: relative;
		display: grid;
		grid-template-columns: minmax(64px, auto) 12px minmax(0, 1fr);
		column-gap: 12px;
		padding-bottom: 16px;
	}
	.event:last-child {
		padding-bottom: 0;
	}
	.event:not(:last-child)::after {
		content: '';
		position: absolute;
		top: 16px;
		bottom: -2px;
		/* Placed in the dot's column (an absolute grid child takes its area), under the dot. */
		grid-column: 2;
		grid-row: 1;
		left: 5px;
		width: 2px;
		background: var(--md-sys-color-outline-variant);
	}
	.event__when {
		display: flex;
		flex-direction: column;
		align-items: flex-end;
		gap: 1px;
		text-align: end;
		font-variant-numeric: tabular-nums;
	}
	.event__day {
		font-size: 0.8125rem;
		font-weight: 600;
		color: var(--md-sys-color-on-surface-variant);
		white-space: nowrap;
	}
	.event__time {
		font-size: 0.9375rem;
		font-weight: 600;
		white-space: nowrap;
	}
	.event__dot {
		position: relative;
		z-index: 1;
		width: 12px;
		height: 12px;
		margin-top: 5px;
		border-radius: 50%;
		background: var(--md-sys-color-surface-container-lowest);
		box-shadow: inset 0 0 0 3px var(--md-sys-color-primary);
	}
	.event__what {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.event__title {
		font-weight: 600;
		line-height: 1.4;
	}
	.event__detail {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
		line-height: 1.4;
	}
</style>
