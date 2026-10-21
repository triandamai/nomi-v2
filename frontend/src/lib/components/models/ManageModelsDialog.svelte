<script lang="ts">
	// Manage models: which models the chat's model picker offers, provider by provider. Every
	// switch saves on its own, straight away (`onchange`); a save that fails is put back.
	import { untrack } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import Button from '$lib/components/m3/Button.svelte';
	import Dialog from '$lib/components/m3/Dialog.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import Switch from '$lib/components/m3/Switch.svelte';
	import IconClose from '$lib/components/icons/IconClose.svelte';
	import IconPlus from '$lib/components/icons/IconPlus.svelte';
	import IconSearch from '$lib/components/icons/IconSearch.svelte';
	import ProviderLogo from '$lib/components/providers/ProviderLogo.svelte';
	import { applyChanges, groupChanges, groupToggle, searchModelGroups, type ModelChange, type ModelGroup } from '$lib/modelPicker';

	let {
		open = $bindable(false),
		groups,
		onchange,
		onconnect,
	}: {
		open?: boolean;
		groups: ModelGroup[];
		/** Saves the changes; resolving `false` puts them back. */
		onchange: (changes: ModelChange[]) => void | Promise<boolean>;
		onconnect: () => void;
	} = $props();

	// What's shown, ahead of the server while a save is on its way. Raw: a provider can have
	// hundreds of models, and they're only ever replaced, never mutated.
	let local = $state.raw(untrack(() => groups));
	$effect(() => {
		local = groups;
	});

	let query = $state('');
	let error = $state<string | null>(null);
	const shown = $derived(searchModelGroups(local, query));
	const byId = $derived(new Map(local.map((group) => [group.id, group])));

	// Every opening starts on the whole list.
	let wasOpen = false;
	$effect(() => {
		const opening = open && !wasOpen;
		wasOpen = open;
		if (opening) {
			untrack(() => {
				query = '';
				error = null;
			});
		}
	});

	async function change(changes: ModelChange[]) {
		if (changes.length === 0) return;
		const keys = new Set(changes.map((c) => c.key));
		const before = local.flatMap((group) => group.models.filter((model) => keys.has(model.key)).map((model) => ({ key: model.key, enabled: model.enabled })));
		local = applyChanges(local, changes);
		error = null;
		const saved = await onchange(changes);
		if (saved === false) {
			local = applyChanges(local, before);
			error = m.msel_save_failed();
		}
	}
</script>

<Dialog bind:open class="models-dialog">
	<div class="manage">
		<div class="head">
			<div class="head__text">
				<h2 class="title">{m.msel_manage()}</h2>
				<p class="lede">{m.msel_manage_lede()}</p>
			</div>
			<div class="head__actions">
				<Button type="button" variant="tonal" onclick={onconnect}><IconPlus size={18} /> {m.prov_dialog_title()}</Button>
				<IconButton aria-label={m.common_close()} onclick={() => (open = false)}><IconClose size={20} /></IconButton>
			</div>
		</div>

		<label class="search">
			<IconSearch size={18} />
			<input
				type="text"
				aria-label={m.msel_search()}
				placeholder={m.msel_search()}
				autocomplete="off"
				spellcheck="false"
				bind:value={query}
				onkeydown={(e) => {
					if (e.key === 'Enter') e.preventDefault();
				}}
			/>
		</label>

		{#if error}<p class="error" role="alert">{error}</p>{/if}

		<div class="groups">
			<!-- Element ids from the position: a group's id can hold spaces (a provider's name). -->
			{#each shown as group, index (group.id)}
				{@const whole = byId.get(group.id) ?? group}
				<section class="group" aria-labelledby="models-{index}">
					<div class="group__head">
						<span class="group__logo"><ProviderLogo id={group.provider_id} hasLogo={group.has_logo} size={18} /></span>
						<h3 class="group__name" id="models-{index}" data-group-name>{group.label}</h3>
						<Switch
							checked={groupToggle(whole) !== 'off'}
							disabled={whole.models.length === 0}
							aria-label={m.msel_toggle_group({ provider: group.label })}
							onchange={(on) => change(groupChanges(whole, on))}
						/>
					</div>
					{#if group.models.length > 0}
						<ul class="card">
							{#each group.models as model (model.key)}
								{@const name = model.label || model.model_id}
								<li class="row">
									<span class="row__name" data-model-name title={model.model_id}>{name}</span>
									{#if model.free}<span class="badge">{m.msel_free()}</span>{/if}
									<Switch
										checked={model.enabled}
										aria-label={m.msel_toggle_model({ model: name })}
										onchange={(on) => change([{ key: model.key, enabled: on }])}
									/>
								</li>
							{/each}
						</ul>
					{:else if group.error === 'unreachable'}
						<p class="quiet">{m.msel_unreachable()}</p>
					{/if}
				</section>
			{/each}
			{#if local.length === 0}
				<p class="quiet">{m.msel_dialog_empty()}</p>
			{:else if shown.length === 0}
				<p class="quiet">{m.list_no_match({ query: query.trim() })}</p>
			{/if}
		</div>
	</div>
</Dialog>

<style>
	:global(dialog.m3-dialog.models-dialog) {
		width: min(640px, calc(100vw - 32px));
		max-width: none;
		min-width: 0;
		background: var(--md-sys-color-surface-container-high);
	}
	:global(dialog.m3-dialog.models-dialog .m3-dialog__content) {
		padding: 20px 16px 16px;
	}
	:global(dialog.m3-dialog.models-dialog .m3-dialog__body) {
		color: var(--md-sys-color-on-surface);
	}
	.manage {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.head {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		flex-wrap: wrap;
		gap: 12px;
		padding: 0 4px 0 8px;
	}
	.head__text {
		flex: 1 1 260px;
		min-width: 0;
	}
	.head__actions {
		display: flex;
		align-items: center;
		gap: 4px;
	}
	.title {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.25rem;
		font-weight: 700;
	}
	.lede {
		margin: 4px 0 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.search {
		display: flex;
		align-items: center;
		gap: 10px;
		min-height: 48px;
		margin: 0 4px;
		padding: 0 14px;
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
	.error {
		margin: 0 8px;
		color: var(--md-sys-color-error);
	}
	.groups {
		display: flex;
		flex-direction: column;
		gap: 16px;
		max-height: min(60vh, 560px);
		overflow-y: auto;
		overscroll-behavior: contain;
		padding: 0 4px 4px;
	}
	.group {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.group__head {
		display: flex;
		align-items: center;
		gap: 10px;
		min-height: 44px;
		padding: 0 4px 0 6px;
	}
	.group__logo {
		display: grid;
		flex: none;
		place-items: center;
		width: 30px;
		height: 30px;
		border-radius: 9px;
		background: var(--md-sys-color-surface-container-lowest);
	}
	.group__name {
		flex: 1;
		min-width: 0;
		margin: 0;
		overflow: hidden;
		font-size: 0.9375rem;
		font-weight: 650;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.card {
		margin: 0;
		padding: 0;
		list-style: none;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-lowest);
		overflow: hidden;
	}
	.row {
		display: flex;
		align-items: center;
		gap: 10px;
		min-height: 52px;
		padding: 0 12px 0 16px;
	}
	.row + .row {
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}
	.row__name {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		font-size: 0.875rem;
		text-overflow: ellipsis;
		white-space: nowrap;
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
	.quiet {
		margin: 0;
		padding: 4px 10px;
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	@media (max-width: 480px) {
		.head__actions {
			width: 100%;
			justify-content: space-between;
		}
	}
</style>
