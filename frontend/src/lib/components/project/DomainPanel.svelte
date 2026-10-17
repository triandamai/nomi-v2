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

	let { deploy }: { deploy: ProjectDeploy } = $props();

	interface Domain { hostname: string | null; subdomain: string | null; url: string | null; can_edit: boolean }
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
	});

	/** `.apps.example.dev`: what follows the name. */
	const suffix = $derived(domain?.hostname && domain.subdomain ? domain.hostname.slice(domain.subdomain.length) : '');

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
	}

	const typed = $derived(name.trim().toLowerCase());
	const unchanged = $derived(typed === (domain?.subdomain ?? ''));

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
		if (!subdomain || subdomain === domain?.subdomain) {
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
		<p class="block__hint">{m.domain_hint()}</p>
		{#if !domain.can_edit}<p class="block__hint">{m.pj_admins_only()}</p>{/if}
	{/if}
	{#if done}<p class="ok" role="status">{done}</p>{/if}
</section>

<section class="block">
	<h3 class="block__title">{m.domain_dns()}</h3>
	<p class="block__hint">{m.domain_dns_hint()}</p>
	<p class="block__hint">{m.domain_custom_soon()}</p>
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
