<script lang="ts">
	// Someone's admin role: the areas of the console they can use, each at Read, Write or Manage.
	// Areas are added by searching for them; the whole role is saved at once. People can only
	// give (or take away) access they hold themselves, so anything above that is disabled.
	import { m } from '$lib/paraglide/messages';
	import { adminApi } from '$lib/adminApi';
	import Button from '$lib/components/m3/Button.svelte';
	import ButtonGroup from '$lib/components/m3/ButtonGroup.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import IconClose from '$lib/components/icons/IconClose.svelte';
	import IconSearch from '$lib/components/icons/IconSearch.svelte';
	import { ADMIN_AREAS, LEVELS, LEVEL_HINTS, LEVEL_LABELS, areaFor, can, highest, searchAreas, type Level } from '$lib/permissions';
	import type { AdminUserDetail } from '$lib/types';

	let {
		user,
		mine,
		editable,
		onsaved,
	}: {
		user: AdminUserDetail;
		/** The access of whoever is editing: the ceiling for what they can hand out. */
		mine: Record<string, Level>;
		editable: boolean;
		onsaved: (permissions: AdminUserDetail['permissions']) => void;
	} = $props();

	type Row = { resource: string; level: Level };

	function fromGrants(grants: AdminUserDetail['permissions']): Row[] {
		const rows: Row[] = [];
		for (const grant of grants) {
			if (grant.scope_type !== 'admin' || !areaFor(grant.resource)) continue;
			const level = highest(grant.actions);
			if (level) rows.push({ resource: grant.resource, level });
		}
		return ADMIN_AREAS.flatMap((area) => rows.filter((r) => r.resource === area.resource));
	}

	let rows = $state<Row[]>([]);
	let saved = $state<Row[]>([]);
	$effect(() => {
		rows = fromGrants(user.permissions);
		saved = rows.map((r) => ({ ...r }));
	});

	const dirty = $derived(JSON.stringify(rows) !== JSON.stringify(saved));
	// An area they hold above the editor's own level can't be changed by the editor at all.
	const locked = (row: Row) => !editable || !can(mine, row.resource, row.level);

	let query = $state('');
	let listOpen = $state(false);
	let highlighted = $state(0);
	const suggestions = $derived(searchAreas(query).filter((area) => !rows.some((r) => r.resource === area.resource)));

	function add(resource: string) {
		if (!can(mine, resource)) return;
		rows = ADMIN_AREAS.flatMap((area) =>
			area.resource === resource ? [{ resource, level: 'view' as Level }] : rows.filter((r) => r.resource === area.resource),
		);
		query = '';
		highlighted = 0;
	}

	function onKey(event: KeyboardEvent) {
		if (event.key === 'ArrowDown') {
			event.preventDefault();
			listOpen = true;
			highlighted = Math.min(highlighted + 1, suggestions.length - 1);
		} else if (event.key === 'ArrowUp') {
			event.preventDefault();
			highlighted = Math.max(highlighted - 1, 0);
		} else if (event.key === 'Enter') {
			const pick = suggestions[highlighted];
			if (pick && can(mine, pick.resource)) {
				event.preventDefault();
				add(pick.resource);
			}
		} else if (event.key === 'Escape') {
			listOpen = false;
		}
	}

	let saving = $state(false);
	let error = $state<string | null>(null);
	let done = $state(false);

	async function save() {
		saving = true;
		error = null;
		done = false;
		try {
			const permissions = await adminApi<AdminUserDetail['permissions']>(`users/${user.id}/admin-access`, {
				method: 'PUT',
				body: { access: rows },
			});
			saved = rows.map((r) => ({ ...r }));
			done = true;
			onsaved(permissions);
		} catch (e) {
			error = (e as Error).message;
		} finally {
			saving = false;
		}
	}

	const levelOptions = (resource: string) => LEVELS.map((level) => ({ value: level, label: LEVEL_LABELS[level](), disabled: !can(mine, resource, level) }));
</script>

