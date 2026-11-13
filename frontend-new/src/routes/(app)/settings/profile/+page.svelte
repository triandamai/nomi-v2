<script lang="ts">
	import { ArrowDown, ArrowUp, GripVertical, Upload, X } from '@lucide/svelte';
	import { invalidate, invalidateAll } from '$app/navigation';
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { clock } from '$lib/clock';
	import { dayKey } from '$lib/chat/dates';
	import { pageAction } from '$lib/pageAction';
	import { crewUsage, dayMonth, duration, heatmap, identityLine, modelUsage, mostActiveDay, statTiles, usernameProblem } from '$lib/settings/profile';
	import { formatTokens } from '$lib/adapters/shell';
	import { toasts } from '$lib/ds/overlays/toasts.svelte';
	import type { MenuEntry } from '$lib/ds/overlays/MenuPanel.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import IconButton from '$lib/ds/actions/IconButton.svelte';
	import Menu from '$lib/ds/overlays/Menu.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import SectionLabel from '$lib/ds/status/SectionLabel.svelte';
	import PreferenceRow from '$lib/ds/inbox/PreferenceRow.svelte';
	import ProfileHeader from '$lib/ds/profile/ProfileHeader.svelte';
	import ProfileAvatar from '$lib/ds/profile/ProfileAvatar.svelte';
	import StatStrip from '$lib/ds/profile/StatStrip.svelte';
	import ActivityHeatmap from '$lib/ds/profile/ActivityHeatmap.svelte';
	import StatPanel from '$lib/ds/profile/StatPanel.svelte';
	import UsageBars from '$lib/ds/profile/UsageBars.svelte';
	import CrewUsage from '$lib/ds/profile/CrewUsage.svelte';
	import ShowcaseCard from '$lib/ds/profile/ShowcaseCard.svelte';
	import ShowcaseAdd from '$lib/ds/profile/ShowcaseAdd.svelte';
	import SaveBar from '$lib/ds/profile/SaveBar.svelte';

	// Settings › Profile (App / Settings / Profile U9Gnrg): what the person has done (their numbers,
	// a year of tokens, the models and crew they use) beside their public profile (photo, name,
	// username, what's shown) and their showcase, saved together. The photo saves on its own.
	// Left out (AGENTS.md §9): the heatmap's Daily / Weekly / Cumulative switch (only Daily is
	// drawn), the Insights panel and the hours in "Most active" (the backend has neither:
	// BACKEND_TODO.md), and the top bar's Preview public profile and Copy link (v2 has no public
	// profile page yet).
	let { data } = $props();

	const SHOWCASE_MAX = 3;
	const language = getLocale();
	const time = clock();
	const profile = $derived(data.profile);
	const name = $derived(profile.display_name || profile.username || profile.email.split('@')[0]);
	// Activity days are counted in the person's saved zone, UTC when they saved none (the backend's rule).
	const today = $derived(dayKey(time.now, data.timeZone ?? 'UTC'));
	const map = $derived(heatmap(profile.activity, today, language));
	const busiest = $derived(mostActiveDay(profile.activity, language));
	const joined = $derived(dayMonth(new Date(profile.joined_at), language));
	const weekdays = $derived(
		[19, 21, 23].map((day) => new Intl.DateTimeFormat(language === 'id' ? 'id-ID' : 'en-GB', { weekday: 'short', timeZone: 'UTC' }).format(Date.UTC(2026, 9, day))) as [string, string, string],
	);

	// The form, from the profile as saved; Discard and a save put it back to that.
	type Featured = (typeof profile.showcase_projects)[number];
	const saved = () => ({
		name: profile.display_name ?? '',
		username: profile.username ?? '',
		showStats: profile.show_stats_publicly,
		showCrew: profile.show_crew_publicly,
		showcase: [...profile.showcase_projects] as Featured[],
	});
	let form = $state(saved());
	let saving = $state(false);
	let failed = $state<{ field?: string; message: string }>();
	$effect.pre(() => {
		// A new profile from the server (saved here, or the photo changed) starts the form again.
		void profile;
		form = saved();
	});

	const changes = $derived(
		[
			form.name.trim() !== (profile.display_name ?? ''),
			form.username.trim().replace(/^@/, '') !== (profile.username ?? ''),
			form.showStats !== profile.show_stats_publicly,
			form.showCrew !== profile.show_crew_publicly,
			form.showcase.map((one) => one.id).join() !== profile.showcase_projects.map((one) => one.id).join(),
		].filter(Boolean).length,
	);

	// The username, checked as it's typed (after a pause): its rules here, whether it's free there.
	let check = $state<{ name: string; available: boolean; reason: string | null }>();
	const wanted = $derived(form.username.trim().replace(/^@/, ''));
	const link = $derived(`${page.url.host}/@${wanted}`);
	$effect(() => {
		const name = wanted;
		if (!name || name === profile.username || usernameProblem(name)) return;
		const wait = setTimeout(async () => {
			const answer = await fetch(`/settings/profile/username?name=${encodeURIComponent(name)}`).then((response) => (response.ok ? response.json() : null)).catch(() => null);
			if (answer) check = { name, ...answer };
		}, 400);
		return () => clearTimeout(wait);
	});
	const usernameError = $derived(
		failed?.field === 'username' && failed.message ? failed.message : usernameProblem(wanted) ?? (check?.name === wanted && !check.available ? (check.reason ?? m.profile_username_taken()) : undefined),
	);
	const usernameHelper = $derived(
		!wanted ? m.profile_username_help() : wanted === profile.username ? m.profile_username_yours({ link }) : check?.name === wanted && check.available ? m.profile_username_free({ link }) : m.profile_username_help(),
	);

	async function save() {
		failed = undefined;
		saving = true;
		const result = await pageAction('/settings/profile', 'save', {
			display_name: form.name,
			username: form.username,
			show_stats_publicly: String(form.showStats),
			show_crew_publicly: String(form.showCrew),
			showcase: JSON.stringify(form.showcase.map((one) => one.id)),
		});
		saving = false;
		if (!result.ok) {
			failed = { field: result.field, message: result.message };
			if (result.message && result.field !== 'username') toasts.show({ kind: 'error', title: m.action_failed(), detail: result.message });
			return;
		}
		toasts.show({ kind: 'success', title: m.profile_saved() });
		await invalidate('app:profile');
		await invalidate('app:shell');
	}

	// The photo: chosen, sent as it is to the backend, which keeps it on the profile.
	let picker: HTMLInputElement | undefined = $state();
	let uploading = $state(false);

	async function upload(file: File, again = true): Promise<void> {
		uploading = true;
		const response = await fetch('/profile/avatar', { method: 'POST', headers: { 'content-type': file.type || 'application/octet-stream' }, body: file }).catch(() => null);
		if (response?.status === 401 && again) {
			// The sign-in ran out: reloading renews it; then once more.
			await invalidateAll();
			return upload(file, false);
		}
		uploading = false;
		if (!response?.ok) {
			const said = response ? await response.text().catch(() => '') : '';
			toasts.show({ kind: 'error', title: m.profile_photo_failed(), detail: said || undefined });
			return;
		}
		toasts.show({ kind: 'success', title: m.profile_photo_saved() });
		await invalidate('app:profile');
		await invalidate('app:shell');
	}

	async function removePhoto() {
		const result = await pageAction('/settings/profile', 'removePhoto');
		if (!result.ok) {
			if (result.message) toasts.show({ kind: 'error', title: m.profile_photo_failed(), detail: result.message });
			return;
		}
		toasts.show({ kind: 'success', title: m.profile_photo_removed() });
		await invalidate('app:profile');
		await invalidate('app:shell');
	}

	// The showcase: up to three live projects, in the order they show.
	const addable = $derived(data.live.filter((project) => !form.showcase.some((one) => one.id === project.id)));

	function move(index: number, by: -1 | 1) {
		const next = [...form.showcase];
		[next[index], next[index + by]] = [next[index + by], next[index]];
		form.showcase = next;
	}

	function showcaseMenu(index: number): MenuEntry[] {
		return [
			...(index > 0 ? [{ label: m.profile_move_up(), icon: ArrowUp, onselect: () => move(index, -1) }] : []),
			...(index < form.showcase.length - 1 ? [{ label: m.profile_move_down(), icon: ArrowDown, onselect: () => move(index, 1) }] : []),
			{ label: m.profile_remove_feature(), icon: X, danger: true, onselect: () => (form.showcase = form.showcase.filter((_, at) => at !== index)) },
		];
	}

	const candidates = $derived<MenuEntry[]>(
		addable.map((project) => ({
			label: project.name,
			onselect: () => (form.showcase = [...form.showcase, { id: project.id, name: project.name, description: null, status: 'live', deploy_url: null, build_duration_secs: null }]),
		})),
	);
