<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import Button from '$lib/components/m3/Button.svelte';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Dialog from '$lib/components/m3/Dialog.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import List from '$lib/components/m3/List.svelte';
	import ListItem from '$lib/components/m3/ListItem.svelte';
	import LoadingIndicator from '$lib/components/m3/LoadingIndicator.svelte';
	import Avatar from '$lib/components/m3/Avatar.svelte';
	import Switch from '$lib/components/m3/Switch.svelte';
	import IconCheck from '$lib/components/icons/IconCheck.svelte';
	import IconGroup from '$lib/components/icons/IconGroup.svelte';
	import { projectApi } from '$lib/projectApi';
	import ImportCode from '$lib/components/project/ImportCode.svelte';
	import type { ProjectDeploy } from '$lib/projectDeploy.svelte';
	import { formatTransferNoticeParams } from '$lib/projectTransfer';
	import type { TransferOrgSummary } from '$lib/types';

	let { deploy, projectName }: { deploy: ProjectDeploy; projectName: string } = $props();

	const isAdmin = $derived(deploy.isAdmin);
	const live = $derived(Boolean(deploy.overview?.live));

	let offlineOpen = $state(false);
	let deleteOpen = $state(false);
	let typed = $state('');
	let deleting = $state(false);
	let error = $state<string | null>(null);

	// Transfer ownership state
	let transferSheetOpen = $state(false);
	let searchQuery = $state('');
	let orgResults = $state<TransferOrgSummary[]>([]);
	let isSearching = $state(false);
	let selectedOrg = $state<TransferOrgSummary | null>(null);
	let transferring = $state(false);
	let transferError = $state<string | null>(null);
	// Picking the organization, then confirming by typing the project's name.
	let transferStep = $state<'pick' | 'confirm'>('pick');
	let transferTyped = $state('');
	let debounceTimer: ReturnType<typeof setTimeout> | undefined;

	async function loadOrgs(query: string) {
		isSearching = true;
		transferError = null;
		try {
			const res = await projectApi<TransferOrgSummary[]>(
				deploy.projectId,
				`transfer/orgs?q=${encodeURIComponent(query)}`
			);
			orgResults = res ?? [];
		} catch (e) {
			transferError = (e as Error).message;
			orgResults = [];
		} finally {
			isSearching = false;
		}
	}

	function onSearchInput(val: string) {
		searchQuery = val;
		clearTimeout(debounceTimer);
		debounceTimer = setTimeout(() => {
			loadOrgs(val);
		}, 300);
	}

	function openTransferSheet() {
		searchQuery = '';
		selectedOrg = null;
		transferError = null;
		transferStep = 'pick';
		transferTyped = '';
		transferSheetOpen = true;
		loadOrgs('');
	}

	async function executeTransfer() {
		if (!selectedOrg || transferTyped.trim() !== projectName.trim()) return;
		transferring = true;
		transferError = null;
		try {
			await projectApi(deploy.projectId, 'transfer', {
				method: 'POST',
				body: { target_org_id: selectedOrg.id, confirm_name: transferTyped.trim() }
			});
			location.href = '/projects';
		} catch (e) {
			transferError = (e as Error).message;
			transferring = false;
		}
	}

	// The showcase: whether this app is on Nomi's public showcase page, with one line about it.
	interface Showcase { showcase: boolean; blurb: string | null; url: string | null; can_edit: boolean }
	const BLURB_MAX = 140;
	let showcase = $state<Showcase | null>(null);
	let blurb = $state('');
	let showcaseBusy = $state(false);
	let showcaseError = $state<string | null>(null);
	let showcaseSaved = $state(false);
	$effect(() => {
		const id = deploy.projectId;
		projectApi<Showcase>(id, 'showcase')
			.then((loaded) => {
				showcase = loaded;
				blurb = loaded.blurb ?? '';
			})
			.catch((e) => (showcaseError = (e as Error).message));
	});
	const blurbChanged = $derived((showcase?.blurb ?? '') !== blurb.trim());

	async function saveShowcase(on: boolean) {
		if (!showcase) return;
		showcaseBusy = true;
		showcaseError = null;
		showcaseSaved = false;
		try {
			showcase = await projectApi<Showcase>(deploy.projectId, 'showcase', { method: 'PUT', body: { showcase: on, blurb: blurb.trim() || null } });
			blurb = showcase.blurb ?? '';
			showcaseSaved = true;
		} catch (e) {
			showcaseError = (e as Error).message;
		} finally {
			showcaseBusy = false;
		}
	}

	async function remove() {
		deleting = true;
		error = null;
		try {
			await projectApi(deploy.projectId, '', { method: 'DELETE' });
			location.href = '/projects';
		} catch (e) {
			error = (e as Error).message;
			deleting = false;
		}
	}
