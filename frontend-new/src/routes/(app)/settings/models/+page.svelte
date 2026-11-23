<script lang="ts">
	import { ArrowUpRight, Ellipsis, KeyRound, MessageSquare, Plus, Server, Unplug } from '@lucide/svelte';
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
	import { phoneView } from '$lib/phone.svelte';
	import { phoneScreen } from '$lib/ds/shells/phoneScreen.svelte';
	import PhoneBackBar from '$lib/ds/chrome/PhoneBackBar.svelte';
	import LargeTitle from '$lib/ds/chrome/LargeTitle.svelte';
	import PhoneSettingsGroup, { type SettingsRow } from '$lib/ds/settings/PhoneSettingsGroup.svelte';
	import PhoneFormScreen from '$lib/ds/settings/PhoneFormScreen.svelte';
	import PhoneButton from '$lib/ds/mobile/PhoneButton.svelte';
	import BottomSheet from '$lib/ds/overlays/BottomSheet.svelte';
	import SheetOption from '$lib/ds/overlays/SheetOption.svelte';
	import ActionSheet from '$lib/ds/overlays/ActionSheet.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import Search from '$lib/ds/inputs/Search.svelte';

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

	// On a phone (Flow / Mobile — You & Settings, steps 12 to 15): the defaults as rows that open a
	// picker sheet, then each provider as a row with how many of its models are in the picker; a
	// provider is its own screen (`?provider=`) with its models as switches and Disconnect last.
	// Connect a provider is a sheet with search (`?connect`); picking one opens its key screen
	// (`?connect=<id>`), checked with the provider as it connects, its models then picked for the
	// picker. No table, no top-bar button.
	const guess = () => data.phone;
	const phone = phoneView(guess());
	const screen = phoneScreen();
	$effect(() => {
		screen.inside = phone.current && !!chosen;
		return () => (screen.inside = false);
	});
	let defaultSheet = $state<'chat' | 'coding' | null>(null);
	let disconnecting = $state(false);
	const provider = $derived(cards.find((card) => card.id === page.url.searchParams.get('provider')));
	const chatOptions = $derived(chatChoices(data.options.groups));
	const codingOptions = $derived(data.models ? codingChoices(data.models) : []);
	const firstPart = (label: string | undefined) => label?.split(' · ')[0];
	const providerDetail = (card: (typeof cards)[number]) =>
		card.kind === 'included'
			? data.usage
				? m.models_nomi_usage({ plan: data.usage.plan.name, used: formatTokens(data.usage.tokens_used), allowance: formatTokens(data.usage.plan.monthly_tokens) })
				: m.models_nomi_detail()
			: [card.kind === 'local' ? m.models_local() : m.models_your_key(), card.detail].filter(Boolean).join(' · ');
	const providerRows = $derived<SettingsRow[]>(
		cards.map((card) => ({
			key: card.id,
			...(card.kind === 'included' ? { crew: { agent: 'nomi' as const, initial: 'N' } } : { icon: card.kind === 'local' ? Server : KeyRound }),
			label: card.label,
			sub: providerDetail(card),
			meta: card.error === 'unreachable' ? m.models_unreachable_status() : m.models_in_picker({ shown: card.shown, total: card.total }),
			tone: card.error === 'unreachable' ? ('warning' as const) : undefined,
			href: `?provider=${encodeURIComponent(card.id)}`,
		})),
	);

	async function disconnect() {
		if (!provider?.connectionId) return;
		const name = provider.label;
		const result = await pageAction(PATH, 'disconnect', { id: provider.connectionId });
		if (!result.ok) return void (result.message && toasts.show({ kind: 'error', title: m.action_failed(), detail: result.message }));
		toasts.show({ kind: 'success', title: m.models_disconnected({ name }) });
		await goto(PATH, { replaceState: true, noScroll: true });
		await invalidate('app:models');
	}
</script>

<svelte:head><title>{m.settings_models()} · {m.app_name()}</title></svelte:head>

