<script lang="ts">
	// The project's web address on Shipyard. Its first publish gives production
	// `<name>.<apps-domain>`; admins can move it to another free name. The sheet asks Shipyard as
	// they type (a name is shared across the whole platform) and offers free names nearby.
	import { m } from '$lib/paraglide/messages';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import Chip from '$lib/components/m3/Chip.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import IconOpenInNew from '$lib/components/icons/IconOpenInNew.svelte';
	import { ProjectApiError, projectApi } from '$lib/projectApi';
	import type { ProjectDeploy } from '$lib/projectDeploy.svelte';
	import { ago } from '$lib/projectApi';
	import { getLocale } from '$lib/paraglide/runtime';
	import { canMakePrimary, foundCount, recordHost, typedHostname } from '$lib/customDomains';
	import type { CustomDomain } from '$lib/types';

	let { deploy }: { deploy: ProjectDeploy } = $props();

	interface Domain {
		hostname: string | null;
		subdomain: string | null;
		url: string | null;
		can_edit: boolean;
		/** Nomi's apps base domain, when an admin set one. */
		base_domain: string | null;
		/** Where the next publish moves the address, when it isn't under the base domain yet. */
		pending_hostname: string | null;
	}
	interface Suggestion { subdomain: string; hostname: string }
	interface Check { subdomain: string; hostname: string | null; available: boolean; reason?: 'taken' | 'invalid'; message?: string; suggestions: Suggestion[] }

	let domain = $state<Domain | null>(null);
	let loadError = $state<string | null>(null);

	async function load() {
		try {
			domain = await projectApi<Domain>(deploy.projectId, 'domain');
			loadError = null;
		} catch (e) {
			loadError = (e as Error).message;
		}
	}
	// Again whenever the project's publishing changes (a first publish gives it an address).
	$effect(() => {
		void deploy.changes;
		void load();
		void loadDomains();
	});

	// ─── Domains of the person's own ─────────────────────────────
	const locale = getLocale();
	let domains = $state<CustomDomain[] | null>(null);
	let domainsError = $state<string | null>(null);
	let newHostname = $state('');
	let adding = $state(false);
	/** The hostname an action is running on, so its buttons wait. */
	let busy = $state<string | null>(null);

	async function loadDomains() {
		try {
			domains = await projectApi<CustomDomain[]>(deploy.projectId, 'domains');
			domainsError = null;
		} catch (e) {
			domainsError = (e as Error).message;
		}
	}

	async function addDomain() {
		const hostname = typedHostname(newHostname);
		if (!hostname) return;
		adding = true;
		domainsError = null;
		try {
			await projectApi<CustomDomain>(deploy.projectId, 'domains', { method: 'POST', body: { hostname } });
			newHostname = '';
			await loadDomains();
		} catch (e) {
			domainsError = (e as Error).message;
		} finally {
			adding = false;
		}
	}

	async function onDomain(hostname: string, action: 'check' | 'primary' | 'remove') {
		busy = hostname;
		domainsError = null;
		const path = `domains/${encodeURIComponent(hostname)}`;
		try {
			if (action === 'check') await projectApi(deploy.projectId, `${path}/check`, { method: 'POST' });
			else if (action === 'primary') await projectApi(deploy.projectId, `${path}/primary`, { method: 'PUT' });
			else await projectApi(deploy.projectId, path, { method: 'DELETE' });
			await loadDomains();
		} catch (e) {
			domainsError = (e as Error).message;
		} finally {
			busy = null;
		}
	}

	/** `.apps.example.dev`: what follows the name. Nomi's base domain when set (the current address
	 * may still be on an older one until it moves), else whatever the current address ends with. */
	const suffix = $derived(
		domain?.base_domain ? `.${domain.base_domain}` : domain?.hostname && domain.subdomain ? domain.hostname.slice(domain.subdomain.length) : '',
	);

	// ─── Change sheet ────────────────────────────────────────────
	let open = $state(false);
	let name = $state('');
	let check = $state<Check | null>(null);
	let checking = $state(false);
	let saving = $state(false);
	let changeError = $state<string | null>(null);
	let done = $state<string | null>(null);
	let timer: ReturnType<typeof setTimeout> | undefined;
	let asked = 0;

	function openSheet() {
		name = domain?.subdomain ?? '';
		check = null;
		changeError = null;
		open = true;
		// Not on the base domain yet: the same name is a move, so it's checked like any other.
		if (domain?.pending_hostname && name) void ask(name);
	}

	const typed = $derived(name.trim().toLowerCase());
	const unchanged = $derived(typed === (domain?.subdomain ?? '') && !domain?.pending_hostname);

	async function ask(subdomain: string): Promise<Check | null> {
		const id = ++asked;
		checking = true;
		try {
			const answer = await projectApi<Check>(deploy.projectId, `domain/check?subdomain=${encodeURIComponent(subdomain)}`);
			// Only the answer for what's in the field now counts.
			if (id === asked) check = answer;
			return answer;
		} catch (e) {
			if (id === asked) changeError = (e as Error).message;
			return null;
		} finally {
			if (id === asked) checking = false;
		}
	}

	function onInput(value: string) {
		name = value;
		changeError = null;
		clearTimeout(timer);
		const subdomain = value.trim().toLowerCase();
		if (!subdomain || (subdomain === domain?.subdomain && !domain?.pending_hostname)) {
			asked++;
			check = null;
			checking = false;
			return;
		}
		checking = true;
		timer = setTimeout(() => void ask(subdomain), 350);
	}

	function pick(suggestion: Suggestion) {
		name = suggestion.subdomain;
		clearTimeout(timer);
		void ask(suggestion.subdomain);
	}

	async function save() {
		saving = true;
		changeError = null;
		try {
			const result = await projectApi<{ hostname: string }>(deploy.projectId, 'domain', { method: 'PUT', body: { subdomain: typed } });
			open = false;
			done = m.domain_changed({ hostname: result.hostname });
			await load();
			void deploy.refresh();
		} catch (e) {
			changeError = (e as Error).message;
			// Taken a moment ago: show what's free now.
			if (e instanceof ProjectApiError && e.status === 409) await ask(typed);
		} finally {
			saving = false;
		}
	}