</script>

<svelte:head><title>{m.settings_profile()} · {m.app_name()}</title></svelte:head>

<div class="flex h-full flex-col overflow-y-auto xl:flex-row xl:overflow-hidden">
	<div class="flex min-w-0 flex-1 flex-col gap-6 px-4 pt-7 pb-6 lg:px-8 xl:overflow-y-auto">
		<ProfileHeader {name} line={identityLine(profile, language)} avatar={profile.avatar_url ?? undefined} />
		<StatStrip tiles={statTiles(profile.stats, profile.joined_at, language)} />
		<section aria-labelledby="activity" class="flex flex-col gap-3">
			<header class="flex min-h-7.5 items-center gap-2.5">
				<h2 id="activity" class="text-ui font-medium text-text">{m.profile_activity()}</h2>
				<span class="font-mono text-label text-text-3">{m.profile_activity_total({ total: formatTokens(map.total) })}</span>
			</header>
			<ActivityHeatmap
				label={m.profile_activity_total({ total: formatTokens(map.total) })}
				weeks={map.weeks}
				months={map.months}
				days={weekdays}
				note={busiest ? m.profile_activity_note({ day: busiest, date: joined }) : m.profile_joined({ date: joined })}
				less={m.profile_less()}
				more={m.profile_more()}
			/>
		</section>
		<div class="grid gap-4 md:grid-cols-2">
			<StatPanel title={m.profile_models()} meta={m.profile_models_meta()}>
				<UsageBars bars={modelUsage(profile.top_models)} empty={m.profile_models_empty()} />
			</StatPanel>
			<StatPanel title={m.profile_crew()} meta={m.profile_crew_meta()}>
				<CrewUsage crew={crewUsage(profile.crew_usage)} empty={m.profile_crew_empty()} />
			</StatPanel>
		</div>
	</div>

	<aside class="flex shrink-0 flex-col border-t border-border bg-bg xl:w-90 xl:overflow-y-auto xl:border-t-0 xl:border-l">
		<form
			class="flex flex-col gap-3.5 border-b border-border px-4 pt-4.5 pb-4.25 xl:pr-5 xl:pl-4.75"
			onsubmit={(event) => {
				event.preventDefault();
				void save();
			}}
		>
			<SectionLabel label={m.profile_public()} />
			<div class="flex items-center gap-3">
				<ProfileAvatar {name} src={profile.avatar_url ?? undefined} size={40} />
				<Button label={m.profile_change_photo()} variant="secondary" iconStart={Upload} loading={uploading} onclick={() => picker?.click()} />
				{#if profile.avatar_url}<Button label={m.profile_remove_photo()} variant="ghost" onclick={removePhoto} />{/if}
				<input
					bind:this={picker}
					type="file"
					accept="image/png,image/jpeg,image/webp,image/gif"
					hidden
					onchange={(event) => {
						const file = event.currentTarget.files?.[0];
						event.currentTarget.value = '';
						if (file) void upload(file);
					}}
				/>
			</div>
			<TextField label={m.profile_display_name()} name="display_name" bind:value={form.name} autocomplete="name" />
			<TextField label={m.profile_username()} name="username" bind:value={form.username} mono autocomplete="username" helper={usernameHelper} error={usernameError} oninput={() => (failed = undefined)} />
			<PreferenceRow title={m.profile_show_stats()} description={m.profile_show_stats_body()} checked={form.showStats} onchange={(on) => (form.showStats = on)} />
			<PreferenceRow title={m.profile_show_crew()} description={m.profile_show_crew_body()} checked={form.showCrew} onchange={(on) => (form.showCrew = on)} />
			<SaveBar
				pending={changes === 1 ? m.profile_unsaved_one() : m.profile_unsaved({ count: changes })}
				dirty={changes > 0}
				{saving}
				discard={m.profile_discard()}
				save={m.profile_save()}
				ondiscard={() => {
					form = saved();
					failed = undefined;
				}}
			/>
		</form>

		<section aria-labelledby="showcase" class="flex flex-col gap-3 px-4 py-4.5 xl:pr-5 xl:pl-4.75">
			<h2 id="showcase" class="flex items-center justify-between font-mono text-label font-normal text-text-3">
				<span class="tracking-label uppercase">{m.profile_showcase()}</span>
				<span>{m.profile_showcase_count({ count: form.showcase.length })}</span>
			</h2>
			{#each form.showcase as project, index (project.id)}
				<ShowcaseCard
					name={project.name}
					status={m.profile_live()}
					meta={project.build_duration_secs ? m.profile_built({ time: duration(project.build_duration_secs) }) : m.profile_built_plain()}
					href={project.deploy_url ?? undefined}
				>
					{#snippet tools()}
						<Menu items={showcaseMenu(index)} align="end">
							{#snippet trigger(props)}<IconButton icon={GripVertical} label={m.profile_showcase_tools({ name: project.name })} variant="ghost" {...props} />{/snippet}
						</Menu>
					{/snippet}
				</ShowcaseCard>
			{/each}
			{#if form.showcase.length < SHOWCASE_MAX}
				{#if candidates.length}
					<Menu items={candidates}>
						{#snippet trigger(props)}<ShowcaseAdd label={m.profile_showcase_add()} {...props} />{/snippet}
					</Menu>
				{:else}
					<ShowcaseAdd label={m.profile_showcase_add()} disabled aria-describedby="showcase-none" />
					<p id="showcase-none" class="text-caption text-text-3">{m.profile_showcase_none()}</p>
				{/if}
			{/if}
			<p class="text-caption text-text-3">{m.profile_showcase_note()}</p>
		</section>
	</aside>
</div>
