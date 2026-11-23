<script lang="ts">
	import { onMount } from 'svelte';
	import { ArrowUpRight, Inbox, LogOut } from '@lucide/svelte';
	import { deserialize } from '$app/forms';
	import { leaveTo } from '$lib/live/leaving';
	import { goto, invalidate } from '$app/navigation';
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { clock } from '$lib/clock';
	import { pageAction } from '$lib/pageAction';
	import { googleErrorFrom } from '$lib/auth/problems';
	import { sayGoogle } from '$lib/auth/say';
	import { devicesLine, maskEmail, parseQuiet, quietOptions, quietValue, zoneOptions } from '$lib/settings/general';
	import type { EmailPreferences, ReminderPreferences } from '$lib/adapters/api';
	import { agentTypeOf, crewOf } from '$lib/adapters/shell';
	import { mentionOptions } from '$lib/adapters/chat';
	import Mascot from '$lib/ds/agents/Mascot.svelte';
	import type { Theme } from '$lib/theme';
	import { toasts } from '$lib/ds/overlays/toasts.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import AuthLink from '$lib/ds/auth/AuthLink.svelte';
	import CodeInput from '$lib/ds/auth/CodeInput.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import Dialog from '$lib/ds/overlays/Dialog.svelte';
	import Segmented from '$lib/ds/inputs/Segmented.svelte';
	import Select from '$lib/ds/inputs/Select.svelte';
	import Toggle from '$lib/ds/inputs/Toggle.svelte';
	import SettingsGroup from '$lib/ds/settings/SettingsGroup.svelte';
	import SettingRow from '$lib/ds/settings/SettingRow.svelte';
	import SettingNote from '$lib/ds/settings/SettingNote.svelte';
	import PageToc from '$lib/ds/settings/PageToc.svelte';
	import RelatedLinks from '$lib/ds/settings/RelatedLinks.svelte';
	import SettingsTip from '$lib/ds/settings/SettingsTip.svelte';
	import { Bell, Clock3, CornerDownLeft, Globe, Languages, Lock, Mail, MessageCircle, Moon, Newspaper, Smartphone, SunMoon, Trash2 } from '@lucide/svelte';
	import { phoneView } from '$lib/phone.svelte';
	import { phoneScreen } from '$lib/ds/shells/phoneScreen.svelte';
	import PhoneBackBar from '$lib/ds/chrome/PhoneBackBar.svelte';
	import LargeTitle from '$lib/ds/chrome/LargeTitle.svelte';
	import PhoneSettingsGroup from '$lib/ds/settings/PhoneSettingsGroup.svelte';
	import PhoneFormScreen from '$lib/ds/settings/PhoneFormScreen.svelte';
	import PhoneButton from '$lib/ds/mobile/PhoneButton.svelte';
	import BottomSheet from '$lib/ds/overlays/BottomSheet.svelte';
	import SheetOption from '$lib/ds/overlays/SheetOption.svelte';
	import ActionSheet from '$lib/ds/overlays/ActionSheet.svelte';

	// Settings › General (App / Settings / General m7lvY1): the page's groups down a column (880,
	// 28 apart) beside On this page; every change saves as it's made and goes back if it can't.
	// Left out (AGENTS.md §9): Formats and tokens under replies (the backend keeps them; nothing in v2 follows them yet), Reduce motion
	// (the device's own setting rules) and Crew updates (nothing to keep it).
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
		{ href: '#chat', label: m.general_chat() },
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

	// After a save, the page's own data is read again too: the rows below keep their state in
	// `$derived(data…)`, and the next `app:shell` refresh rebuilds `data` from whatever the page last
	// loaded, which would put a row just saved back as it was.
	const refresh = (shell = false) => Promise.all([invalidate('app:general'), shell && invalidate('app:shell')]);

	/** Saves one preference through /preferences; false when it didn't take. */
	async function savePreference(change: { theme?: Theme; language?: string; timezone?: string; default_agent?: string; enter_sends?: boolean }): Promise<boolean> {
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
		if (await savePreference({ theme })) return void refresh(true);
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
			await refresh(true);
		} else zone = before;
	}

	// Which emails: switched at once, switched back if saving fails.
	let emails = $derived<EmailPreferences | null>(data.emails);
	async function saveEmails(change: Partial<EmailPreferences>) {
		if (!emails) return;
		const before = emails;
		emails = { ...emails, ...change };
		const result = await pageAction('/settings/general', 'emails', { email_account: String(emails.email_account), email_promos: String(emails.email_promos) });
		if (result.ok) return void refresh();
		emails = before;
		if (result.message) toasts.show({ kind: 'error', title: m.general_save_failed(), detail: result.message });
	}

	// Chat: who new chats start with (Home's To) and whether Enter sends, through /preferences.
	const starters = $derived([
		{ value: '', label: 'Nomi' },
		...mentionOptions(data.crew.rows).flatMap((option) => {
			const type = option.kind === 'crew' && option.agent !== 'nomi' ? agentTypeOf(option.agent) : undefined;
			return type ? [{ value: type, label: option.name }] : [];
		}),
	]);
	let startsWith = $derived(data.preferences.default_agent && starters.some((option) => option.value === data.preferences.default_agent) ? data.preferences.default_agent : '');
	const starterMascot = $derived((startsWith && crewOf(startsWith)) || 'nomi');
	async function changeStarter(next: string) {
		const before = data.preferences.default_agent ?? '';
		if (await savePreference({ default_agent: next })) await refresh(true);
		else startsWith = before;
	}
	let enterSends = $derived(data.preferences.enter_sends ?? true);
	async function changeEnterSends(on: boolean) {
		enterSends = on;
		if (await savePreference({ enter_sends: on })) await refresh(true);
		else enterSends = !on;
	}

	// Reminders: pushes and quiet hours, switched at once and back if saving fails.
	let reminders = $derived<ReminderPreferences | null>(data.reminders);
	// The night the select shows: the saved one, or the pen's 22:00–06:00 while they're off.
	let night = $derived(quietValue(data.reminders?.quiet_hours ?? { start: '22:00', end: '06:00' }));
	const nights = $derived(quietOptions(reminders?.quiet_hours ?? null));
	async function saveReminders(fields: Record<string, string>, next: ReminderPreferences) {
		if (!reminders) return;
		const before = reminders;
		reminders = next;
		const result = await pageAction('/settings/general', 'reminders', fields);
		if (result.ok) return void refresh();
		reminders = before;
		if (result.message) toasts.show({ kind: 'error', title: m.general_save_failed(), detail: result.message });
	}
	const setPush = (on: boolean) => reminders && saveReminders({ push_reminders: String(on) }, { ...reminders, push_reminders: on });
	const setQuiet = (on: boolean, value = night) => reminders && saveReminders({ quiet: on ? value : 'off' }, { ...reminders, quiet_hours: on ? parseQuiet(value) : null });

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

	// The password, set or changed here (the pen's row leads nowhere): a dialog with the current
	// one when there is one, the new one twice, and the emailed reset for a forgotten one.
	let passwordOpen = $state(false);
	let passwords = $state({ current: '', next: '', confirm: '' });
	let passwordError = $state<{ field?: string; message: string }>();
	let savingPassword = $state(false);
	function openPassword() {
		passwords = { current: '', next: '', confirm: '' };
		passwordError = undefined;
		passwordOpen = true;
	}
	async function savePassword() {
		savingPassword = true;
		const result = await pageAction('/settings/general', 'password', data.methods.password ? passwords : { next: passwords.next, confirm: passwords.confirm });
		savingPassword = false;
		if (!result.ok) return void (passwordError = result.message ? { field: result.field, message: result.message } : undefined);
		passwordOpen = false;
		toasts.show({ kind: 'success', title: result.data.password === 'set' ? m.general_password_was_set() : m.general_password_changed() });
		await invalidate('app:general');
	}
	const fieldError = (field: string) => (passwordError?.field === field ? passwordError.message : undefined);

	type Sent = { challengeId: string; email: string; minutes: number };
	type Problem = { field?: string; message: string; ended?: boolean };
	const problemOf = (result: { ok: false; message: string; field?: string }): Problem | undefined => (result.message ? { field: result.field, message: result.message } : undefined);

	// Changing the email (the pen's Change, whose dialog it doesn't draw): the new address, then the
	// code sent to it. Nothing changes until the code goes in.
	let emailOpen = $state(false);
	let newEmail = $state('');
	let emailSent = $state<Sent | null>(null);
	let emailCode = $state('');
	let emailProblem = $state<Problem>();
	let emailBusy = $state(false);
	function openEmail() {
		newEmail = '';
		emailSent = null;
		emailCode = '';
		emailProblem = undefined;
		emailOpen = true;
	}
	async function sendEmailCode() {
		emailBusy = true;
		const result = await pageAction('/settings/general', 'emailStart', { email: newEmail });
		emailBusy = false;
		if (!result.ok) return void (emailProblem = problemOf(result));
		emailProblem = undefined;
		emailCode = '';
		emailSent = result.data.code as Sent;
	}
	async function confirmEmail(code = emailCode) {
		if (!emailSent || emailBusy) return;
		emailBusy = true;
		const result = await pageActionWithData('emailVerify', { challenge_id: emailSent.challengeId, code });
		emailBusy = false;
		if (!result.ok) {
			emailProblem = result.problem;
			emailCode = '';
			return;
		}
		emailOpen = false;
		toasts.show({ kind: 'success', title: m.general_email_changed({ email: String(result.data.email) }) });
		await invalidate('app:general');
	}

	// Deleting the account: with the password, or a code emailed to its address (an account made
	// with a code or Google has no password). A workspace only they own, with others in it, is
	// named, to hand over first.
	let deleteOpen = $state(false);
	let deleteWith = $state<'password' | 'code'>('password');
	let deletePassword = $state('');
	let deleteSent = $state<Sent | null>(null);
	let deleteCode = $state('');
	let deleteProblem = $state<Problem>();
	let soleOwner = $state<{ id: string; name: string }[]>([]);
	let deleteBusy = $state(false);
	function openDelete() {
		deleteWith = data.methods.password ? 'password' : 'code';
		deletePassword = '';
		deleteSent = null;
		deleteCode = '';
		deleteProblem = undefined;
		soleOwner = [];
		deleteOpen = true;
	}
	async function sendDeleteCode() {
		deleteBusy = true;
		const result = await pageAction('/settings/general', 'deleteCode');
		deleteBusy = false;
		if (!result.ok) return void (deleteProblem = problemOf(result));
		deleteProblem = undefined;
		deleteCode = '';
		deleteSent = result.data.code as Sent;
	}
	async function closeAccount(code = deleteCode) {
		if (deleteBusy) return;
		if (deleteWith === 'code' && !deleteSent) return void sendDeleteCode();
		deleteBusy = true;
		const fields: Record<string, string> = deleteWith === 'password' ? { password: deletePassword } : { challenge_id: deleteSent!.challengeId, code };
		// Closing signs every device out, this one too: the feed's sign-out may come first.
		leaveTo('/sign-in?closed=1');
		const result = await pageActionWithData('deleteAccount', fields);
		deleteBusy = false;
		// Closed: the action signed this device out and is on its way to sign in. Keep where to go
		// set: the feed's own sign-out may still be finishing, and it must land on the same page.
		if (result.ok || result.left) return;
		leaveTo(null);
		soleOwner = result.soleOwner ?? [];
		deleteProblem = result.problem;
		deleteCode = '';
	}

	/** A page action whose refusal carries more than a message (an ended code, the workspaces to hand over). */
	async function pageActionWithData(
		name: string,
		fields: Record<string, string>,
	): Promise<{ ok: true; data: Record<string, unknown> } | { ok: false; left?: true; problem?: Problem; soleOwner?: { id: string; name: string }[] }> {
		const body = new FormData();
		for (const [key, value] of Object.entries(fields)) body.set(key, value);
		const response = await fetch(`/settings/general?/${name}`, { method: 'POST', body, headers: { 'x-sveltekit-action': 'true' } }).catch(() => null);
		const result = response ? await response.text().then(deserialize).catch(() => null) : null;
		if (result?.type === 'success') return { ok: true, data: result.data ?? {} };
		if (result?.type === 'redirect') {
			// Signed out (closing the account): a full load, so nothing of the app stays in memory.
			location.assign(result.location);
			return { ok: false, left: true };
		}
		if (result?.type === 'failure') {
			const said = (result.data ?? {}) as { message?: string; field?: string; ended?: boolean; soleOwner?: { id: string; name: string }[] };
			return { ok: false, soleOwner: said.soleOwner, problem: said.message ? { field: said.field, message: said.message, ended: said.ended } : undefined };
		}
		return { ok: false, problem: { message: m.action_failed() } };
	}

	const others = $derived((data.devices ?? []).filter((device) => !device.current).length);
	const resetHref = $derived(`/forgot-password?email=${encodeURIComponent(data.email)}`);

	// On a phone (Flow / Mobile — You & Settings, steps 2–6: EtSi2) General is grouped rows, each
	// showing what's set: picking opens a sheet, typing (the email, the password) its own screen with
	// the tab bar out of the way, and Delete account a sheet. The same state and actions as above.
	const guess = () => data.phone;
	const phone = phoneView(guess());
	const screen = phoneScreen();
	type Sheet = 'language' | 'zone' | 'theme' | 'starter' | 'quiet' | 'google';
	let sheet = $state<Sheet | null>(null);
	const closeSheet = (which: Sheet) => () => sheet === which && (sheet = null);
	$effect(() => {
		screen.inside = phone.current && (emailOpen || passwordOpen);
		return () => (screen.inside = false);
	});
	let googleForm: HTMLFormElement | undefined = $state();
	const THEMES = $derived([
		{ value: 'system', label: m.theme_system() },
		{ value: 'light', label: m.theme_light() },
		{ value: 'dark', label: m.theme_dark() },
	]);
	const LANGUAGES = $derived([
		{ value: 'en', label: m.language_en() },
		{ value: 'id', label: m.language_id() },
	]);
	const starterName = $derived(starters.find((option) => option.value === startsWith)?.label ?? 'Nomi');

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

