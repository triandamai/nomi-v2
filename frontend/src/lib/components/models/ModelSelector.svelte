<script lang="ts">
	// The composer's model picker: which model this chat replies with. A search on top, the
	// models switched on in Manage models grouped under their providers (plus the chosen one,
	// always), and a way into Manage models at the bottom. Focus stays in the search field; the
	// arrow keys move a highlight across the rows (aria-activedescendant), Enter picks.
	import { tick } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import IconCheck from '$lib/components/icons/IconCheck.svelte';
	import IconChevronDown from '$lib/components/icons/IconChevronDown.svelte';
	import IconSearch from '$lib/components/icons/IconSearch.svelte';
	import IconSliders from '$lib/components/icons/IconSliders.svelte';
	import ProviderLogo from '$lib/components/providers/ProviderLogo.svelte';
	import { flattenModels, searchModelGroups, selectorGroups, type ModelChoice, type ModelGroup } from '$lib/modelPicker';
	import { CUSTOM_PROVIDER_ID, stepHighlight } from '$lib/providers';
	import { touchScreen } from '$lib/touch';

	let {
		groups,
		selected,
		onselect,
		onmanage,
		disabled = false,
		hint,
	}: {
		groups: ModelGroup[];
		selected: ModelChoice | null;
		onselect: (key: string) => void;
		onmanage: () => void;
		disabled?: boolean;
		/** A page's one line on what the pick also means, added to the trigger's tooltip. */
		hint?: string;
	} = $props();

	const uid = $props.id();
	const optionId = (index: number) => `${uid}-opt-${index}`;

	let open = $state(false);
	let query = $state('');
	let highlighted = $state(0);
	let anchorEl: HTMLDivElement | undefined = $state();
	let panelEl: HTMLDivElement | undefined = $state();
	let searchEl: HTMLInputElement | undefined = $state();

	const selectedKey = $derived(selected?.key ?? null);
	const shown = $derived(searchModelGroups(selectorGroups(groups, selectedKey), query));
	const rows = $derived(flattenModels(shown));
	const rowIndex = $derived(new Map(rows.map((row, index) => [row.model.key, index])));
	const activeId = $derived(rows[highlighted] ? optionId(highlighted) : undefined);
	const name = $derived(selected?.label || selected?.model_id || m.msel_choose());
	const tooltip = $derived([selected ? `${selected.provider_label} · ${name}` : name, hint].filter(Boolean).join('\n'));

	async function show() {
		query = '';
		const at = selectedKey ? rowIndex.get(selectedKey) : undefined;
		highlighted = at ?? 0;
		open = true;
		await tick();
		// A manual popover sits in the top layer, so the composer's frosted pane can't clip it.
		if (panelEl && typeof panelEl.showPopover === 'function' && !panelEl.matches(':popover-open')) panelEl.showPopover();
		place();
		// On a phone the keyboard would cover the list: the search is a tap away instead.
		if (!touchScreen()) searchEl?.focus();
		if (activeId) document.getElementById(activeId)?.scrollIntoView?.({ block: 'nearest' });
	}

	function close(refocus = false) {
		if (!open) return;
		if (panelEl && typeof panelEl.hidePopover === 'function' && panelEl.matches(':popover-open')) panelEl.hidePopover();
		open = false;
		if (refocus) anchorEl?.querySelector('button')?.focus();
	}

	const MARGIN = 8;

	// Opens above the trigger (the composer sits at the bottom), below it when there's more room
	// there, left-aligned and kept inside the viewport on every side.
	function place() {
		if (!panelEl || !anchorEl || typeof window === 'undefined') return;
		const anchor = anchorEl.getBoundingClientRect();
		const width = Math.min(380, window.innerWidth - MARGIN * 2);
		const above = anchor.top - MARGIN - 6;
		const below = window.innerHeight - anchor.bottom - MARGIN - 6;
		const up = above >= Math.min(420, below) || above >= below;
		panelEl.style.width = `${width}px`;
		panelEl.style.maxHeight = `${Math.max(160, Math.min(480, up ? above : below))}px`;
		panelEl.style.left = `${Math.max(MARGIN, Math.min(anchor.left, window.innerWidth - width - MARGIN))}px`;
		if (up) {
			panelEl.style.top = 'auto';
			panelEl.style.bottom = `${window.innerHeight - anchor.top + 6}px`;
		} else {
			panelEl.style.bottom = 'auto';
			panelEl.style.top = `${anchor.bottom + 6}px`;
		}
	}

	// A click anywhere else, or the window changing size, while it's open.
	$effect(() => {
		if (!open) return;
		const outside = (event: PointerEvent) => {
			const target = event.target as Node;
			if (!anchorEl?.contains(target) && !panelEl?.contains(target)) close();
		};
		const reposition = () => place();
		document.addEventListener('pointerdown', outside, true);
		window.addEventListener('resize', reposition);
		return () => {
			document.removeEventListener('pointerdown', outside, true);
			window.removeEventListener('resize', reposition);
		};
	});

	function pick(key: string) {
		close(true);
		if (key !== selectedKey) onselect(key);
	}

	function onSearchKey(event: KeyboardEvent) {
		if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
			event.preventDefault();
			highlighted = stepHighlight(highlighted, rows.length, event.key);
			if (activeId) document.getElementById(activeId)?.scrollIntoView?.({ block: 'nearest' });
		} else if (event.key === 'Enter') {
			// Never the composer's form: Enter here picks a model, it doesn't send.
			event.preventDefault();
			const row = rows[highlighted];
			if (row) pick(row.model.key);
		}
	}

	function onPanelKey(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			event.preventDefault();
			event.stopPropagation();
			close(true);
		}
	}

	function onFocusOut(event: FocusEvent) {
		const next = event.relatedTarget as Node | null;
		if (next && !anchorEl?.contains(next) && !panelEl?.contains(next)) close();
	}
