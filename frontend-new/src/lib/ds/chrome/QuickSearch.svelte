<script lang="ts" module>
	export type QuickSearchRow = { id: string; title: string; snippet?: string; meta: string; href: string };
	export type QuickSearchGroup = { kind: 'chat' | 'memory' | 'file'; rows: QuickSearchRow[] };
</script>

<script lang="ts">
	import type { Component } from 'svelte';
	import { Brain, FileCode, MessageSquare, Search as SearchIcon, type LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import Kbd from '../actions/Kbd.svelte';

	// App / Home (Vr4nn) → Top Bar → Search: the pen's quiet 240 × 30 field ("Search chats, memory,
	// files" and its ⌘K), [0,10] with the 14 glyph. What it finds the pen doesn't draw: a floating
	// panel under it in the menu's style (surface, border-strong inside, the float shadow, 4 in), the
	// backend's kinds as groups under mono labels (Chats, Memory, Files), each row its kind's glyph,
	// the title, a line of the match, and when. A combobox: the arrows move through the rows, Enter
	// opens one, Escape closes.
	type Props = {
		value?: string;
		/** What was found, in groups; null before anything was looked for. */
		groups: QuickSearchGroup[] | null;
		searching?: boolean;
		failed?: boolean;
		shortcut?: string;
		/** `empty` is said when nothing was found ("Nothing found for “kopi”"). */
		labels: { field: string; groups: Record<QuickSearchGroup['kind'], string>; empty: string; searching: string; failed: string };
		oninput?: (value: string) => void;
		onpick?: (row: QuickSearchRow) => void;
		/** The field, for ⌘K to focus. */
		field?: HTMLInputElement | null;
	};
	let { value = $bindable(''), groups, searching = false, failed = false, shortcut, labels, oninput, onpick, field = $bindable(null) }: Props = $props();

	const id = $props.id();
	const GLYPHS: Record<QuickSearchGroup['kind'], Component<LucideProps>> = { chat: MessageSquare, memory: Brain, file: FileCode };
	let open = $state(false);
	let active = $state(-1);
	const rows = $derived((groups ?? []).flatMap((group) => group.rows));
	const showing = $derived(open && value.trim() !== '');
	const optionId = (index: number) => `${id}-option-${index}`;
	$effect(() => {
		void groups;
		active = -1;
	});

	function pick(row: QuickSearchRow) {
		open = false;
		onpick?.(row);
	}
	function onkeydown(event: KeyboardEvent) {
		if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
			event.preventDefault();
			open = true;
			if (!rows.length) return;
			active = event.key === 'ArrowDown' ? (active + 1) % rows.length : (active - 1 + rows.length) % rows.length;
		} else if (event.key === 'Enter' && active >= 0 && rows[active]) {
			event.preventDefault();
			pick(rows[active]);
		} else if (event.key === 'Escape' && open) {
			event.preventDefault();
			open = false;
		}
	}
</script>

<div class="relative w-60 max-w-full">
	<label data-part="field" class="flex h-7.5 w-full items-center gap-2 rounded-sm border border-border px-2.25 text-text-3 focus-within:border-accent focus-within:shadow-focus">
		<Icon icon={SearchIcon} size={14} />
		<input
			bind:this={field}
			bind:value
			type="text"
			role="combobox"
			placeholder={labels.field}
			aria-label={labels.field}
			aria-expanded={showing}
			aria-controls="{id}-results"
			aria-autocomplete="list"
			aria-activedescendant={showing && active >= 0 ? optionId(active) : undefined}
			autocomplete="off"
			oninput={() => {
				open = true;
				oninput?.(value);
			}}
			onfocus={() => (open = true)}
			onclick={() => (open = true)}
			onblur={() => setTimeout(() => (open = false), 150)}
			{onkeydown}
			class="min-w-0 flex-1 bg-transparent text-ui text-text outline-none placeholder:text-text-3"
		/>
		{#if shortcut && !value}<Kbd keys={shortcut} />{/if}
	</label>
	{#if showing}
		<div id="{id}-results" role="listbox" aria-label={labels.field} class="absolute top-full right-0 z-30 mt-2 flex max-h-110 w-100 flex-col max-sm:fixed max-sm:inset-x-4 max-sm:top-14 max-sm:w-auto gap-1 overflow-y-auto rounded-md bg-surface p-1 text-text shadow-float outline -outline-offset-1 outline-border-strong">
			{#if groups?.length}
				{#each groups as group (group.kind)}
					<div role="group" aria-label={labels.groups[group.kind]} class="flex flex-col">
						<p aria-hidden="true" class="px-2 pt-1.5 pb-1 font-mono text-label tracking-label text-text-3 uppercase">{labels.groups[group.kind]}</p>
						{#each group.rows as row, at (row.id)}
							{@const index = rows.indexOf(row)}
							<a
								id={optionId(index)}
								href={row.href}
								role="option"
								aria-selected={index === active}
								tabindex="-1"
								onmousedown={(event) => event.preventDefault()}
								onclick={(event) => {
									event.preventDefault();
									pick(row);
								}}
								class={['flex items-start gap-2.5 rounded-xs px-2 py-1.75 text-ui text-text hover:bg-surface-2 pointer-coarse:min-h-11', index === active && 'bg-surface-2']}
								data-row={at}
							>
								<span class="mt-px inline-flex shrink-0 text-text-2"><Icon icon={GLYPHS[group.kind]} size={14} /></span>
								<span class="flex min-w-0 flex-1 flex-col gap-0.5">
									<span class={['truncate', group.kind === 'file' && 'font-mono text-label']}>{row.title}</span>
									{#if row.snippet}<span class="truncate text-caption text-text-3">{row.snippet}</span>{/if}
								</span>
								<span class="shrink-0 font-mono text-label text-text-3">{row.meta}</span>
							</a>
						{/each}
					</div>
				{/each}
			{:else}
				<p role="status" class="px-2 py-2 text-ui text-text-3">{failed ? labels.failed : searching ? labels.searching : labels.empty}</p>
			{/if}
		</div>
	{/if}
</div>
