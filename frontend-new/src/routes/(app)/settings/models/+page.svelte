<script lang="ts">
	import { Ellipsis, Unplug } from '@lucide/svelte';
	import { goto, invalidate } from '$app/navigation';
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import { pageAction } from '$lib/pageAction';
	import { formatTokens } from '$lib/adapters/shell';
	import { catalogSections, chatChoices, codingChoices, codingValue, isLocal, providerCards } from '$lib/settings/models';
	import { toasts } from '$lib/ds/overlays/toasts.svelte';
	import type { MenuEntry } from '$lib/ds/overlays/MenuPanel.svelte';
	import IconButton from '$lib/ds/actions/IconButton.svelte';
	import Menu from '$lib/ds/overlays/Menu.svelte';
	import Modal from '$lib/ds/overlays/Modal.svelte';
	import Mascot from '$lib/ds/agents/Mascot.svelte';
	import Select from '$lib/ds/inputs/Select.svelte';
	import Status from '$lib/ds/status/Status.svelte';
	import Tag from '$lib/ds/status/Tag.svelte';
	import SettingsGroup from '$lib/ds/settings/SettingsGroup.svelte';
	import SettingRow from '$lib/ds/settings/SettingRow.svelte';
	import SettingsTip from '$lib/ds/settings/SettingsTip.svelte';
	import ProviderCard from '$lib/ds/settings/ProviderCard.svelte';
	import ConnectRow from '$lib/ds/settings/ConnectRow.svelte';
	import WhoPays from '$lib/ds/settings/WhoPays.svelte';
	import ConnectDialog from '$lib/ds/settings/ConnectDialog.svelte';
	import ProviderList from '$lib/ds/settings/ProviderList.svelte';
	import ConnectPane from '$lib/ds/settings/ConnectPane.svelte';

	// Settings › Models & providers (App / Settings / Models & providers RbTgo): the defaults for new
	// chats and for Koda; Nomi's own models and each connected provider with which models are in the
	// picker; and Connect a provider (voUxB), opened from the top bar or the row (`?connect`). The
	// backend checks a key with its provider before keeping it, so Check and connect does both; the
	// models it offers are then switched on for the picker, and backing out disconnects it again.
	// Left out (AGENTS.md §9): the providers' logos (their initials stand in), the Context column and
	// the models' Fast / Reasoning / Vision tags, "Each crew member" (until Crew is in v2), the
	// Gateways / Direct / Local filters, and signing in to a provider (OAuth).
	let { data } = $props();

	const PATH = '/settings/models';
	const SUGGESTED = ['openrouter', 'anthropic', 'openai', 'google'];

	// The picker: what the backend says, with switches flipped here until it confirms them.
	let flipped = $state<Record<string, boolean>>({});
	const cards = $derived(
		providerCards(data.options.groups, data.usage, data.options.current?.key ?? null, data.connections).map((card) => {
			const models = card.models.map((model) => (model.key in flipped ? { ...model, enabled: flipped[model.key] } : model));
			return { ...card, models, shown: models.filter((model) => model.enabled).length };
		}),
	);
	// Nomi's own models start open; the person's providers start closed.
	let opened = $state<Record<string, boolean>>({});

	async function show(key: string, shown: boolean) {
		flipped = { ...flipped, [key]: shown };
		const result = await pageAction(PATH, 'setShown', { changes: JSON.stringify([{ key, enabled: shown }]) });
		if (!result.ok && result.message) toasts.show({ kind: 'error', title: m.general_save_failed(), detail: result.message });
		await invalidate('app:models');
		const { [key]: _, ...rest } = flipped;
		flipped = rest;
	}

	// The defaults: chosen at once, put back if saving fails.
	let chatDefault = $derived(data.options.current?.key ?? '');
	let codingDefault = $derived(data.models ? codingValue(data.models.coding) : '');
	async function choose(action: 'setDefault' | 'setCoding', fields: Record<string, string>) {
		const result = await pageAction(PATH, action, fields);
		if (!result.ok && result.message) toasts.show({ kind: 'error', title: m.general_save_failed(), detail: result.message });
		await invalidate('app:models');
	}

	function menu(card: (typeof cards)[number]): MenuEntry[] {
		return [
			{
				label: m.models_disconnect(),
				icon: Unplug,
				danger: true,
				onselect: async () => {
					const result = await pageAction(PATH, 'disconnect', { id: card.connectionId! });
					if (!result.ok) return void (result.message && toasts.show({ kind: 'error', title: m.action_failed(), detail: result.message }));
					toasts.show({ kind: 'success', title: m.models_disconnected({ name: card.label }) });
					await invalidate('app:models');
				},
			},
		];
	}

	// Connect a provider: open while the address says so (`?connect`, or `?connect=<provider>`).
	const connecting = $derived(page.url.searchParams.has('connect'));
	const chosen = $derived(data.catalog.find((provider) => provider.id === page.url.searchParams.get('connect')));
	const connectedIds = $derived(data.connections.map((connection) => connection.provider_id));
	let query = $state('');
	const sections = $derived(
		catalogSections(data.catalog, connectedIds, query).map((section) => ({
			label: section.label,
			providers: section.providers.map((provider) => ({ id: provider.id, name: provider.name, connected: connectedIds.includes(provider.id) })),
		})),
	);
	const suggestions = $derived(SUGGESTED.map((id) => data.catalog.find((provider) => provider.id === id)).filter((provider) => provider !== undefined));

	let secret = $state('');
	let busy = $state(false);
	let failed = $state<string>();
	let made = $state<{ id: string; label: string; models: { id: string; label: string; shown: boolean }[] }>();
	const addressed = $derived(chosen?.auth_type === 'custom_endpoint');
	const local = $derived(addressed && isLocal(secret || chosen?.default_base_url || null));
	const picked = $derived(made?.models.filter((model) => model.shown).length ?? 0);

	function pick(id: string) {
		void goto(`?connect=${encodeURIComponent(id)}`, { replaceState: true, noScroll: true, keepFocus: true });
	}

	$effect(() => {
		// A new provider starts the pane again, its local address already filled in.
		const provider = chosen;
		secret = provider?.auth_type === 'custom_endpoint' ? (provider.default_base_url ?? '') : '';
		failed = undefined;
		made = undefined;
	});

	async function close(done = false) {
		if (made && !done) await pageAction(PATH, 'disconnect', { id: made.id });
		made = undefined;
		await goto(PATH, { replaceState: true, noScroll: true });
		if (done) await invalidate('app:models');
	}

	async function act() {
		if (!chosen) return;
		busy = true;
		failed = undefined;
		if (!made) {
			const result = await pageAction(PATH, 'connect', { provider_id: chosen.id, ...(addressed ? { base_url: secret } : { api_key: secret }) });
			busy = false;
			if (!result.ok) return void (failed = result.message || m.action_failed());
			const connection = result.data.connection as { id: string; label: string };
			const offered = (result.data.models as { id: string; label: string | null }[]) ?? [];
			made = { ...connection, models: offered.map((model) => ({ id: model.id, label: model.label ?? model.id, shown: false })) };
			return;
		}
		const shown = made.models.filter((model) => model.shown).map((model) => model.id);
		if (shown.length) {
			const result = await pageAction(PATH, 'showModels', { connection_id: made.id, models: JSON.stringify(shown) });
			if (!result.ok) {
				busy = false;
				return void (failed = result.message || m.action_failed());
			}
		}
		busy = false;
		toasts.show({ kind: 'success', title: m.models_added({ name: made.label }) });
		await close(true);
	}