</script>

<div class="selector" bind:this={anchorEl} onfocusout={onFocusOut}>
	<button
		type="button"
		class="trigger"
		aria-haspopup="listbox"
		aria-expanded={open}
		aria-controls="{uid}-panel"
		aria-label={selected ? m.msel_trigger_label({ model: name }) : name}
		title={tooltip}
		{disabled}
		onclick={() => (open ? close() : show())}
	>
		<ProviderLogo id={selected?.provider_id ?? CUSTOM_PROVIDER_ID} hasLogo={selected?.has_logo ?? false} size={16} />
		<span class="trigger__label">{name}</span>
		<IconChevronDown size={16} />
	</button>

	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div bind:this={panelEl} id="{uid}-panel" class="panel" popover="manual" onkeydown={onPanelKey}>
		{#if open}
			<label class="search">
				<IconSearch size={18} />
				<input
					bind:this={searchEl}
					type="text"
					role="combobox"
					aria-label={m.msel_search()}
					aria-expanded="true"
					aria-controls="{uid}-list"
					aria-autocomplete="list"
					aria-activedescendant={activeId}
					placeholder={m.msel_search()}
					autocomplete="off"
					spellcheck="false"
					bind:value={query}
					oninput={() => (highlighted = 0)}
					onkeydown={onSearchKey}
				/>
			</label>
			<div class="list" id="{uid}-list" role="listbox" aria-label={m.msel_list_label()}>
				<!-- Element ids from the position: a group's id can hold spaces (a provider's name). -->
				{#each shown as group, index (group.id)}
					<div class="group" role="group" aria-labelledby="{uid}-group-{index}">
						<p class="group__label" id="{uid}-group-{index}" data-group-label>{group.label}</p>
						{#each group.models as model (model.key)}
							{@const index = rowIndex.get(model.key) ?? -1}
							{@const chosen = model.key === selectedKey}
							<!-- Focus stays in the search field (aria-activedescendant); the mouse picks here. -->
							<div
								id={optionId(index)}
								role="option"
								tabindex="-1"
								aria-selected={chosen}
								class="option"
								class:option--active={index === highlighted}
								onmousedown={(e) => e.preventDefault()}
								onclick={() => pick(model.key)}
								onkeydown={() => {}}
								onmouseenter={() => (highlighted = index)}
							>
								<span class="option__name" data-name>{model.label || model.model_id}</span>
								{#if model.free}<span class="badge">{m.msel_free()}</span>{/if}
								{#if chosen}<span class="option__check" data-check><IconCheck size={18} /></span>{/if}
							</div>
						{/each}
					</div>
				{/each}
				{#if rows.length === 0}
					<p class="empty">{query.trim() ? m.list_no_match({ query: query.trim() }) : m.msel_none()}</p>
				{/if}
			</div>
			<button
				type="button"
				class="manage"
				onclick={() => {
					close();
					onmanage();
				}}
			>
				<IconSliders size={18} />
				{m.msel_manage()}
			</button>
		{/if}
	</div>
</div>

<style>
	.selector {
		position: relative;
		display: inline-flex;
		flex: 0 1 auto;
		min-width: 0;
	}
	.trigger {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		min-width: 0;
		height: 40px;
		padding: 0 10px 0 12px;
		border: none;
		border-radius: 20px;
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface);
		font: inherit;
		font-size: 0.8125rem;
		font-weight: 650;
		cursor: pointer;
		transition: border-radius var(--nomi-motion-spatial-fast);
	}
	.trigger:hover:not(:disabled) {
		border-radius: var(--md-sys-shape-corner-medium);
	}
	.trigger:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.trigger:disabled {
		opacity: 0.38;
		cursor: not-allowed;
	}
	.trigger__label {
		min-width: 0;
		max-width: 180px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	@media (pointer: coarse) {
		.trigger {
			height: 44px;
			border-radius: 22px;
		}
	}
	/* A narrow composer (a phone, a project's side column): the selector takes only the room the
	   other controls leave, so Send never wraps onto a line of its own; the name truncates. */
	@container (max-width: 480px) {
		.selector {
			flex: 1 1 0;
		}
		.trigger {
			width: 100%;
		}
		.trigger__label {
			flex: 1 1 auto;
			max-width: none;
			text-align: left;
		}
	}
	.panel {
		position: fixed;
		inset: auto;
		margin: 0;
		padding: 0;
		width: min(380px, calc(100vw - 16px));
		max-height: min(480px, 70vh);
		overflow: hidden;
		flex-direction: column;
		border: none;
		border-radius: var(--md-sys-shape-corner-large-increased);
		background: var(--md-sys-color-surface-container);
		color: var(--md-sys-color-on-surface);
		box-shadow: var(--md-sys-elevation-shadow-level2);
	}
	.panel:popover-open {
		display: flex;
	}
	.search {
		flex: none;
		display: flex;
		align-items: center;
		gap: 10px;
		min-height: 44px;
		margin: 8px 8px 4px;
		padding: 0 12px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface-variant);
	}
	.search:focus-within {
		outline: 2px solid var(--md-sys-color-primary);
	}
	.search input {
		flex: 1;
		min-width: 0;
		border: 0;
		background: transparent;
		color: var(--md-sys-color-on-surface);
		font: inherit;
		outline: none;
	}
	.list {
		flex: 1 1 auto;
		min-height: 0;
		overflow-y: auto;
		overscroll-behavior: contain;
		padding: 0 6px 6px;
	}
	.group + .group {
		margin-top: 6px;
	}
	.group__label {
		margin: 0;
		padding: 8px 10px 4px;
		font-size: 0.75rem;
		font-weight: 650;
		color: var(--md-sys-color-on-surface-variant);
	}
	.option {
		display: flex;
		align-items: center;
		gap: 8px;
		min-height: 40px;
		padding: 0 10px;
		border-radius: var(--md-sys-shape-corner-medium);
		cursor: pointer;
	}
	.option--active {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	@media (pointer: coarse) {
		.option {
			min-height: 44px;
		}
	}
	.option__name {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-size: 0.875rem;
	}
	.option__check {
		display: inline-flex;
		flex: none;
		color: var(--md-sys-color-primary);
	}
	.option--active .option__check {
		color: inherit;
	}
	.badge {
		flex: none;
		padding: 1px 8px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-tertiary-container);
		color: var(--md-sys-color-on-tertiary-container);
		font-size: 0.6875rem;
		font-weight: 650;
	}
	.empty {
		margin: 0;
		padding: 12px 10px;
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.manage {
		flex: none;
		display: flex;
		align-items: center;
		gap: 10px;
		min-height: 48px;
		padding: 0 16px;
		border: none;
		border-top: 1px solid var(--md-sys-color-outline-variant);
		background: transparent;
		color: var(--md-sys-color-on-surface);
		font: inherit;
		font-size: 0.875rem;
		font-weight: 600;
		text-align: left;
		cursor: pointer;
	}
	.manage:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
	.manage:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: -2px;
	}
</style>
