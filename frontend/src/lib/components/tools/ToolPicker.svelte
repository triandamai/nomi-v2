<script lang="ts">
	// Picks the tools a crew member can use from the catalog: type to narrow the list (by name,
	// keyword or description), pick with the arrow keys and Enter or a tap; picked tools sit above as
	// chips you can remove (a tap, or Backspace in the empty field). A multi-select combobox: focus
	// stays in the field and `aria-activedescendant` points at the highlighted option.
	import { X } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import M3Chip from '$lib/components/m3/Chip.svelte';
	import Icon from '$lib/components/ds/foundations/Icon.svelte';
	import { nextOption } from '$lib/components/ds/chat/mention';
	import { addTool, filterTools, removeLastTool, removeTool, toolDescription } from '$lib/toolPicker';
	import type { CatalogTool } from '$lib/types';

	let {
		options,
		selected = $bindable([]),
		disabled = false,
		label,
		name,
	}: {
		/** The catalog's tools. */
		options: CatalogTool[];
		/** The picked tools' names. */
		selected: string[];
		disabled?: boolean;
		label: string;
		/** Posts the picked tools in a form, one hidden field of this name each. */
		name?: string;
	} = $props();

	const uid = $props.id();
	let query = $state('');
	let open = $state(false);
	let active = $state(0);
	let input: HTMLInputElement | undefined = $state();

	const matches = $derived(filterTools(options, query, toolDescription));
	const listId = $derived(`${uid}-list`);

	function toggle(tool: string) {
		selected = selected.includes(tool) ? removeTool(selected, tool) : addTool(selected, tool);
	}

	function onkeydown(event: KeyboardEvent) {
		if (event.isComposing || event.keyCode === 229) return;
		if (['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) {
			// Home and End move the caret in the field until the list is open.
			if (!open) {
				if (event.key !== 'ArrowDown' && event.key !== 'ArrowUp') return;
				open = true;
			} else {
				active = nextOption(active, event.key, matches.length);
			}
		} else if (event.key === 'Enter') {
			// Enter inside a form would submit it; here it picks.
			event.preventDefault();
			if (open && matches[active]) toggle(matches[active].name);
			return;
		} else if (event.key === 'Escape') {
			if (!open && !query) return;
			open = false;
			query = '';
		} else if (event.key === 'Backspace' && query === '' && selected.length > 0) {
			selected = removeLastTool(selected);
			return;
		} else {
			return;
		}
		event.preventDefault();
		event.stopPropagation();
	}
</script>

<div class="tool-picker">
	<label class="tool-picker__label" for="{uid}-input">{label}</label>
	{#if name}
		{#each selected as tool (tool)}<input type="hidden" {name} value={tool} />{/each}
	{/if}
	<div class="tool-picker__box" class:tool-picker__box--disabled={disabled}>
		{#each selected as tool (tool)}
			<M3Chip {disabled} aria-label={m.toolpicker_remove({ name: tool })} onclick={() => {
					selected = removeTool(selected, tool);
					input?.focus();
				}}>
				<span class="tool-picker__chip">{tool}</span>
				<Icon icon={X} size={16} />
			</M3Chip>
		{/each}
		<input
			bind:this={input}
			id="{uid}-input"
			type="text"
			role="combobox"
			autocomplete="off"
			aria-autocomplete="list"
			aria-expanded={open}
			aria-controls={listId}
			aria-activedescendant={open && matches.length > 0 ? `${uid}-opt-${active}` : undefined}
			placeholder={m.toolpicker_placeholder()}
			{disabled}
			bind:value={query}
			oninput={() => {
				open = true;
				active = 0;
			}}
			onfocus={() => (open = true)}
			onblur={() => {
				open = false;
				query = '';
			}}
			{onkeydown}
		/>
	</div>
	{#if open}
		<ul class="tool-picker__list" id={listId} role="listbox" aria-multiselectable="true" aria-label={label}>
			{#each matches as tool, index (tool.name)}
				<!-- The keyboard drives the list from the field (a combobox), so the options take only the pointer. -->
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<li
					id="{uid}-opt-{index}"
					role="option"
					class="tool-picker__option"
					class:tool-picker__option--hi={index === active}
					aria-selected={selected.includes(tool.name)}
					onmousedown={(event) => event.preventDefault()}
					onmouseenter={() => (active = index)}
					onclick={() => toggle(tool.name)}
				>
					<span class="tool-picker__head">
						<span class="tool-picker__name">{tool.name}</span>
						<span class="tool-picker__group">{tool.group}</span>
					</span>
					<span class="tool-picker__what">{toolDescription(tool)}</span>
				</li>
			{:else}
				<li class="tool-picker__none">{m.toolpicker_none()}</li>
			{/each}
		</ul>
	{/if}
	<span class="sr-only" role="status" aria-live="polite">{open ? m.toolpicker_count({ count: matches.length }) : ''}</span>
</div>

<style>
	.tool-picker {
		display: flex;
		flex-direction: column;
		gap: 4px;
		min-width: 0;
	}
	.tool-picker__label {
		font-size: var(--md-sys-typescale-body-small-size);
		color: var(--md-sys-color-on-surface-variant);
	}
	.tool-picker__box {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 6px;
		min-height: 52px;
		padding: 8px 12px;
		box-sizing: border-box;
		border: 1.5px solid var(--md-sys-color-outline);
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface);
		transition: background-color var(--nomi-motion-effects-fast);
	}
	.tool-picker__box:hover:not(.tool-picker__box--disabled) {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 4%, var(--md-sys-color-surface));
	}
	.tool-picker__box:focus-within {
		border-color: var(--md-sys-color-primary);
		outline: 1px solid var(--md-sys-color-primary);
	}
	.tool-picker__box--disabled {
		opacity: 0.6;
	}
	.tool-picker__chip {
		font-family: var(--md-ref-typeface-mono);
		font-size: var(--md-sys-typescale-label-medium-size);
	}
	.tool-picker__box input {
		flex: 1 1 120px;
		min-width: 0;
		height: 36px;
		border: none;
		background: none;
		color: var(--md-sys-color-on-surface);
		font: inherit;
		font-size: var(--md-sys-typescale-body-large-size);
		outline: none;
	}
	.tool-picker__list {
		max-height: 280px;
		margin: 0;
		padding: 4px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 12px;
		background: var(--md-sys-color-surface-container);
		list-style: none;
		overflow-y: auto;
	}
	.tool-picker__option {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-height: 48px;
		padding: 6px 10px;
		border-radius: 8px;
		cursor: pointer;
	}
	.tool-picker__option--hi {
		background: var(--md-sys-color-surface-container-highest);
	}
	.tool-picker__option[aria-selected='true'] .tool-picker__name {
		color: var(--md-sys-color-primary);
	}
	.tool-picker__option[aria-selected='true'] .tool-picker__name::before {
		content: '✓ ';
	}
	.tool-picker__head {
		display: flex;
		align-items: baseline;
		gap: 8px;
	}
	.tool-picker__name {
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--md-sys-color-on-surface);
	}
	.tool-picker__group {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.75rem;
	}
	.tool-picker__what {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.tool-picker__none {
		padding: 12px 10px;
		color: var(--md-sys-color-on-surface-variant);
	}
	@media (pointer: coarse) {
		.tool-picker__option {
			min-height: 52px;
		}
	}
</style>