</script>

<svelte:head><title>{m.settings_models()} · {m.app_name()}</title></svelte:head>

<div class="h-full overflow-y-auto">
	<div class="flex gap-14 px-4 pt-8 pb-12 lg:px-12">
		<div class="flex max-w-220 min-w-0 flex-1 flex-col gap-8">
			<header class="flex flex-col gap-1.5">
				<h1 class="text-display font-medium tracking-display text-text">{m.settings_models()}</h1>
				<p class="text-body text-text-2">{m.models_intro()}</p>
			</header>

			<SettingsGroup id="defaults" title={m.models_defaults()} description={m.models_defaults_body()}>
				<SettingRow title={m.models_new_chats()} description={m.models_new_chats_body()}>
					{#snippet control()}
						<span class="flex w-70 max-w-full">
							<Select
								label={m.models_new_chats()}
								options={chatChoices(data.options.groups)}
								bind:value={chatDefault}
								placeholder={m.models_none_shown()}
								mono
								tall
								onchange={(key) => choose('setDefault', { key })}
							>
								{#snippet lead()}<Mascot agent="nomi" size={20} />{/snippet}
							</Select>
						</span>
					{/snippet}
				</SettingRow>
				{#if data.models}
					<SettingRow title={m.models_coding()} description={m.models_coding_body()}>
						{#snippet control()}
							<span class="flex w-70 max-w-full">
								<Select label={m.models_coding()} options={codingChoices(data.models!)} bind:value={codingDefault} mono tall onchange={(value) => choose('setCoding', { value })}>
									{#snippet lead()}<Mascot agent="koda" size={20} />{/snippet}
								</Select>
							</span>
						{/snippet}
					</SettingRow>
				{/if}
			</SettingsGroup>

			<section aria-labelledby="providers-title" class="flex flex-col gap-3">
				<header class="flex flex-col gap-0.5">
					<h2 id="providers-title" class="text-body font-medium text-text">{m.models_providers()}</h2>
					<p class="text-caption text-text-3">{m.models_providers_body()}</p>
				</header>
				{#each cards as card (card.id)}
					<ProviderCard
						name={card.label}
						mascot={card.kind === 'included' ? 'nomi' : undefined}
						detail={card.kind === 'included'
							? data.usage
								? m.models_nomi_usage({ plan: data.usage.plan.name, used: formatTokens(data.usage.tokens_used), allowance: formatTokens(data.usage.plan.monthly_tokens) })
								: m.models_nomi_detail()
							: (card.detail ?? undefined)}
						count={m.models_in_picker({ shown: card.shown, total: card.total })}
						models={card.models}
						columns={{ model: m.models_col_model(), month: m.models_col_month(), picker: m.models_col_picker() }}
						defaultLabel={m.models_default_badge()}
						empty={m.models_no_models()}
						bind:open={() => opened[card.id] ?? card.kind === 'included', (open) => (opened = { ...opened, [card.id]: open })}
						onshow={show}
					>
						{#snippet tag()}
							<Tag kind={card.kind === 'included' ? 'go' : 'neutral'} label={card.kind === 'included' ? m.models_included() : card.kind === 'local' ? m.models_local() : m.models_your_key()} />
						{/snippet}
						{#snippet status()}
							{#if card.error === 'unreachable'}<Status kind="failed" label={m.models_unreachable_status()} />{/if}
						{/snippet}
						{#snippet tools()}
							{#if card.connectionId}
								<Menu items={menu(card)} align="end">
									{#snippet trigger(props)}<IconButton icon={Ellipsis} label={m.models_more({ name: card.label })} variant="ghost" {...props} />{/snippet}
								</Menu>
							{/if}
						{/snippet}
					</ProviderCard>
				{/each}
				<ConnectRow
					title={m.models_connect()}
					detail={m.models_connect_detail()}
					suggestions={suggestions.map((provider) => ({ id: provider.id, name: provider.name }))}
					all={m.models_all({ count: data.catalog.length })}
					onpick={pick}
					onall={() => goto('?connect', { noScroll: true, keepFocus: true })}
				/>
			</section>
		</div>

		<aside class="sticky top-0 hidden w-58 shrink-0 flex-col gap-5 self-start pt-24 xl:flex">
			<WhoPays
				label={m.models_who_pays()}
				kinds={[
					{ tag: m.models_included(), text: m.models_who_included({ allowance: formatTokens(data.usage?.plan.monthly_tokens ?? 0) }), accent: true },
					{ tag: m.models_your_key(), text: m.models_who_key() },
					{ tag: m.models_local(), text: m.models_who_local() },
				]}
			/>
			<SettingsTip kind="info" text={m.models_tip()} />
		</aside>
	</div>
</div>

<Modal open={connecting} labelledby="connect-title" onclose={() => close()}>
	<ConnectDialog>
		{#snippet list()}
			<ProviderList
				search={m.models_search()}
				bind:query
				{sections}
				chosen={chosen?.id}
				empty={m.models_search_none()}
				custom={{ title: m.models_custom(), detail: m.models_custom_detail() }}
				onpick={pick}
			/>
		{/snippet}
		{#snippet pane()}
			{#if chosen}
				{#if chosen.auth_type === 'oauth'}
					<div class="flex flex-1 flex-col items-start gap-3 p-5">
						<h2 id="connect-title" class="text-body font-medium text-text">{m.models_connect_title({ name: chosen.name })}</h2>
						<p class="text-caption text-text-3">{m.models_oauth({ name: chosen.name })}</p>
					</div>
				{:else}
					<ConnectPane
						titleId="connect-title"
						title={m.models_connect_title({ name: chosen.name })}
						name={chosen.name}
						billing={local ? m.models_billed_local() : m.models_billed_by({ name: chosen.name })}
						closeLabel={m.models_close()}
						field={{
							label: addressed ? m.models_address() : m.models_api_key(),
							placeholder: addressed ? 'https://…/v1' : m.models_api_key_placeholder(),
							secret: !addressed,
							getKey: !addressed && chosen.doc_url ? { href: chosen.doc_url, label: m.models_get_key() } : undefined,
						}}
						bind:value={secret}
						check={failed
							? { kind: 'failed', text: failed }
							: made
								? { kind: 'ok', text: made.models.length === 1 ? m.models_key_works_one() : m.models_key_works({ count: made.models.length }) }
								: undefined}
						models={made && made.models.length
							? { label: m.models_show_in_picker(), note: m.models_selected({ count: picked }), items: made.models }
							: undefined}
						foot={local ? m.models_foot_local() : m.models_foot_key()}
						cancel={m.models_cancel()}
						action={!made ? m.models_check() : picked === 0 ? m.models_done() : picked === 1 ? m.models_add_one() : m.models_add({ count: picked })}
						{busy}
						disabled={!made && !secret.trim()}
						onclose={() => close()}
						onaction={act}
						onshow={(id, shown) => made && (made = { ...made, models: made.models.map((model) => (model.id === id ? { ...model, shown } : model)) })}
					/>
				{/if}
			{:else}
				<div class="flex flex-1 flex-col items-start justify-center gap-1.5 p-8">
					<h2 id="connect-title" class="text-body font-medium text-text">{m.models_pick_first()}</h2>
					<p class="text-caption text-text-3">{m.models_pick_first_body()}</p>
				</div>
			{/if}
		{/snippet}
	</ConnectDialog>
</Modal>
