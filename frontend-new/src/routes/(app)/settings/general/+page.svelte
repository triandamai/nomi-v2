<script lang="ts">
	import { onMount } from 'svelte';
	import { ArrowUpRight, Inbox, LogOut } from '@lucide/svelte';
	import { goto, invalidate } from '$app/navigation';
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { clock } from '$lib/clock';
	import { pageAction } from '$lib/pageAction';
	import { googleErrorFrom } from '$lib/auth/problems';
	import { sayGoogle } from '$lib/auth/say';
	import { devicesLine, maskEmail, zoneOptions } from '$lib/settings/general';
	import type { EmailPreferences } from '$lib/adapters/api';
	import type { Theme } from '$lib/theme';
	import { toasts } from '$lib/ds/overlays/toasts.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import Segmented from '$lib/ds/inputs/Segmented.svelte';
	import Select from '$lib/ds/inputs/Select.svelte';
	import Toggle from '$lib/ds/inputs/Toggle.svelte';
	import SettingsGroup from '$lib/ds/settings/SettingsGroup.svelte';
	import SettingRow from '$lib/ds/settings/SettingRow.svelte';
	import SettingNote from '$lib/ds/settings/SettingNote.svelte';
	import PageToc from '$lib/ds/settings/PageToc.svelte';
	import RelatedLinks from '$lib/ds/settings/RelatedLinks.svelte';
	import SettingsTip from '$lib/ds/settings/SettingsTip.svelte';

	// Settings › General (App / Settings / General m7lvY1): the page's groups down a column (880,
	// 28 apart) beside On this page; every change saves as it's made and goes back if it can't.
	// Left out, as the backend has no place to keep them (AGENTS.md §9, BACKEND_TODO.md): changing
	// the email, Formats, Reduce motion, the Chat group (who new chats start with, Enter sends,
	// tokens under replies), the push and quiet-hours rows of Notifications, and Delete account.
	let { data } = $props();

	const time = clock();
	const language = getLocale();
	// The zones' short names as they are at load time (server and browser agree).
	const loadedAt = () => new Date(data.now);
	const known = zoneOptions(loadedAt());

	const groups = $derived([
		{ href: '#account', label: m.general_account() },
		{ href: '#language', label: m.general_language_region() },
		{ href: '#appearance', label: m.general_appearance() },
		{ href: '#notifications', label: m.general_notifications() },
		{ href: '#sessions', label: m.general_sessions() },
	]);

	// The group in view, for On this page: the last one whose top has passed the top of the column.
	let scroller: HTMLElement | undefined = $state();
	let active = $state('#account');
	function follow() {
		if (!scroller) return;
		const top = scroller.getBoundingClientRect().top + 96;
		const passed = groups.filter((group) => (document.querySelector(group.href)?.getBoundingClientRect().top ?? Infinity) <= top);
		active = passed.at(-1)?.href ?? groups[0].href;
	}

	/** Saves one preference through /preferences; false when it didn't take. */
	async function savePreference(change: { theme?: Theme; language?: string; timezone?: string }): Promise<boolean> {
		const response = await fetch('/preferences', { method: 'PUT', headers: { 'content-type': 'application/json' }, body: JSON.stringify(change) }).catch(() => null);
		const ok = !!response?.ok && !response.redirected;
		if (!ok) toasts.show({ kind: 'error', title: m.general_save_failed() });
		return ok;
	}

	let theme = $derived<Theme>(data.preferences.theme);
	function applyTheme(next: Theme) {
		if (next === 'system') delete document.documentElement.dataset.theme;
		else document.documentElement.dataset.theme = next;
	}
	async function changeTheme(next: string) {
		const before = theme;
		theme = next as Theme;
		applyTheme(theme);
		if (await savePreference({ theme })) return void invalidate('app:shell');
		theme = before;
		applyTheme(before);
	}

	async function changeLanguage(next: string) {
		if (next !== language && (await savePreference({ language: next }))) location.reload();
	}

	let zone = $derived(data.preferences.has_stored_timezone ? data.preferences.timezone : '');
	// The saved zone stays choosable even when this runtime doesn't list it ("UTC", an old alias).
	const zones = $derived(zone && !known.some((option) => option.value === zone) ? [{ value: zone, label: zone }, ...known] : known);
	async function changeZone(next: string) {
		const before = data.preferences.has_stored_timezone ? data.preferences.timezone : '';
		if (await savePreference({ timezone: next })) {
			await invalidate('app:general');
			await invalidate('app:shell');
		} else zone = before;
	}

	// Which emails: switched at once, switched back if saving fails.
	let emails = $derived<EmailPreferences | null>(data.emails);
	async function saveEmails(change: Partial<EmailPreferences>) {
		if (!emails) return;
		const before = emails;
		emails = { ...emails, ...change };
		const result = await pageAction('/settings/general', 'emails', { email_account: String(emails.email_account), email_promos: String(emails.email_promos) });
		if (result.ok) return;
		emails = before;
		if (result.message) toasts.show({ kind: 'error', title: m.general_save_failed(), detail: result.message });
	}

	let busy = $state<string>();
	async function act(name: string, done: (data: Record<string, unknown>) => string) {
		busy = name;
		const result = await pageAction('/settings/general', name);
		busy = undefined;
		if (!result.ok) {
			if (result.message) toasts.show({ kind: 'error', title: m.action_failed(), detail: result.message });
			return;
		}
		toasts.show({ kind: 'success', title: done(result.data) });
		await invalidate('app:general');
	}

	const others = $derived((data.devices ?? []).filter((device) => !device.current).length);
	const resetHref = $derived(`/forgot-password?email=${encodeURIComponent(data.email)}`);

	// Back from Google: say how adding it went, then take the word off the address.
	onMount(() => {
		const linked = page.url.searchParams.get('google') === 'linked';
		const failed = googleErrorFrom(page.url.searchParams.get('google_error'));
		if (!linked && !failed) return;
		if (linked) toasts.show({ kind: 'success', title: m.general_google_done() });
		if (failed) toasts.show({ kind: 'error', title: sayGoogle(failed) });
		void goto(page.url.pathname, { replaceState: true, noScroll: true, keepFocus: true });
	});
