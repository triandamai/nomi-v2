<script lang="ts" module>
	export type AccessLevel = 'view' | 'write' | 'manage';
	export type AccessArea = { key: string; label: string; description: string };
</script>

<script lang="ts">
	import Segment from '../inputs/Segment.svelte';

	// Admin → Access editor: someone's role in the admin console, area by area, each at none, view,
	// write or manage (a level includes the ones below it). Each area is a radio group drawn as
	// Segmented. Nobody can give more than they hold: levels above the editor's own (`max`) can't be
	// chosen, and an area the person holds above the editor's own can't be changed at all, said in
	// its row. `readonly` shows the role without changing it. With `name`, the role goes with the
	// form as JSON.
	type Value = Record<string, AccessLevel | null>;
	type Props = {
		/** Names the editor ("Admin access for ana@example.com"). */
		label: string;
		areas: AccessArea[];
		value?: Value;
		/** The editor's own level on each area: the most they may give. */
		max: Value;
		levelLabels: { none: string; view: string; write: string; manage: string };
		/** Said in an area's row when it's above what the editor holds. */
		lockedNote: string;
		readonly?: boolean;
		name?: string;
		onchange?: (value: Value) => void;
	};
	let { label, areas, value = $bindable({}), max, levelLabels, lockedNote, readonly = false, name, onchange }: Props = $props();

	const ORDER = [null, 'view', 'write', 'manage'] as const;
	const rank = (level: AccessLevel | null | undefined) => ORDER.indexOf(level ?? null);
	const id = $props.id();

	/** The editor can't change an area held above their own level. */
	const locked = (area: string) => readonly || rank(value[area]) > rank(max[area]) || rank(max[area]) <= 0;
	const allowed = (area: string, level: AccessLevel | null) => !locked(area) && rank(level) <= rank(max[area]);

	function choose(area: string, level: AccessLevel | null, focus?: HTMLElement | null) {
		if (!allowed(area, level)) return;
		value = { ...value, [area]: level };
		onchange?.(value);
		focus?.focus();
	}

	function onkeydown(event: KeyboardEvent, area: string) {
		const step = event.key === 'ArrowRight' || event.key === 'ArrowDown' ? 1 : event.key === 'ArrowLeft' || event.key === 'ArrowUp' ? -1 : 0;
		if (!step) return;
		event.preventDefault();
		const choices = ORDER.filter((level) => allowed(area, level));
		if (!choices.length) return;
		const at = choices.indexOf(value[area] ?? null);
		const next = choices[(at + step + choices.length) % choices.length];
		const group = event.currentTarget as HTMLElement;
		choose(area, next, group.querySelector<HTMLElement>(`[data-level="${next ?? 'none'}"]`));
	}
</script>

<div role="group" aria-label={label} class="flex w-full flex-col rounded-md bg-bg outline -outline-offset-1 outline-border">
	{#if name}<input type="hidden" {name} value={JSON.stringify(value)} />{/if}
	{#each areas as area, index (area.key)}
		{@const isLocked = locked(area.key)}
		<div class={['flex flex-wrap items-center justify-between gap-x-4 gap-y-2 px-3.5 py-2.5', index > 0 && 'border-t border-border']}>
			<div class="flex min-w-0 flex-1 basis-56 flex-col gap-0.5">
				<span id="{id}-{area.key}" class="text-ui font-medium text-text">{area.label}</span>
				<span class="text-caption text-text-3">{isLocked && !readonly && rank(max[area.key]) < rank(value[area.key]) ? lockedNote : area.description}</span>
			</div>
			<div
				role="radiogroup"
				aria-labelledby="{id}-{area.key}"
				aria-disabled={isLocked || undefined}
				tabindex="-1"
				onkeydown={(event) => onkeydown(event, area.key)}
				class="inline-flex shrink-0 gap-0.5 rounded-sm bg-surface-2 p-0.5"
			>
				{#each ORDER as level (level ?? 'none')}
					{@const checked = (value[area.key] ?? null) === level}
					{@const enabled = allowed(area.key, level)}
					<button
						type="button"
						role="radio"
						data-level={level ?? 'none'}
						aria-checked={checked}
						disabled={!enabled && !checked}
						aria-disabled={!enabled || undefined}
						tabindex={checked && enabled ? 0 : -1}
						onclick={() => choose(area.key, level)}
						class={['group inline-flex rounded-sm outline-offset-1 pointer-coarse:min-h-11', enabled ? 'cursor-pointer' : 'cursor-default', !enabled && !checked && 'opacity-40']}
					>
						<Segment label={levelLabels[level ?? 'none']} {checked} />
					</button>
				{/each}
			</div>
		</div>
	{/each}
</div>