{#if phone.current}
	<div class="h-full overflow-y-auto">
		{#if emailOpen}
			<PhoneFormScreen
				backLabel={m.settings_general()}
				onback={() => (emailOpen = false)}
				title={emailSent ? m.general_email_code_title() : m.general_email_change_title()}
				sub={emailSent ? m.verify_sub({ email: emailSent.email, minutes: emailSent.minutes }) : m.general_email_change_body()}
				onsubmit={() => (emailSent ? confirmEmail() : sendEmailCode())}
			>
				{#if emailSent}
					<CodeInput bind:value={emailCode} invalid={emailProblem?.field === 'code'} disabled={emailBusy || !!emailProblem?.ended} autofocus oncomplete={(code) => confirmEmail(code)} />
					{#if emailProblem}<p role="alert" class="text-phone-sm text-danger">{emailProblem.message}</p>{/if}
					<span class="flex"><AuthLink label={m.general_email_other()} onclick={() => ((emailSent = null), (emailProblem = undefined))} /></span>
				{:else}
					<TextField label={m.general_email_new()} type="email" name="email" autocomplete="email" bind:value={newEmail} error={emailProblem?.field === 'email' ? emailProblem.message : undefined} required />
					{#if emailProblem && !emailProblem.field}<p role="alert" class="text-phone-sm text-danger">{emailProblem.message}</p>{/if}
				{/if}
				{#snippet action()}
					{#if emailSent}
						<PhoneButton
							label={emailProblem?.ended ? m.verify_start_over() : m.general_email_confirm()}
							disabled={emailBusy || (!emailProblem?.ended && emailCode.length < 6)}
							onclick={() => (emailProblem?.ended ? ((emailSent = null), (emailProblem = undefined)) : confirmEmail())}
						/>
					{:else}
						<PhoneButton label={m.general_email_send()} disabled={emailBusy || !newEmail.trim()} onclick={sendEmailCode} />
					{/if}
				{/snippet}
			</PhoneFormScreen>
		{:else if passwordOpen}
			<PhoneFormScreen
				backLabel={m.settings_general()}
				onback={() => (passwordOpen = false)}
				title={data.methods.password ? m.general_password_change_title() : m.general_password_set_title()}
				sub={data.methods.password ? m.general_password_change_body() : m.general_password_set_body()}
				onsubmit={savePassword}
			>
				<input type="email" name="username" autocomplete="username" value={data.email} hidden readonly />
				{#if data.methods.password}
					<TextField label={m.general_password_current()} type="password" name="current" autocomplete="current-password" bind:value={passwords.current} error={fieldError('current')} required />
				{/if}
				<TextField label={m.general_password_new()} type="password" name="next" autocomplete="new-password" helper={m.register_password_help()} bind:value={passwords.next} error={fieldError('next')} required />
				<TextField label={m.general_password_confirm()} type="password" name="confirm" autocomplete="new-password" bind:value={passwords.confirm} error={fieldError('confirm')} required />
				{#if passwordError && !passwordError.field}<p role="alert" class="text-phone-sm text-danger">{passwordError.message}</p>{/if}
				{#if data.methods.password}<span class="flex"><AuthLink label={m.general_password_forgot()} href={resetHref} quiet /></span>{/if}
				{#snippet action()}
					<PhoneButton label={m.general_password_save()} disabled={savingPassword} onclick={savePassword} />
				{/snippet}
			</PhoneFormScreen>
		{:else}
			<PhoneBackBar label={m.nav_you()} href="/settings" />
			<LargeTitle title={m.settings_general()} />
			<div class="flex flex-col gap-5 px-4 pt-1 pb-8">
				<PhoneSettingsGroup
					label={m.general_account()}
					rows={[
						{ key: 'email', icon: Mail, label: m.general_email(), meta: maskEmail(data.email), mono: true, onclick: openEmail },
						{ key: 'password', icon: Lock, label: m.general_password(), meta: data.methods.password ? m.general_password_change() : m.general_password_add(), onclick: openPassword },
						...(data.methods.configured || data.methods.google_email
							? [
									{
										key: 'google',
										icon: Globe,
										label: m.general_google(),
										sub: data.methods.google_email ?? undefined,
										meta: data.methods.google_email ? m.general_connected() : m.general_connect(),
										busy: busy === 'unlinkGoogle',
										onclick: () => (data.methods.google_email ? (sheet = 'google') : googleForm?.requestSubmit()),
									},
								]
							: []),
					]}
				/>
				<p class="-mt-3 px-1 text-phone-sm text-text-3">{m.general_google_note()}</p>
				<PhoneSettingsGroup
					label={m.general_language_region()}
					rows={[
						{ key: 'language', icon: Languages, label: m.general_language(), meta: LANGUAGES.find((option) => option.value === language)?.label, onclick: () => (sheet = 'language') },
						{ key: 'zone', icon: Clock3, label: m.general_zone(), meta: zone || m.general_zone_choose(), mono: !!zone, onclick: () => (sheet = 'zone') },
					]}
				/>
				<PhoneSettingsGroup label={m.general_appearance()} rows={[{ key: 'theme', icon: SunMoon, label: m.general_theme(), meta: THEMES.find((option) => option.value === theme)?.label, onclick: () => (sheet = 'theme') }]} />
				<PhoneSettingsGroup
					label={m.general_chat()}
					rows={[
						{ key: 'starter', icon: MessageCircle, label: m.general_starts_with(), meta: starterName, onclick: () => (sheet = 'starter') },
						{ key: 'enter', icon: CornerDownLeft, label: m.general_enter_sends(), sub: enterSends ? m.general_enter_sends_on() : m.general_enter_sends_off(), toggle: { checked: enterSends, onchange: changeEnterSends } },
					]}
				/>
				<PhoneSettingsGroup
					label={m.general_notifications()}
					rows={[
						...(reminders ? [{ key: 'push', icon: Bell, label: m.general_reminders(), sub: m.general_reminders_body(), toggle: { checked: reminders.push_reminders, onchange: setPush } }] : []),
						...(emails
							? [
									{ key: 'mail', icon: Mail, label: m.general_email(), sub: m.general_email_account(), toggle: { checked: emails.email_account, onchange: (on: boolean) => saveEmails({ email_account: on }) } },
									{ key: 'news', icon: Newspaper, label: m.general_product_news(), sub: m.general_product_news_body(), toggle: { checked: emails.email_promos, onchange: (on: boolean) => saveEmails({ email_promos: on }) } },
								]
							: []),
						...(reminders
							? [{ key: 'quiet', icon: Moon, label: m.general_quiet(), meta: reminders.quiet_hours ? `${reminders.quiet_hours.start}–${reminders.quiet_hours.end}` : m.phone_general_off(), mono: !!reminders.quiet_hours, onclick: () => (sheet = 'quiet') }]
							: []),
					]}
				/>
				{#if !reminders || !emails}<p class="-mt-3 px-1 text-phone-sm text-text-3">{!reminders ? m.general_reminders_unloaded() : m.general_emails_unloaded()}</p>{/if}
				{#if data.devices}
					<PhoneSettingsGroup
						label={m.general_sessions()}
						rows={[
							{ key: 'devices', icon: Smartphone, label: data.devices.length === 1 ? m.general_devices_one() : m.general_devices({ count: data.devices.length }), sub: devicesLine(data.devices, time.now, time.timeZone) },
							...(others > 0
								? [{ key: 'others', icon: LogOut, label: m.general_sign_out_others(), busy: busy === 'signOutOthers', onclick: () => act('signOutOthers', (done) => m.general_signed_out({ count: Number(done.signedOut ?? others) })) }]
								: []),
						]}
					/>
				{:else}
					<p class="px-1 text-phone-sm text-text-3">{m.general_devices_unloaded()}</p>
				{/if}
				<PhoneSettingsGroup rows={[{ key: 'delete', icon: Trash2, label: m.general_delete(), danger: true, onclick: openDelete }]} />
			</div>
		{/if}
	</div>
	<form bind:this={googleForm} method="POST" action="?/linkGoogle" hidden></form>

	<BottomSheet open={sheet === 'language'} title={m.general_language()} subtitle={m.general_language_body()} onclose={closeSheet('language')}>
		{#each LANGUAGES as option (option.value)}
			<SheetOption label={option.label} chosen={option.value === language} onclick={() => ((sheet = null), changeLanguage(option.value))} />
		{/each}
	</BottomSheet>
	<BottomSheet open={sheet === 'zone'} title={m.general_zone()} subtitle={m.general_zone_body()} onclose={closeSheet('zone')}>
		{#each zones as option (option.value)}
			<SheetOption label={option.label} chosen={option.value === zone} onclick={() => ((sheet = null), (zone = option.value), changeZone(option.value))} />
		{/each}
	</BottomSheet>
	<BottomSheet open={sheet === 'theme'} title={m.general_theme()} subtitle={m.general_theme_body()} onclose={closeSheet('theme')}>
		{#each THEMES as option (option.value)}
			<SheetOption label={option.label} chosen={option.value === theme} onclick={() => ((sheet = null), changeTheme(option.value))} />
		{/each}
	</BottomSheet>
	<BottomSheet open={sheet === 'starter'} title={m.general_starts_with()} subtitle={m.general_starts_with_body()} onclose={closeSheet('starter')}>
		{#each starters as option (option.value)}
			<SheetOption label={option.label} chosen={option.value === startsWith} onclick={() => ((sheet = null), (startsWith = option.value), changeStarter(option.value))}>
				{#snippet lead()}<Mascot agent={(option.value && crewOf(option.value)) || 'nomi'} size={20} live={false} />{/snippet}
			</SheetOption>
		{/each}
	</BottomSheet>
	<BottomSheet open={sheet === 'quiet'} title={m.general_quiet()} subtitle={m.phone_general_quiet_sub()} onclose={closeSheet('quiet')}>
		<SheetOption label={m.phone_general_off()} chosen={!reminders?.quiet_hours} onclick={() => ((sheet = null), setQuiet(false))} />
		{#each nights as option (option.value)}
			<SheetOption label={option.label} chosen={!!reminders?.quiet_hours && option.value === night} onclick={() => ((sheet = null), (night = option.value), setQuiet(true, option.value))} />
		{/each}
	</BottomSheet>
	<ActionSheet
		open={sheet === 'google'}
		title={m.general_google()}
		body={data.methods.google_email ? m.general_google_linked({ email: data.methods.google_email }) : undefined}
		actions={[{ label: m.general_disconnect(), danger: true, onclick: () => act('unlinkGoogle', () => m.general_google_removed()) }]}
		cancel={m.action_cancel()}
		onclose={closeSheet('google')}
	/>
	<BottomSheet bind:open={deleteOpen} title={m.general_delete_title()} subtitle={m.general_delete_what()}>
		<form
			class="phone-form flex flex-col gap-4 px-4 pb-2"
			onsubmit={(event) => {
				event.preventDefault();
				void closeAccount();
			}}
		>
			{#if soleOwner.length}
				<div role="alert" class="flex flex-col gap-1.5 rounded-lg bg-warning-bg px-3 py-2.5 text-phone-sm text-text">
					<p>{m.general_delete_sole_owner()}</p>
					<ul class="flex flex-col gap-1">
						{#each soleOwner as org (org.id)}<li><a class="text-text underline underline-offset-2" href="/settings/workspaces/{org.id}">{org.name}</a></li>{/each}
					</ul>
				</div>
			{/if}
			{#if deleteWith === 'password'}
				<input type="email" name="username" autocomplete="username" value={data.email} hidden readonly />
				<TextField label={m.auth_password()} type="password" name="password" autocomplete="current-password" helper={m.general_delete_password_hint()} bind:value={deletePassword} error={deleteProblem?.field === 'password' ? deleteProblem.message : undefined} required />
				<span class="flex"><AuthLink label={m.general_delete_use_code()} onclick={() => ((deleteWith = 'code'), (deleteProblem = undefined))} /></span>
			{:else if deleteSent}
				<p class="text-phone-sm text-text-2">{m.general_delete_code_sent({ email: deleteSent.email, minutes: deleteSent.minutes })}</p>
				<CodeInput bind:value={deleteCode} invalid={deleteProblem?.field === 'code'} disabled={deleteBusy || !!deleteProblem?.ended} />
				{#if data.methods.password}<span class="flex"><AuthLink label={m.general_delete_use_password()} onclick={() => ((deleteWith = 'password'), (deleteProblem = undefined))} /></span>{/if}
			{:else}
				<p class="text-phone-sm text-text-2">{m.general_delete_code_hint({ email: maskEmail(data.email) })}</p>
				{#if data.methods.password}<span class="flex"><AuthLink label={m.general_delete_use_password()} onclick={() => ((deleteWith = 'password'), (deleteProblem = undefined))} /></span>{/if}
			{/if}
			{#if deleteProblem && deleteProblem.field !== 'password'}<p role="alert" class="text-phone-sm text-danger">{deleteProblem.message}</p>{/if}
			<button type="submit" hidden aria-hidden="true" tabindex="-1"></button>
			{#if deleteWith === 'code' && (!deleteSent || deleteProblem?.ended)}
				<PhoneButton label={m.general_email_send()} disabled={deleteBusy} onclick={sendDeleteCode} />
			{:else}
				<PhoneButton label={m.general_delete_confirm()} variant="danger" icon={Trash2} disabled={deleteBusy} onclick={() => closeAccount()} />
			{/if}
			<PhoneButton label={m.phone_general_keep()} variant="secondary" onclick={() => (deleteOpen = false)} />
		</form>
	</BottomSheet>
{:else}
<div bind:this={scroller} onscroll={follow} class="h-full overflow-y-auto">
	<div class="flex gap-14 px-4 pt-8 pb-10 lg:px-12">
		<div class="flex max-w-220 min-w-0 flex-1 flex-col gap-7">
			<header class="flex flex-col gap-1.5">
				<h1 class="text-display font-medium tracking-display text-text">{m.settings_general()}</h1>
				<p class="text-body text-text-2">{m.general_intro()}</p>
			</header>

			<SettingsGroup id="account" title={m.general_account()} description={m.general_account_body()}>
				<SettingRow title={m.general_email()} description={maskEmail(data.email)} mono>
					{#snippet control()}<Button label={m.general_password_change()} aria-label={m.general_email_change_title()} variant="secondary" onclick={openEmail} />{/snippet}
				</SettingRow>
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
						<Button
							label={data.methods.password ? m.general_password_change() : m.general_password_add()}
							aria-label={data.methods.password ? m.general_password_change_title() : undefined}
							variant="secondary"
							onclick={openPassword}
						/>
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

			<SettingsGroup id="chat" title={m.general_chat()} description={m.general_chat_body()}>
				<SettingRow title={m.general_starts_with()} description={m.general_starts_with_body()}>
					{#snippet control()}
						<span class="flex w-32 max-w-full">
							<Select label={m.general_starts_with()} options={starters} bind:value={startsWith} onchange={changeStarter}>
								{#snippet lead()}<Mascot agent={starterMascot} size={20} />{/snippet}
							</Select>
						</span>
					{/snippet}
				</SettingRow>
				<SettingRow title={m.general_enter_sends()} description={enterSends ? m.general_enter_sends_on() : m.general_enter_sends_off()}>
					{#snippet control()}<Toggle label={m.general_enter_sends()} checked={enterSends} onchange={changeEnterSends} />{/snippet}
				</SettingRow>
			</SettingsGroup>

			<SettingsGroup id="notifications" title={m.general_notifications()} description={m.general_notifications_body()}>
				{#if reminders}
					<SettingRow title={m.general_reminders()} description={m.general_reminders_body()}>
						{#snippet control()}<Toggle label={m.general_reminders()} checked={reminders!.push_reminders} onchange={setPush} />{/snippet}
					</SettingRow>
				{/if}
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
				{#if reminders}
					<SettingRow
						title={m.general_quiet()}
						description={reminders.quiet_hours ? m.general_quiet_on({ range: `${reminders.quiet_hours.start}–${reminders.quiet_hours.end}` }) : m.general_quiet_off()}
					>
						{#snippet control()}
							<span class="flex w-30 max-w-full">
								<Select label={m.general_quiet()} options={nights} bind:value={night} mono disabled={!reminders!.quiet_hours} onchange={(value) => setQuiet(true, value)} />
							</span>
							<Toggle label={m.general_quiet()} checked={!!reminders!.quiet_hours} onchange={(on) => setQuiet(on)} />
						{/snippet}
					</SettingRow>
				{:else}
					<SettingNote text={m.general_reminders_unloaded()} />
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
				<SettingRow title={m.general_delete()} description={m.general_delete_body()}>
					{#snippet control()}<Button label={m.general_delete_button()} variant="danger" onclick={openDelete} />{/snippet}
				</SettingRow>
			</SettingsGroup>
		</div>

		<aside class="sticky top-0 hidden w-58 shrink-0 flex-col gap-5 self-start pt-24 xl:flex">
			<PageToc label={m.general_on_page()} links={groups} {active} />
			<RelatedLinks label={m.general_related()} links={[{ href: '/inbox', label: m.nav_inbox(), icon: Inbox }]} />
			<SettingsTip text={m.general_tip()} />
		</aside>
	</div>
</div>

<Dialog
	bind:open={passwordOpen}
	title={data.methods.password ? m.general_password_change_title() : m.general_password_set_title()}
	description={data.methods.password ? m.general_password_change_body() : m.general_password_set_body()}
>
	<form
		class="flex flex-col gap-4 pt-2"
		onsubmit={(event) => {
			event.preventDefault();
			void savePassword();
		}}
	>
		<!-- For the browser's password manager: whose password this is. -->
		<input type="email" name="username" autocomplete="username" value={data.email} hidden readonly />
		{#if data.methods.password}
			<TextField label={m.general_password_current()} type="password" name="current" autocomplete="current-password" bind:value={passwords.current} error={fieldError('current')} required />
		{/if}
		<TextField label={m.general_password_new()} type="password" name="next" autocomplete="new-password" helper={m.register_password_help()} bind:value={passwords.next} error={fieldError('next')} required />
		<TextField label={m.general_password_confirm()} type="password" name="confirm" autocomplete="new-password" bind:value={passwords.confirm} error={fieldError('confirm')} required />
		{#if passwordError && !passwordError.field}<p role="alert" class="text-caption text-danger">{passwordError.message}</p>{/if}
		<!-- Enter saves, as the dialog's own button does. -->
		<button type="submit" hidden aria-hidden="true" tabindex="-1"></button>
		{#if data.methods.password}<span class="flex"><AuthLink label={m.general_password_forgot()} href={resetHref} quiet /></span>{/if}
	</form>
	{#snippet actions()}
		<Button variant="secondary" label={m.action_cancel()} onclick={() => (passwordOpen = false)} />
		<Button variant="inverse" label={m.general_password_save()} loading={savingPassword} onclick={savePassword} />
	{/snippet}
</Dialog>

<Dialog
	bind:open={emailOpen}
	title={emailSent ? m.general_email_code_title() : m.general_email_change_title()}
	description={emailSent ? m.verify_sub({ email: emailSent.email, minutes: emailSent.minutes }) : m.general_email_change_body()}
>
	{#if emailSent}
		<div class="flex flex-col gap-3 pt-2">
			<CodeInput bind:value={emailCode} invalid={emailProblem?.field === 'code'} disabled={emailBusy || !!emailProblem?.ended} autofocus oncomplete={(code) => confirmEmail(code)} />
			{#if emailProblem}<p role="alert" class="text-caption text-danger">{emailProblem.message}</p>{/if}
			<span class="flex"><AuthLink label={m.general_email_other()} onclick={() => ((emailSent = null), (emailProblem = undefined))} /></span>
		</div>
	{:else}
		<form
			class="flex flex-col gap-4 pt-2"
			onsubmit={(event) => {
				event.preventDefault();
				void sendEmailCode();
			}}
		>
			<TextField label={m.general_email_new()} type="email" name="email" autocomplete="email" bind:value={newEmail} error={emailProblem?.field === 'email' ? emailProblem.message : undefined} required />
			{#if emailProblem && !emailProblem.field}<p role="alert" class="text-caption text-danger">{emailProblem.message}</p>{/if}
		</form>
	{/if}
	{#snippet actions()}
		<Button variant="secondary" label={m.action_cancel()} onclick={() => (emailOpen = false)} />
		{#if emailSent}
			<Button variant="inverse" label={emailProblem?.ended ? m.verify_start_over() : m.general_email_confirm()} loading={emailBusy} onclick={() => (emailProblem?.ended ? ((emailSent = null), (emailProblem = undefined)) : confirmEmail())} />
		{:else}
			<Button variant="inverse" label={m.general_email_send()} loading={emailBusy} onclick={sendEmailCode} />
		{/if}
	{/snippet}
</Dialog>

<Dialog bind:open={deleteOpen} title={m.general_delete_title()} description={m.general_delete_what()}>
	<form
		class="flex flex-col gap-4 pt-2"
		onsubmit={(event) => {
			event.preventDefault();
			void closeAccount();
		}}
	>
		{#if soleOwner.length}
			<div role="alert" class="flex flex-col gap-1.5 rounded-sm bg-danger-bg px-3 py-2.5 text-caption text-text">
				<p>{m.general_delete_sole_owner()}</p>
				<ul class="flex flex-col gap-1">
					{#each soleOwner as org (org.id)}<li><a class="text-text underline underline-offset-2" href="/settings/workspaces/{org.id}">{org.name}</a></li>{/each}
				</ul>
			</div>
		{/if}
		{#if deleteWith === 'password'}
			<input type="email" name="username" autocomplete="username" value={data.email} hidden readonly />
			<TextField
				label={m.auth_password()}
				type="password"
				name="password"
				autocomplete="current-password"
				helper={m.general_delete_password_hint()}
				bind:value={deletePassword}
				error={deleteProblem?.field === 'password' ? deleteProblem.message : undefined}
				required
			/>
			<span class="flex"><AuthLink label={m.general_delete_use_code()} onclick={() => ((deleteWith = 'code'), (deleteProblem = undefined))} /></span>
		{:else if deleteSent}
			<p class="text-caption text-text-2">{m.general_delete_code_sent({ email: deleteSent.email, minutes: deleteSent.minutes })}</p>
			<CodeInput bind:value={deleteCode} invalid={deleteProblem?.field === 'code'} disabled={deleteBusy || !!deleteProblem?.ended} />
			{#if data.methods.password}<span class="flex"><AuthLink label={m.general_delete_use_password()} onclick={() => ((deleteWith = 'password'), (deleteProblem = undefined))} /></span>{/if}
		{:else}
			<p class="text-caption text-text-2">{m.general_delete_code_hint({ email: maskEmail(data.email) })}</p>
			{#if data.methods.password}<span class="flex"><AuthLink label={m.general_delete_use_password()} onclick={() => ((deleteWith = 'password'), (deleteProblem = undefined))} /></span>{/if}
		{/if}
		{#if deleteProblem && deleteProblem.field !== 'password'}<p role="alert" class="text-caption text-danger">{deleteProblem.message}</p>{/if}
		<button type="submit" hidden aria-hidden="true" tabindex="-1"></button>
	</form>
	{#snippet actions()}
		<Button variant="secondary" label={m.action_cancel()} onclick={() => (deleteOpen = false)} />
		{#if deleteWith === 'code' && (!deleteSent || deleteProblem?.ended)}
			<Button variant="inverse" label={m.general_email_send()} loading={deleteBusy} onclick={sendDeleteCode} />
		{:else}
			<Button variant="danger" label={m.general_delete_confirm()} loading={deleteBusy} onclick={() => closeAccount()} />
		{/if}
	{/snippet}
</Dialog>
{/if}

<style>
	/* 16pt on a phone so iOS doesn't zoom into a field as it's focused. */
	.phone-form :global(input) {
		font-size: var(--text-phone);
	}
</style>