<section class="role">
	{#if user.is_platform_admin}
		<p class="note">{m.role_platform_admin()}</p>
	{/if}

	{#if editable}
		<div class="search">
			<label class="search__field">
				<IconSearch size={18} />
				<input
					type="text"
					role="combobox"
					aria-expanded={listOpen && suggestions.length > 0}
					aria-controls="role-areas"
					aria-autocomplete="list"
					aria-activedescendant={listOpen && suggestions[highlighted] ? `area-${suggestions[highlighted].resource}` : undefined}
					placeholder={m.role_search()}
					bind:value={query}
					onfocus={() => (listOpen = true)}
					onblur={() => setTimeout(() => (listOpen = false), 120)}
					oninput={() => {
						listOpen = true;
						highlighted = 0;
					}}
					onkeydown={onKey}
				/>
			</label>
			{#if listOpen && suggestions.length > 0}
				<ul id="role-areas" class="search__list" role="listbox" aria-label={m.role_areas()}>
					{#each suggestions as area, i (area.resource)}
						<li
							id="area-{area.resource}"
							role="option"
							aria-selected={i === highlighted}
							aria-disabled={!can(mine, area.resource)}
							class="option"
							class:option--active={i === highlighted}
							onmousedown={(e) => {
								e.preventDefault();
								add(area.resource);
							}}
							onmouseenter={() => (highlighted = i)}
						>
							<span class="option__label">{area.label()} <code>{area.resource}</code></span>
							<span class="option__desc">{can(mine, area.resource) ? area.description() : m.role_not_yours()}</span>
						</li>
					{/each}
				</ul>
			{:else if listOpen && query.trim()}
				<p class="search__empty">{m.role_no_match({ query: query.trim() })}</p>
			{/if}
		</div>
	{/if}

	{#if rows.length === 0}
		<p class="note">{m.role_none()}</p>
	{:else}
		<ul class="rows">
			{#each rows as row (row.resource)}
				{@const area = areaFor(row.resource)}
				<li class="row">
					<div class="row__head">
						<span class="row__text">
							<span class="row__label">{area?.label() ?? row.resource}</span>
							<span class="row__desc">{LEVEL_HINTS[row.level]()}</span>
						</span>
						{#if !locked(row)}
							<IconButton
								aria-label={m.role_remove({ area: area?.label() ?? row.resource })}
								onclick={() => (rows = rows.filter((r) => r.resource !== row.resource))}
							>
								<IconClose size={18} />
							</IconButton>
						{/if}
					</div>
					{#if locked(row)}
						<span class="row__fixed">{LEVEL_LABELS[row.level]()}</span>
					{:else}
						<ButtonGroup
							options={levelOptions(row.resource)}
							bind:value={row.level}
							aria-label={m.role_level_for({ area: area?.label() ?? row.resource })}
						/>
					{/if}
				</li>
			{/each}
		</ul>
	{/if}

	{#if editable}
		{#if error}<p class="error" role="alert">{error}</p>{/if}
		{#if done && !dirty}<p class="ok" role="status">{m.role_saved()}</p>{/if}
		<div class="actions">
			<Button variant="filled" disabled={!dirty || saving} onclick={save}>{m.role_save()}</Button>
			{#if dirty}
				<Button variant="text" onclick={() => (rows = saved.map((r) => ({ ...r })))}>{m.common_cancel()}</Button>
			{/if}
		</div>
	{:else}
		<p class="note">{m.users_need_manage()}</p>
	{/if}
</section>

<style>
	.role {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}
	.note {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.search {
		position: relative;
	}
	.search__field {
		display: flex;
		align-items: center;
		gap: 10px;
		min-height: 48px;
		padding: 0 14px;
		border: 1px solid var(--md-sys-color-outline);
		border-radius: var(--md-sys-shape-corner-full);
		color: var(--md-sys-color-on-surface-variant);
	}
	.search__field:focus-within {
		border-color: var(--md-sys-color-primary);
		outline: 1px solid var(--md-sys-color-primary);
	}
	.search__field input {
		flex: 1;
		min-width: 0;
		border: none;
		background: transparent;
		color: var(--md-sys-color-on-surface);
		font: inherit;
		font-size: 1rem;
		outline: none;
	}
	.search__list {
		position: absolute;
		top: calc(100% + 4px);
		right: 0;
		left: 0;
		z-index: 5;
		max-height: 300px;
		margin: 0;
		padding: 6px;
		overflow-y: auto;
		list-style: none;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-high);
		box-shadow: var(--md-sys-elevation-level2);
	}
	.search__empty {
		margin: 8px 4px 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.option {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-height: 44px;
		padding: 8px 12px;
		border-radius: var(--md-sys-shape-corner-medium);
		cursor: pointer;
	}
	.option--active {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
	.option[aria-disabled='true'] {
		cursor: not-allowed;
		opacity: 0.55;
	}
	.option__label {
		color: var(--md-sys-color-on-surface);
		font-weight: 600;
	}
	.option__label code {
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
		font-weight: 400;
	}
	.option__desc {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.rows {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.row {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 12px 12px 14px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container);
	}
	.row__head {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.row__text {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.row__label {
		color: var(--md-sys-color-on-surface);
		font-weight: 700;
	}
	.row__desc {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.row__fixed {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
		font-weight: 600;
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}
	.error {
		margin: 0;
		color: var(--md-sys-color-error);
	}
	.ok {
		margin: 0;
		color: var(--md-sys-color-primary);
		font-weight: 600;
	}
</style>