</script>

<svelte:head><title>{m.settings_general()} · {m.app_name()}</title></svelte:head>

<div bind:this={scroller} onscroll={follow} class="h-full overflow-y-auto">
	<div class="flex gap-14 px-4 pt-8 pb-10 lg:px-12">
		<div class="flex max-w-220 min-w-0 flex-1 flex-col gap-7">
			<header class="flex flex-col gap-1.5">
				<h1 class="text-display font-medium tracking-display text-text">{m.settings_general()}</h1>
				<p class="text-body text-text-2">{m.general_intro()}</p>
			</header>

			<SettingsGroup id="account" title={m.general_account()} description={m.general_account_body()}>
				<SettingRow title={m.general_email()} description={maskEmail(data.email)} mono />
				{#if data.methods.configured || data.methods.google_email}
					<SettingRow title={m.general_google()} description={data.methods.google_email ? m.general_google_linked({ email: data.methods.google_email }) : m.general_google_off()}>
						{#snippet control()}
							{#if data.methods.google_email}
								<span class="font-mono text-label text-text-2">{m.general_connected()}</span>
								<Button label={m.general_disconnect()} variant="ghost" loading={busy === 'unlinkGoogle'} onclick={() => act('unlinkGoogle', () => m.general_google_removed())} />
							{:else}
								<form method="POST" action="?/linkGoogle"><Button label={m.general_connect()} variant="secondary" type="submit" iconEnd={ArrowUpRight} /></form>
							{/if}
						{/snippet}
					</SettingRow>
				{/if}
				<SettingRow title={m.general_password()} description={data.methods.password ? m.general_password_set() : m.general_password_none()}>
					{#snippet control()}
						<Button label={data.methods.password ? m.general_password_change() : m.general_password_add()} variant="secondary" href={resetHref} iconEnd={ArrowUpRight} />
					{/snippet}
				</SettingRow>
				<SettingNote text={m.general_google_note()} />
			</SettingsGroup>

			<SettingsGroup id="language" title={m.general_language_region()} description={m.general_language_region_body()}>
				<SettingRow title={m.general_language()} description={m.general_language_body()}>
					{#snippet control()}
						<Segmented label={m.general_language()} options={[{ value: 'en', label: m.language_en() }, { value: 'id', label: m.language_id() }]} value={language} onchange={changeLanguage} />
					{/snippet}
				</SettingRow>
				<SettingRow title={m.general_zone()} description={zone ? m.general_zone_body() : m.general_zone_unset()}>
					{#snippet control()}
						<span class="flex w-52 max-w-full"><Select label={m.general_zone()} options={zones} bind:value={zone} placeholder={m.general_zone_choose()} onchange={changeZone} /></span>
					{/snippet}
				</SettingRow>
			</SettingsGroup>

			<SettingsGroup id="appearance" title={m.general_appearance()}>
				<SettingRow title={m.general_theme()} description={m.general_theme_body()}>
					{#snippet control()}
						<Segmented
							label={m.general_theme()}
							options={[
								{ value: 'system', label: m.theme_system() },
								{ value: 'light', label: m.theme_light() },
								{ value: 'dark', label: m.theme_dark() },
							]}
							value={theme}
							onchange={changeTheme}
						/>
					{/snippet}
				</SettingRow>
			</SettingsGroup>

			<SettingsGroup id="notifications" title={m.general_notifications()} description={m.general_notifications_body()}>
				{#if emails}
					<SettingRow title={m.general_email()} description={m.general_email_account()}>
						{#snippet control()}<Toggle label={m.general_email()} checked={emails!.email_account} onchange={(on) => saveEmails({ email_account: on })} />{/snippet}
					</SettingRow>
					<SettingRow title={m.general_product_news()} description={m.general_product_news_body()}>
						{#snippet control()}<Toggle label={m.general_product_news()} checked={emails!.email_promos} onchange={(on) => saveEmails({ email_promos: on })} />{/snippet}
					</SettingRow>
				{:else}
					<SettingNote text={m.general_emails_unloaded()} />
				{/if}
			</SettingsGroup>

			<SettingsGroup id="sessions" title={m.general_sessions()} description={m.general_sessions_body()}>
				{#if data.devices}
					<SettingRow
						title={data.devices.length === 1 ? m.general_devices_one() : m.general_devices({ count: data.devices.length })}
						description={devicesLine(data.devices, time.now, time.timeZone)}
					>
						{#snippet control()}
							{#if others > 0}
								<Button
									label={m.general_sign_out_others()}
									variant="ghost"
									iconStart={LogOut}
									loading={busy === 'signOutOthers'}
									onclick={() => act('signOutOthers', (done) => m.general_signed_out({ count: Number(done.signedOut ?? others) }))}
								/>
							{/if}
						{/snippet}
					</SettingRow>
				{:else}
					<SettingNote text={m.general_devices_unloaded()} />
				{/if}
			</SettingsGroup>
		</div>

		<aside class="sticky top-0 hidden w-58 shrink-0 flex-col gap-5 self-start pt-24 xl:flex">
			<PageToc label={m.general_on_page()} links={groups} {active} />
			<RelatedLinks label={m.general_related()} links={[{ href: '/inbox', label: m.nav_inbox(), icon: Inbox }]} />
			<SettingsTip text={m.general_tip()} />
		</aside>
	</div>
</div>