{#snippet kodaLead()}<Mascot agent="koda" size={20} live={false} />{/snippet}

{#if phone.current}
	<div class="h-full overflow-y-auto">
		{#if chosen}
			{#if chosen.auth_type === 'oauth'}
				<PhoneBackBar label={m.phone_models_back()} onback={() => close()} />
				<LargeTitle title={chosen.name} />
				<p class="px-5 text-phone-sm leading-firm text-text-3">{m.models_oauth({ name: chosen.name })}</p>
			{:else}
				<PhoneFormScreen
					backLabel={m.phone_models_back()}
					onback={() => close()}
					title={chosen.name}
					sub={addressed ? m.phone_models_address_sub() : m.phone_models_key_sub({ name: chosen.name })}
					onsubmit={act}
				>
					<TextField
						label={addressed ? m.models_address() : m.models_api_key()}
						type={addressed ? 'url' : 'password'}
						name={addressed ? 'base_url' : 'api_key'}
						autocomplete="off"
						placeholder={addressed ? 'https://…/v1' : m.models_api_key_placeholder()}
						bind:value={secret}
						disabled={!!made}
						error={failed}
						helper={failed ? undefined : made ? (made.models.length === 1 ? m.models_key_works_one() : m.models_key_works({ count: made.models.length })) : local ? m.models_foot_local() : m.models_foot_key()}
					/>
					{#if !addressed && chosen.doc_url && !made}
						<!-- Its own tab, so a key half-typed here is still here after. -->
						<a
							href={chosen.doc_url}
							target="_blank"
							rel="noopener noreferrer"
							class="-mt-2 inline-flex h-11 items-center gap-1 self-start text-phone-sm font-medium text-text underline-offset-2 outline-offset-1 hover:underline focus-visible:outline-2 focus-visible:outline-focus"
							>{m.models_get_key()}<ArrowUpRight size={14} aria-hidden="true" /></a
						>
					{/if}
					{#if made && made.models.length}
						<PhoneSettingsGroup
							label={m.models_show_in_picker()}
							note={m.models_selected({ count: picked })}
							rows={made.models.map((model) => ({
								key: model.id,
								label: model.label,
								toggle: { checked: model.shown, onchange: (shown) => made && (made = { ...made, models: made.models.map((one) => (one.id === model.id ? { ...one, shown } : one)) }) },
							}))}
						/>
					{/if}
					{#snippet action()}
						<PhoneButton
							label={!made ? m.models_check() : picked === 0 ? m.models_done() : picked === 1 ? m.models_add_one() : m.models_add({ count: picked })}
							disabled={busy || (!made && !secret.trim())}
							onclick={act}
						/>
					{/snippet}
				</PhoneFormScreen>
			{/if}
		{:else if provider}
			<PhoneBackBar label={m.phone_models_back()} href={PATH} />
			<LargeTitle title={provider.label} />
			<p class="-mt-1 px-5 pb-3.5 text-phone-sm text-text-3">{providerDetail(provider)}</p>
			<div class="flex flex-col gap-5 px-4 pb-8">
				<PhoneSettingsGroup
					label={m.models_col_picker()}
					note={m.models_in_picker({ shown: provider.shown, total: provider.total })}
					rows={provider.models.length
						? provider.models.map((model) => ({
								key: model.key,
								label: model.label,
								sub: model.isDefault ? m.models_default_badge() : model.month === '—' ? m.phone_models_month_none() : m.phone_models_month({ tokens: model.month }),
								toggle: { checked: model.enabled, onchange: (shown) => void show(model.key, shown) },
							}))
						: [{ key: 'none', label: m.models_no_models() }]}
				/>
				{#if provider.connectionId}
					<PhoneSettingsGroup rows={[{ key: 'disconnect', icon: Unplug, label: m.phone_models_disconnect({ name: provider.label }), danger: true, onclick: () => (disconnecting = true) }]} />
				{/if}
			</div>
		{:else}
			<PhoneBackBar label={m.nav_you()} href="/settings" />
			<LargeTitle title={m.settings_models()} />
			<div class="flex flex-col gap-5 px-4 pt-1 pb-8">
				<div class="flex flex-col gap-2">
					<PhoneSettingsGroup
						label={m.models_defaults()}
						rows={[
							{ key: 'chat', icon: MessageSquare, label: m.models_new_chats(), meta: firstPart(chatOptions.find((one) => one.value === chatDefault)?.label) ?? m.models_none_shown(), mono: true, onclick: () => (defaultSheet = 'chat') },
							...(data.models ? [{ key: 'coding', lead: kodaLead, label: m.models_coding(), meta: firstPart(codingOptions.find((one) => one.value === codingDefault)?.label), onclick: () => (defaultSheet = 'coding') }] : []),
						]}
					/>
					<p class="px-1 text-phone-sm leading-firm text-text-3">{m.models_defaults_body()}</p>
				</div>
				<PhoneSettingsGroup label={m.models_providers()} rows={providerRows} />
				<PhoneSettingsGroup rows={[{ key: 'connect', icon: Plus, label: m.models_connect(), sub: m.models_connect_detail(), href: '?connect' }]} />
			</div>
		{/if}
	</div>

	<BottomSheet
		open={defaultSheet !== null}
		title={defaultSheet === 'coding' ? m.models_coding() : m.models_new_chats()}
		subtitle={defaultSheet === 'coding' ? m.models_coding_body() : m.models_new_chats_body()}
		onclose={() => (defaultSheet = null)}
	>
		{#each defaultSheet === 'coding' ? codingOptions : chatOptions as option (option.value)}
			<SheetOption
				label={firstPart(option.label) ?? option.label}
				detail={option.label.split(' · ')[1]}
				chosen={option.value === (defaultSheet === 'coding' ? codingDefault : chatDefault)}
				onclick={() => {
					const which = defaultSheet;
					defaultSheet = null;
					if (which === 'coding') {
						codingDefault = option.value;
						void choose('setCoding', { value: option.value });
					} else {
						chatDefault = option.value;
						void choose('setDefault', { key: option.value });
					}
				}}
			/>
		{/each}
	</BottomSheet>
	<BottomSheet open={connecting && !chosen} title={m.models_connect()} subtitle={m.models_meta_providers({ count: data.catalog.length })} onclose={() => connecting && !chosen && void close()}>
		<div class="flex flex-col gap-4 px-4 pb-4">
			<Search placeholder={m.models_search()} label={m.models_search()} bind:value={query} />
			{#each sections as section (section.label)}
				<PhoneSettingsGroup
					label={section.label}
					rows={section.providers.map((one) => ({ key: one.id, icon: KeyRound, label: one.name, meta: one.connected ? m.models_connected() : undefined, tone: one.connected ? ('accent' as const) : undefined, onclick: () => pick(one.id) }))}
				/>
			{:else}
				<p class="px-1 text-phone-sm text-text-3">{m.models_search_none()}</p>
			{/each}
			<PhoneSettingsGroup rows={[{ key: 'custom', icon: Server, label: m.models_custom(), sub: m.models_custom_detail(), onclick: () => pick('custom') }]} />
		</div>
	</BottomSheet>
	<ActionSheet
		bind:open={disconnecting}
		title={m.phone_models_disconnect({ name: provider?.label ?? '' })}
		body={m.phone_models_disconnect_body()}
		cancel={m.action_cancel()}
		actions={[{ label: m.models_disconnect(), danger: true, onclick: () => ((disconnecting = false), void disconnect()) }]}
	/>
{:else}

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
								{#snippet lead()}<Mascot agent="nomi" size={20} live />{/snippet}
							</Select>
						</span>
					{/snippet}
				</SettingRow>
				{#if data.models}
					<SettingRow title={m.models_coding()} description={m.models_coding_body()}>
						{#snippet control()}
							<span class="flex w-70 max-w-full">
								<Select label={m.models_coding()} options={codingChoices(data.models!)} bind:value={codingDefault} mono tall onchange={(value) => choose('setCoding', { value })}>
									{#snippet lead()}<Mascot agent="koda" size={20} live />{/snippet}
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
{/if}
