<script lang="ts" module>
	import type { Component } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';

	export type ReminderChip = { icon: Component<LucideProps>; label: string };
</script>

<script lang="ts">
	import Mascot from '../agents/Mascot.svelte';

	// App / Inbox (BFoiZ) → Quick Add: a reminder in one line, as Kala would hear it. The field
	// (surface, [10,12], border-strong, the accent while typing) holds Kala and the line; under it,
	// what was read from it as chips (the day, the time, how it repeats) and the hint at the end
	// ("↵ add"). Reading the line is the page's: this only shows it. The chips' glyphs are 11px, as
	// drawn: Icon.svelte (the old app's twin) has no 11, so they're Lucide's own, at the 1.5 stroke.
	type Props = {
		label: string;
		value?: string;
		placeholder?: string;
		chips: ReminderChip[];
		hint?: string;
		error?: string;
		busy?: boolean;
		/** For the page's "New reminder" to put focus here. */
		id?: string;
		onsubmit?: () => void;
	};
	let { label, value = $bindable(''), placeholder, chips, hint, error, busy = false, id: given, onsubmit }: Props = $props();

	const own = $props.id();
	const id = $derived(given ?? own);
</script>

<form
	class="flex flex-col gap-2.5"
	onsubmit={(event) => {
		event.preventDefault();
		if (!busy) onsubmit?.();
	}}
>
	<label for={id} class="font-mono text-label tracking-label text-text-3 uppercase">{label}</label>
	<div
		data-quick-frame
		class={[
			'flex flex-col gap-2.5 rounded-md border bg-surface px-2.75 py-2.25 focus-within:shadow-focus',
			error ? 'border-danger' : 'border-border-strong focus-within:border-accent',
		]}
	>
		<div class="flex items-center gap-2">
			<Mascot agent="kala" size={20} />
			<input
				{id}
				bind:value
				{placeholder}
				autocomplete="off"
				enterkeyhint="done"
				aria-invalid={error ? 'true' : undefined}
				aria-describedby="{id}-read{error ? ` ${id}-error` : ''}"
				class="min-w-0 flex-1 bg-transparent text-ui text-text caret-accent outline-none placeholder:text-text-3"
			/>
		</div>
		<!-- The hint may run into the field's right padding, as the pen's does (9px past the row). -->
		<div id="{id}-read" class="-mr-2.25 flex min-h-5.5 flex-wrap items-center gap-1.5">
			{#each chips as chip (chip.label)}
				<span class="inline-flex h-5.5 items-center gap-1 rounded-xs bg-surface-2 px-1.5">
					<span class="inline-flex text-text-3"><chip.icon size={11} strokeWidth={1.5} aria-hidden="true" /></span>
					<span class="font-mono text-label text-text-2">{chip.label}</span>
				</span>
			{/each}
			{#if hint}<span class="ml-auto font-mono text-label text-text-3">{hint}</span>{/if}
		</div>
	</div>
	{#if error}<p id="{id}-error" class="text-caption text-danger">{error}</p>{/if}
</form>