</script>

<section class="block">
	<h3 class="block__title">{m.domain_address()}</h3>
	{#if loadError}
		<p class="error" role="alert">{loadError}</p>
	{:else if !domain}
		<p class="block__hint">{m.common_loading()}</p>
	{:else if !domain.hostname}
		<p class="block__hint">{m.domain_unpublished()}</p>
	{:else}
		<div class="address">
			<a class="address__link" href={domain.url ?? `https://${domain.hostname}`} target="_blank" rel="noopener noreferrer">
				<span class="address__host">{domain.hostname}</span>
				<IconOpenInNew size={18} />
			</a>
			{#if domain.can_edit}
				<Button variant="tonal" size="s" onclick={openSheet}>{m.domain_change()}</Button>
			{/if}
		</div>
		{#if domain.pending_hostname}
			<div class="pending">
				<p class="block__hint">{m.domain_pending({ hostname: domain.pending_hostname })}</p>
				{#if domain.can_edit}<Button variant="text" size="s" onclick={openSheet}>{m.domain_move_now()}</Button>{/if}
			</div>
		{/if}
		<p class="block__hint">{m.domain_hint()}</p>
		{#if !domain.can_edit}<p class="block__hint">{m.pj_admins_only()}</p>{/if}
	{/if}
	{#if done}<p class="ok" role="status">{done}</p>{/if}
</section>

<section class="block">
	<h3 class="block__title">{m.domain_dns()}</h3>
	<p class="block__hint">{m.domain_dns_hint()}</p>
</section>

<section class="block" aria-label={m.domain_own_title()}>
	<h3 class="block__title">{m.domain_own_title()}</h3>
	<p class="block__hint">{m.domain_own_hint()}</p>
	{#if domains && domains.length > 0}
		<ul class="domains">
			{#each domains as d (d.hostname)}
				{@const count = foundCount(d)}
				<li class="domain">
					<div class="domain__head">
						<span class="address__host domain__host">{d.hostname}</span>
						{#if d.primary}<span class="tag">{m.domain_own_primary()}</span>{/if}
						<span class="block__hint">{d.verified_at ? m.domain_own_connected() : m.domain_own_found({ found: count.found, total: count.total })}</span>
					</div>
					<table class="records">
						<thead>
							<tr><th>{m.domain_own_type()}</th><th>{m.domain_own_name()}</th><th>{m.domain_own_value()}</th><th>{m.domain_own_status()}</th></tr>
						</thead>
						<tbody>
							{#each d.records as r (r.type + r.name + r.value)}
								<tr>
									<td>{r.type}</td>
									<td class="mono">{recordHost(r.name, d.hostname)}</td>
									<td class="mono value">{r.value}</td>
									<td class={r.status === 'found' ? 'ok' : 'waiting'}>{r.status === 'found' ? m.domain_own_record_found() : m.domain_own_record_waiting()}</td>
								</tr>
							{/each}
						</tbody>
					</table>
					<p class="block__hint">
						{d.checked_at ? m.domain_own_checked({ when: ago(d.checked_at, locale) }) : m.domain_own_never_checked()}
						· {d.https === 'issued' ? m.domain_own_https_issued() : m.domain_own_https_pending()}
					</p>
					<div class="domain__actions">
						<Button variant="tonal" size="s" disabled={busy === d.hostname} onclick={() => onDomain(d.hostname, 'check')}>{m.domain_own_check()}</Button>
						{#if domain?.can_edit}
							{#if canMakePrimary(d)}
								<Button variant="text" size="s" disabled={busy === d.hostname} onclick={() => onDomain(d.hostname, 'primary')}>{m.domain_own_make_primary()}</Button>
							{/if}
							<Button variant="text" size="s" disabled={busy === d.hostname} onclick={() => onDomain(d.hostname, 'remove')}>{m.domain_own_remove()}</Button>
						{/if}
					</div>
				</li>
			{/each}
		</ul>
	{/if}
	{#if domain?.can_edit && domain.hostname}
		<form
			class="add"
			onsubmit={(e) => {
				e.preventDefault();
				void addDomain();
			}}
		>
			<TextField name="hostname" label={m.domain_own_add_label()} bind:value={newHostname} autocomplete="off" autocapitalize="none" spellcheck={false} />
			<Button variant="filled" size="s" type="submit" disabled={adding || !typedHostname(newHostname)}>{m.domain_own_add()}</Button>
		</form>
	{:else if domain && !domain.hostname}
		<p class="block__hint">{m.domain_unpublished()}</p>
	{/if}
	{#if domainsError}<p class="error" role="alert">{domainsError}</p>{/if}
</section>

<BottomSheet bind:open label={m.domain_change_title()}>
	<form
		class="sheet"
		onsubmit={(e) => {
			e.preventDefault();
			if (check?.available && !unchanged) void save();
		}}
	>
		<h2 class="sheet__title">{m.domain_change_title()}</h2>
		<p class="block__hint">{m.domain_change_lede()}</p>
		<div class="name">
			<TextField
				name="subdomain"
				label={m.domain_name()}
				value={name}
				oninput={(e) => onInput((e.target as HTMLInputElement).value)}
				autocomplete="off"
				autocapitalize="none"
				spellcheck={false}
				maxlength={63}
				error={!!check && !check.available}
			/>
			{#if suffix}<span class="name__suffix">{suffix}</span>{/if}
		</div>

		<div class="answer" aria-live="polite">
			{#if unchanged && typed}
				<p class="block__hint">{m.domain_current()}</p>
			{:else if checking}
				<p class="block__hint">{m.domain_checking()}</p>
			{:else if check?.available}
				<p class="ok">{m.domain_available({ hostname: check.hostname ?? `${typed}${suffix}` })}</p>
			{:else if check}
				<p class="error">{check.message ?? m.domain_taken()}</p>
			{/if}
		</div>

		{#if check && !check.available && check.suggestions.length > 0 && !checking}
			<div class="suggestions">
				<span class="block__hint">{m.domain_try()}</span>
				<div class="suggestions__chips">
					{#each check.suggestions as suggestion (suggestion.subdomain)}
						<Chip variant="suggestion" onclick={() => pick(suggestion)}>{suggestion.hostname}</Chip>
					{/each}
				</div>
			</div>
		{/if}

		{#if changeError}<p class="error" role="alert">{changeError}</p>{/if}

		<div class="sheet__actions">
			<Button variant="text" type="button" onclick={() => (open = false)}>{m.common_cancel()}</Button>
			<Button variant="filled" type="submit" disabled={saving || checking || unchanged || !check?.available}>{m.domain_use()}</Button>
		</div>
	</form>
</BottomSheet>

<style>
	.pending {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 4px 12px;
	}
	.block {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin-bottom: 20px;
		padding: 16px 18px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-large);
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
	.address {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: 8px 16px;
	}
	.address__link {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		min-width: 0;
		min-height: 44px;
		color: var(--md-sys-color-primary);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.9375rem;
		font-weight: 600;
		text-decoration: none;
	}
	.address__link:hover .address__host {
		text-decoration: underline;
	}
	.address__link:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
		border-radius: 4px;
	}
	.address__host {
		overflow-wrap: anywhere;
	}
	.sheet {
		display: flex;
		flex-direction: column;
		gap: 12px;
		padding: 8px 16px 16px;
	}
	.sheet__title {
		margin: 0;
		font-size: 1.375rem;
		font-weight: 700;
	}
	.name {
		display: flex;
		flex-wrap: wrap;
		align-items: flex-end;
		gap: 4px 8px;
	}
	.name :global(.m3-field) {
		flex: 1 1 200px;
	}
	.name__suffix {
		padding-bottom: 14px;
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.875rem;
		overflow-wrap: anywhere;
	}
	.answer {
		min-height: 22px;
	}
	.answer p {
		margin: 0;
	}
	.suggestions {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.suggestions__chips {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}
	.sheet__actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 8px;
	}
	.domains {
		display: flex;
		flex-direction: column;
		gap: 16px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.domain {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.domain__head {
		display: flex;
		flex-wrap: wrap;
		align-items: baseline;
		gap: 4px 10px;
	}
	.domain__host {
		font-family: var(--md-ref-typeface-mono);
		font-weight: 600;
	}
	.tag {
		padding: 2px 8px;
		border-radius: 999px;
		background: var(--md-sys-color-surface-container-high);
		font-size: 0.75rem;
		font-weight: 600;
	}
	.records {
		display: block;
		overflow-x: auto;
		border-collapse: collapse;
		font-size: 0.8125rem;
	}
	.records th,
	.records td {
		padding: 6px 10px 6px 0;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
		text-align: left;
		white-space: nowrap;
	}
	.records th {
		color: var(--md-sys-color-on-surface-variant);
		font-weight: 600;
	}
	.mono {
		font-family: var(--md-ref-typeface-mono);
	}
	.value {
		overflow-wrap: anywhere;
		white-space: normal;
	}
	.waiting {
		color: var(--md-sys-color-on-surface-variant);
	}
	.domain__actions {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}
	.add {
		display: flex;
		flex-wrap: wrap;
		align-items: flex-end;
		gap: 8px;
	}
	.add :global(.m3-field) {
		flex: 1 1 220px;
	}
	.ok {
		margin: 0;
		color: var(--md-sys-color-primary);
		font-weight: 600;
	}
	.error {
		margin: 0;
		color: var(--md-sys-color-error);
	}
</style>