</script>

<section class="block">
	<div class="block__row">
		<div class="block__text">
			<h3 class="block__title">{m.showcase_setting()}</h3>
			<p class="block__hint">
				{#if showcase && !showcase.url}{m.showcase_setting_unpublished()}{:else}{m.showcase_setting_hint()}{/if}
				<a href="/showcase" target="_blank" rel="noopener">{m.showcase_see()}</a>
			</p>
		</div>
		<Switch
			checked={showcase?.showcase ?? false}
			disabled={!showcase || !showcase.can_edit || !showcase.url || showcaseBusy}
			aria-label={m.showcase_setting()}
			onchange={(on) => saveShowcase(on)}
		/>
	</div>
	{#if showcase?.url && showcase.can_edit}
		<TextField
			name="showcase_blurb"
			label={m.showcase_blurb()}
			bind:value={blurb}
			maxlength={BLURB_MAX}
			supportingText={m.showcase_blurb_hint({ count: blurb.length, max: BLURB_MAX })}
		/>
		{#if blurbChanged}
			<div class="block__actions">
				<Button variant="tonal" size="s" disabled={showcaseBusy} onclick={() => saveShowcase(showcase?.showcase ?? false)}>{m.showcase_save_blurb()}</Button>
			</div>
		{/if}
	{:else if showcase && !showcase.can_edit}
		<p class="block__hint">{m.pj_admins_only()}</p>
	{/if}
	{#if showcaseError}<p class="error" role="alert">{showcaseError}</p>{/if}
	{#if showcaseSaved && !showcaseError}<p class="block__hint" role="status">{showcase?.showcase ? m.showcase_on() : m.showcase_off()}</p>{/if}
</section>

<section class="block">
	<h3 class="block__title">{m.import_title()}</h3>
	<p class="block__hint">{m.import_lede()}</p>
	<ImportCode projectId={deploy.projectId} />
</section>

{#if !isAdmin}
	<p class="muted">{m.pj_admins_only()}</p>
{:else}
	<section class="danger">
		<div class="danger__row">
			<div>
				<h3 class="danger__title">{m.pj_offline()}</h3>
				<p class="muted">{m.pj_offline_hint()}</p>
			</div>
			<Button variant="outlined" size="s" disabled={!live || deploy.busy} onclick={() => (offlineOpen = true)}>{m.pj_offline()}</Button>
		</div>
		<div class="danger__row">
			<div>
				<h3 class="danger__title">{m.pj_transfer()}</h3>
				<p class="muted">{m.pj_transfer_hint()}</p>
			</div>
			<Button variant="outlined" size="s" onclick={openTransferSheet}>{m.pj_transfer()}</Button>
		</div>
		<div class="danger__row">
			<div>
				<h3 class="danger__title">{m.pj_delete()}</h3>
				<p class="muted">{m.pj_delete_hint()}</p>
			</div>
			<Button variant="filled" size="s" class="danger-button" onclick={() => { typed = ''; deleteOpen = true; }}>{m.pj_delete()}</Button>
		</div>
	</section>
	{#if error || deploy.error}<p class="error" role="alert">{error ?? deploy.error}</p>{/if}
{/if}

<Dialog bind:open={offlineOpen} headline={m.pj_offline()}>
	<p>{m.pj_offline_confirm()}</p>
	{#snippet actions()}
		<Button variant="text" type="button" onclick={() => (offlineOpen = false)}>{m.issues_cancel()}</Button>
		<Button variant="filled" type="button" onclick={async () => { offlineOpen = false; await deploy.takeOffline(); }}>{m.pj_offline()}</Button>
	{/snippet}
</Dialog>

<Dialog bind:open={deleteOpen} headline={m.pj_delete()}>
	<p>{m.pj_delete_confirm({ name: projectName })}</p>
	<TextField name="confirm" label={m.pj_delete_type()} bind:value={typed} />
	{#snippet actions()}
		<Button variant="text" type="button" onclick={() => (deleteOpen = false)}>{m.issues_cancel()}</Button>
		<Button variant="filled" type="button" disabled={typed.trim() !== projectName || deleting} onclick={remove}>{m.pj_delete()}</Button>
	{/snippet}
</Dialog>

<BottomSheet bind:open={transferSheetOpen} label={m.pj_transfer_title()} class="transfer-bottom-sheet">
	<div class="transfer-sheet">
		<div class="transfer-sheet__head">
			<div class="transfer-sheet__icon" aria-hidden="true">
				<IconGroup size={24} />
			</div>
			<div>
				<h2 class="md-headline-small-emphasized" style="margin: 0; color: var(--md-sys-color-on-surface)">
					{m.pj_transfer_title()}
				</h2>
				<p class="md-body-medium" style="margin: 4px 0 0; color: var(--md-sys-color-on-surface-variant)">
					{m.pj_transfer_hint()}
				</p>
			</div>
		</div>

		{#if transferStep === 'pick'}
		<div class="transfer-sheet__search">
			<TextField
				name="org_search"
				label={m.pj_transfer_search_placeholder()}
				value={searchQuery}
				oninput={(e) => onSearchInput((e.target as HTMLInputElement).value)}
				autocomplete="off"
			/>
		</div>

		{#if isSearching}
			<div class="transfer-sheet__loading">
				<LoadingIndicator size={36} label={m.pj_transfer_title()} />
			</div>
		{:else if orgResults.length === 0}
			<div class="transfer-sheet__empty">
				<p class="md-body-medium" style="margin: 0; color: var(--md-sys-color-on-surface-variant)">
					{m.pj_transfer_no_orgs()}
				</p>
			</div>
		{:else}
			<div class="transfer-sheet__list-scroll">
				<List>
					{#each orgResults as org (org.id)}
						<ListItem
							headline={org.name}
							supportingText={m.pj_transfer_owner({ name: org.owner_name ?? '—' })}
							selected={selectedOrg?.id === org.id}
							onclick={() => (selectedOrg = org)}
							label={org.name}
						>
							{#snippet leading()}
								<Avatar name={org.name} size={36} />
							{/snippet}
							{#snippet trailing()}
								<div class="transfer-sheet__trailing">
									{#if org.is_personal}
										<span class="m3-badge">Personal</span>
									{/if}
									{#if org.my_role}
										<span class="m3-badge m3-badge--subtle">{org.my_role}</span>
									{/if}
									{#if selectedOrg?.id === org.id}
										<span class="transfer-sheet__check" aria-hidden="true">
											<IconCheck size={20} />
										</span>
									{/if}
								</div>
							{/snippet}
						</ListItem>
					{/each}
				</List>
			</div>
		{/if}

		{#if selectedOrg}
			<div class="transfer-sheet__notice" role="status">
				<p class="md-body-medium" style="margin: 0;">
					{m.pj_transfer_confirm_notice(formatTransferNoticeParams(selectedOrg))}
				</p>
			</div>
		{/if}

		{#if transferError}
			<p class="error md-body-medium" role="alert">{transferError}</p>
		{/if}

		<div class="transfer-sheet__actions">
			<Button variant="text" type="button" onclick={() => (transferSheetOpen = false)}>
				{m.issues_cancel()}
			</Button>
			<Button variant="filled" type="button" disabled={!selectedOrg} onclick={() => { transferTyped = ''; transferError = null; transferStep = 'confirm'; }}>
				{m.pj_transfer_next()}
			</Button>
		</div>
		{:else if selectedOrg}
			<form
				class="transfer-sheet__confirm"
				onsubmit={(e) => {
					e.preventDefault();
					void executeTransfer();
				}}
			>
				<p class="md-body-large" style="margin: 0;">
					{m.pj_transfer_confirm_notice(formatTransferNoticeParams(selectedOrg))}
				</p>
				<p class="md-body-medium" style="margin: 0; color: var(--md-sys-color-on-surface-variant)">
					{m.pj_transfer_owner_told({ org: selectedOrg.name })}
				</p>
				{#if deploy.overview?.app}
					<p class="md-body-medium" style="margin: 0; color: var(--md-sys-color-on-surface-variant)">
						{m.pj_transfer_shipyard({ org: selectedOrg.name })}
					</p>
				{/if}
				<TextField name="confirm_transfer" label={m.pj_transfer_type({ name: projectName })} bind:value={transferTyped} autocomplete="off" />
				{#if transferError}
					<p class="error md-body-medium" role="alert">{transferError}</p>
				{/if}
				<div class="transfer-sheet__actions">
					<Button variant="text" type="button" onclick={() => (transferStep = 'pick')}>{m.pj_transfer_back()}</Button>
					<Button variant="filled" type="submit" disabled={transferTyped.trim() !== projectName.trim() || transferring}>
						{transferring ? m.pj_transfer_transferring() : m.pj_transfer_button()}
					</Button>
				</div>
			</form>
		{/if}
	</div>
</BottomSheet>

<style>
	.block {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin-bottom: 20px;
		padding: 16px 18px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-large);
	}
	.block__row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
	}
	.block__text {
		display: flex;
		flex-direction: column;
		gap: 4px;
		min-width: 0;
	}
	.block__hint a {
		color: var(--md-sys-color-primary);
		font-weight: 600;
	}
	.block__hint a:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.block__actions {
		display: flex;
		justify-content: flex-end;
	}
	.block__title {
		margin: 0;
		font-size: 0.9375rem;
		font-weight: 700;
	}
	.block__hint {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.danger {
		border: 1px solid color-mix(in srgb, var(--md-sys-color-error) 40%, var(--md-sys-color-outline-variant));
		border-radius: var(--md-sys-shape-corner-large);
	}
	.danger__row {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 16px 18px;
	}
	.danger__row + .danger__row {
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}
	.danger__title {
		margin: 0;
		font-family: var(--md-sys-typescale-title-medium-font);
		font-size: var(--md-sys-typescale-title-medium-size);
		font-weight: var(--md-sys-typescale-title-medium-weight);
		line-height: var(--md-sys-typescale-title-medium-line-height);
		color: var(--md-sys-color-on-surface);
	}
	.muted {
		margin: 4px 0 0;
		max-width: 60ch;
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-sys-typescale-body-medium-font);
		font-size: var(--md-sys-typescale-body-medium-size);
		line-height: var(--md-sys-typescale-body-medium-line-height);
	}
	.error {
		margin: 12px 0 0;
		color: var(--md-sys-color-error);
	}
	.danger :global(.danger-button) {
		background: var(--md-sys-color-error);
		color: var(--md-sys-color-on-error);
	}

	:global(.transfer-bottom-sheet) {
		max-width: min(560px, calc(100% - 32px)) !important;
		height: fit-content !important;
		max-height: 85dvh !important;
		bottom: 16px !important;
		margin: 0 auto !important;
	}
	:global(.transfer-bottom-sheet .m3-bottom-sheet__panel) {
		height: auto !important;
		max-height: 85dvh !important;
		border-radius: var(--md-sys-shape-corner-extra-large) !important;
	}
	:global(.transfer-bottom-sheet .m3-bottom-sheet__body) {
		padding: 0 24px calc(20px + env(safe-area-inset-bottom)) !important;
	}

	.transfer-sheet {
		display: flex;
		flex-direction: column;
		gap: 16px;
		padding: 4px 0 12px;
		height: auto;
	}
	.transfer-sheet__head {
		display: flex;
		align-items: center;
		gap: 14px;
	}
	.transfer-sheet__icon {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 44px;
		height: 44px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
		flex-shrink: 0;
	}
	.transfer-sheet__search {
		margin-top: 2px;
	}
	.transfer-sheet__loading {
		display: flex;
		justify-content: center;
		align-items: center;
		padding: 28px 0;
	}
	.transfer-sheet__empty {
		padding: 24px 0;
		text-align: center;
	}
	.transfer-sheet__list-scroll {
		max-height: 260px;
		overflow-y: auto;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-large);
		padding: 4px;
		background: var(--md-sys-color-surface-container-low);
	}
	.transfer-sheet__trailing {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.transfer-sheet__check {
		display: flex;
		align-items: center;
		color: var(--md-sys-color-primary);
	}
	.m3-badge {
		display: inline-flex;
		align-items: center;
		padding: 2px 8px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
		font-family: var(--md-sys-typescale-label-small-font);
		font-size: var(--md-sys-typescale-label-small-size);
		font-weight: 600;
		letter-spacing: 0.02em;
		line-height: 1.2;
	}
	.m3-badge--subtle {
		background: var(--md-sys-color-surface-container-highest);
		color: var(--md-sys-color-on-surface-variant);
	}
	.transfer-sheet__notice {
		padding: 12px 16px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.transfer-sheet__confirm {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.transfer-sheet__actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 4px;
	}
</style>
