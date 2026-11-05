<script lang="ts">
	import { untrack } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import { enhance } from '$app/forms';
	import { invalidateAll } from '$app/navigation';
	import { ArrowRight, Eye, Flame, Link2, Moon, Plus, Sparkles, X } from '@lucide/svelte';
	import Dialog from '$lib/components/m3/Dialog.svelte';
	import Button from '$lib/components/ds/actions/Button.svelte';
	import IconButton from '$lib/components/ds/actions/IconButton.svelte';
	import Toggle from '$lib/components/ds/inputs/Toggle.svelte';
	import Mascot from '$lib/components/ds/agents/Mascot.svelte';
	import Status from '$lib/components/ds/status/Status.svelte';
	import Icon from '$lib/components/ds/foundations/Icon.svelte';
	import type { CrewId } from '$lib/components/ds/agents/crew';
	import TokenActivityHeatmap from '$lib/components/profile/TokenActivityHeatmap.svelte';
	import type { ActionData, PageData } from './$types';
	import type { ShowcaseProject, ProjectSummary } from '$lib/types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	// Initial values for dirty tracking derived from data.profile
	const initialDisplayName = $derived(data.profile?.display_name ?? '');
	const initialUsername = $derived(data.profile?.username ?? '');
	const initialAvatarUrl = $derived(data.profile?.avatar_url ?? '');
	const initialLocation = $derived(data.profile?.location ?? 'Jakarta');
	const initialShowStats = $derived(data.profile?.show_stats_publicly ?? true);
	const initialShowCrew = $derived(data.profile?.show_crew_publicly ?? false);
	const initialShowcaseIds = $derived((data.profile?.showcase_projects ?? []).map((p) => p.id));

	// Form states
	let displayName = $state('');
	let username = $state('');
	let avatarUrl = $state('');
	let location = $state('Jakarta');
	let showStatsPublicly = $state(true);
	let showCrewPublicly = $state(false);
	let showcaseProjects = $state<ShowcaseProject[]>([]);

	let showAddProjectModal = $state(false);
	let copiedLink = $state(false);
	let usernameAvailable = $state<boolean | null>(null);
	let usernameChecking = $state(false);
	let usernameDebounceTimer: ReturnType<typeof setTimeout> | null = null;
	let showAllCrew = $state(false);

	// Synchronize form states whenever data.profile updates, without reactive dependency loops
	$effect(() => {
		const profile = data.profile;
		untrack(() => {
			displayName = profile?.display_name ?? '';
			username = profile?.username ?? '';
			avatarUrl = profile?.avatar_url ?? '';
			location = profile?.location ?? 'Jakarta';
			showStatsPublicly = profile?.show_stats_publicly ?? true;
			showCrewPublicly = profile?.show_crew_publicly ?? false;
			showcaseProjects = profile?.showcase_projects ?? [];
		});
	});

	// Dirty tracking
	const isDirty = $derived(
		displayName !== initialDisplayName ||
		username !== initialUsername ||
		avatarUrl !== initialAvatarUrl ||
		location !== initialLocation ||
		showStatsPublicly !== initialShowStats ||
		showCrewPublicly !== initialShowCrew ||
		JSON.stringify(showcaseProjects.map((p) => p.id)) !== JSON.stringify(initialShowcaseIds)
	);

	const unsavedCount = $derived(
		(displayName !== initialDisplayName ? 1 : 0) +
		(username !== initialUsername ? 1 : 0) +
		(avatarUrl !== initialAvatarUrl ? 1 : 0) +
		(location !== initialLocation ? 1 : 0) +
		(showStatsPublicly !== initialShowStats ? 1 : 0) +
		(showCrewPublicly !== initialShowCrew ? 1 : 0) +
		(JSON.stringify(showcaseProjects.map((p) => p.id)) !== JSON.stringify(initialShowcaseIds) ? 1 : 0)
	);

	function discardChanges() {
		displayName = initialDisplayName;
		username = initialUsername;
		avatarUrl = initialAvatarUrl;
		location = initialLocation;
		showStatsPublicly = initialShowStats;
		showCrewPublicly = initialShowCrew;
		showcaseProjects = data.profile?.showcase_projects ?? [];
		usernameAvailable = null;
	}

	function handleUsernameInput(e: Event) {
		const target = e.target as HTMLInputElement;
		const raw = target.value.replace(/^@/, '').trim();
		username = raw;

		if (usernameDebounceTimer) clearTimeout(usernameDebounceTimer);
		if (!raw || raw === initialUsername) {
			usernameAvailable = null;
			usernameChecking = false;
			return;
		}

		usernameChecking = true;
		usernameDebounceTimer = setTimeout(async () => {
			try {
				const res = await fetch(`/api/profile/check-username?username=${encodeURIComponent(raw)}`);
				if (res.ok) {
					const body = (await res.json()) as { available: boolean };
					usernameAvailable = body.available;
				} else {
					usernameAvailable = false;
				}
			} catch {
				usernameAvailable = null;
			} finally {
				usernameChecking = false;
			}
		}, 300);
	}

	let uploading = $state(false);
	let uploadError = $state<string | null>(null);
	let fileInput: HTMLInputElement | undefined = $state();
	const MAX_AVATAR_BYTES = 5 * 1024 * 1024;

	async function handleFileSelected(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const file = input.files?.[0];
		input.value = '';
		if (!file) return;
		if (file.size > MAX_AVATAR_BYTES) {
			uploadError = m.profile_avatar_too_big();
			return;
		}

		uploading = true;
		uploadError = null;
		try {
			let response = await fetch('/profile/avatar', {
				method: 'POST',
				body: file,
				headers: { 'Content-Type': file.type },
			});
			if (response.status === 401) {
				const refreshed = await fetch('/attachments/session', { method: 'POST' }).catch(() => null);
				if (refreshed?.ok) {
					response = await fetch('/profile/avatar', {
						method: 'POST',
						body: file,
						headers: { 'Content-Type': file.type },
					});
				}
			}
			if (!response.ok) {
				uploadError = (await response.text()) || m.profile_upload_failed();
				return;
			}
			const json = (await response.json()) as { avatar_url: string };
			avatarUrl = json.avatar_url;
			await invalidateAll();
		} catch {
			uploadError = m.profile_upload_failed();
		} finally {
			uploading = false;
		}
	}

	function removeAvatar() {
		avatarUrl = '';
	}

	async function copyProfileLink() {
		const userSlug = username || 'me';
		const url = `${window.location.origin}/@${userSlug}`;
		try {
			await navigator.clipboard.writeText(url);
			copiedLink = true;
			setTimeout(() => {
				copiedLink = false;
			}, 2000);
		} catch {
			// fallback
		}
	}

	function removeShowcaseProject(id: string) {
		showcaseProjects = showcaseProjects.filter((p) => p.id !== id);
	}

	function addShowcaseProject(project: ProjectSummary) {
		if (showcaseProjects.length >= 3) return;
		if (showcaseProjects.some((p) => p.id === project.id)) return;
		showcaseProjects = [
			...showcaseProjects,
			{
				id: project.id,
				name: project.name,
				description: project.description ?? null,
				status: project.status,
				deploy_url: null,
				build_duration_secs: null,
			},
		];
		showAddProjectModal = false;
	}

	// Calculations for stats
	const lifetimeTokens = $derived(data.profile?.stats?.lifetime_tokens ?? 186400);
	const peakDayTokens = $derived(data.profile?.stats?.peak_day_tokens ?? 14200);
	const peakDayDate = $derived(data.profile?.stats?.peak_day_date ?? '2026-08-02');
	const longestTaskSecs = $derived(data.profile?.stats?.longest_task_secs ?? 252);
	const longestTaskName = $derived(data.profile?.stats?.longest_task_name ?? 'salah tracker');
	const longestStreakDays = $derived(data.profile?.stats?.longest_streak_days ?? 9);
	const longestStreakRange = $derived(data.profile?.stats?.longest_streak_range ?? '1-9 Aug');
	const currentStreakDays = $derived(data.profile?.stats?.current_streak_days ?? 3);
	const bestStreakTarget = $derived(data.profile?.stats?.best_streak_target ?? 9);

	// Formatting helpers
	function formatTokens(n: number): string {
		if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + 'M';
		if (n >= 1_000) return (n / 1_000).toFixed(1) + 'K';
		return n.toLocaleString();
	}

	function formatDuration(secs: number): string {
		const m = Math.floor(secs / 60);
		const s = secs % 60;
		if (m > 0) return `${m}m ${s}s`;
		return `${s}s`;
	}

	function formatJoinedDate(dateStr?: string): string {
		if (!dateStr) return '14 Jul 2026';
		try {
			const d = new Date(dateStr);
			return d.toLocaleDateString('en-US', { day: 'numeric', month: 'short', year: 'numeric' });
		} catch {
			return '14 Jul 2026';
		}
	}

	const joinedFormatted = $derived(formatJoinedDate(data.profile?.joined_at));

	// Initials for avatar
	const initials = $derived.by(() => {
		const name = displayName || data.profile?.email || 'Nomi User';
		return name
			.trim()
			.split(/\s+/)
			.slice(0, 2)
			.map((p) => p[0]?.toUpperCase() ?? '')
			.join('') || 'MP';
	});

	// Models breakdown (fallback to mock if empty)
	const topModels = $derived.by(() => {
		if (data.profile?.top_models && data.profile.top_models.length > 0) {
			return data.profile.top_models;
		}
		return [
			{ label: 'deepseek-flash', tokens: 13500, percentage: 62 },
			{ label: 'gemini-flash', tokens: 5200, percentage: 24 },
			{ label: 'qwen-coder', tokens: 2000, percentage: 9 },
			{ label: 'llama · local', tokens: 1100, percentage: 5 },
		];
	});

	// Crew usage list (fallback to mock if empty)
	const crewUsage = $derived.by(() => {
		if (data.profile?.crew_usage && data.profile.crew_usage.length > 0) {
			return data.profile.crew_usage;
		}
		return [
			{ agent_id: 'nomi', name: 'Nomi', count: 142, kind: 'chats' },
			{ agent_id: 'dana', name: 'Dana', count: 31, kind: 'chats' },
			{ agent_id: 'kala', name: 'Kala', count: 18, kind: 'chats' },
			{ agent_id: 'koda', name: 'Koda', count: 2, kind: 'projects' },
		];
	});

	// Maximum crew count for bar scaling
	const maxCrewCount = $derived(Math.max(1, ...crewUsage.map((c) => c.count)));

	// Available projects for showcase modal
	const availableProjects = $derived(
		(data.projects ?? []).filter((p) => !showcaseProjects.some((sp) => sp.id === p.id))
	);
</script>

<div class="min-h-[100dvh] bg-bg text-text px-4 py-5 sm:px-8 sm:py-7 font-sans pb-[calc(env(safe-area-inset-bottom,0px)+2rem)]">
	<div class="max-w-6xl mx-auto flex flex-col gap-6">
		<!-- Top Navigation & Action Bar -->
		<div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
			<div class="flex items-center gap-2 text-ui text-text-2">
				<a href="/settings" class="hover:text-text transition-colors">Settings</a>
				<span class="text-caption text-text-3">›</span>
				<span class="text-text font-medium">Profile</span>
			</div>

			<div class="flex items-center gap-2">
				<Button
					variant="secondary"
					label="Preview public profile"
					iconStart={Eye}
					href="/@{username || 'me'}"
					external
				/>
				<Button
					variant="secondary"
					label={copiedLink ? 'Copied link!' : 'Copy link'}
					iconStart={Link2}
					onclick={copyProfileLink}
				/>
			</div>
		</div>

		<!-- Error banners if any -->
		{#if form?.error}
			<div class="rounded-lg bg-danger-bg border border-danger-border p-3 text-caption text-danger">
				{form.error}
			</div>
		{/if}
		{#if uploadError}
			<div class="rounded-lg bg-danger-bg border border-danger-border p-3 text-caption text-danger">
				{uploadError}
			</div>
		{/if}

		<!-- Main Layout Grid: Left (2/3) + Right (1/3) -->
		<div class="grid grid-cols-1 lg:grid-cols-3 gap-6 items-start">
			<!-- LEFT COLUMN: Profile info, stats, heatmap, models, crew, insights -->
			<div class="lg:col-span-2 flex flex-col gap-6 min-w-0">
				<!-- Profile Header -->
				<div class="flex items-center gap-4">
					{#if avatarUrl}
						<img
							src={avatarUrl}
							alt={displayName || 'Avatar'}
							class="w-16 h-16 sm:w-20 sm:h-20 rounded-2xl object-cover border border-border bg-surface-2 shadow-sm shrink-0"
						/>
					{:else}
						<div class="w-16 h-16 sm:w-20 sm:h-20 rounded-2xl bg-surface-2 border border-border flex items-center justify-center text-heading font-bold text-text shadow-sm shrink-0 font-mono">
							{initials}
						</div>
					{/if}
					<div class="flex flex-col min-w-0">
						<h1 class="text-display font-bold text-text tracking-tight truncate">
							{displayName || 'Mario Prasetyo'}
						</h1>
						<div class="flex flex-wrap items-center gap-1.5 text-caption text-text-2 mt-1">
							<span>@{username || 'mario'}</span>
							<span class="text-text-3">·</span>
							<span>{location || 'Jakarta'}</span>
							<span class="text-text-3">·</span>
							<span>joined {joinedFormatted}</span>
							<span class="text-text-3">·</span>
							<span>{data.profile?.plan_name || 'Free plan'}</span>
						</div>
					</div>
				</div>

				<!-- 5 Metric Stat Cards -->
				<div class="grid grid-cols-2 sm:grid-cols-5 gap-3">
					<!-- 1. Lifetime tokens -->
					<div class="rounded-xl border border-border bg-surface p-3.5 sm:p-4 flex flex-col justify-between">
						<div>
							<div class="text-display font-bold text-text tracking-tight">{formatTokens(lifetimeTokens)}</div>
							<div class="text-ui text-text-2 mt-1 font-medium">Lifetime tokens</div>
						</div>
						<div class="text-caption text-text-3 mt-2 truncate">since {joinedFormatted.slice(0, 6)}</div>
					</div>

					<!-- 2. Peak day -->
					<div class="rounded-xl border border-border bg-surface p-3.5 sm:p-4 flex flex-col justify-between">
						<div>
							<div class="text-display font-bold text-text tracking-tight">{formatTokens(peakDayTokens)}</div>
							<div class="text-ui text-text-2 mt-1 font-medium">Peak day</div>
						</div>
						<div class="text-caption text-text-3 mt-2 truncate">{peakDayDate}</div>
					</div>

					<!-- 3. Longest task -->
					<div class="rounded-xl border border-border bg-surface p-3.5 sm:p-4 flex flex-col justify-between">
						<div>
							<div class="text-display font-bold text-text tracking-tight">{formatDuration(longestTaskSecs)}</div>
							<div class="text-ui text-text-2 mt-1 font-medium">Longest task</div>
						</div>
						<div class="text-caption text-text-3 mt-2 truncate">Koda · {longestTaskName}</div>
					</div>

					<!-- 4. Longest streak -->
					<div class="rounded-xl border border-border bg-surface p-3.5 sm:p-4 flex flex-col justify-between">
						<div>
							<div class="text-display font-bold text-text tracking-tight">{longestStreakDays} days</div>
							<div class="text-ui text-text-2 mt-1 font-medium">Longest streak</div>
						</div>
						<div class="text-caption text-text-3 mt-2 truncate">{longestStreakRange}</div>
					</div>

					<!-- 5. Current streak -->
					<div class="rounded-xl border border-border bg-surface p-3.5 sm:p-4 flex flex-col justify-between col-span-2 sm:col-span-1">
						<div>
							<div class="text-display font-bold text-text tracking-tight flex items-center gap-1.5">
								{currentStreakDays} days <span class="text-accent text-body inline-flex"><Icon icon={Flame} size={20} /></span>
							</div>
							<div class="text-ui text-text-2 mt-1 font-medium">Current streak</div>
						</div>
						<div class="text-caption text-text-3 mt-2 truncate">
							best is {bestStreakTarget} — {Math.max(0, bestStreakTarget - currentStreakDays)} to go
						</div>
					</div>
				</div>

				<!-- Token Activity Heatmap -->
				<TokenActivityHeatmap
					activity={data.profile?.activity ?? []}
					joinedDate={joinedFormatted}
					totalTokens={lifetimeTokens}
				/>

				<!-- Bottom 3 Cards (Models, Crew, Insights) -->
				<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
					<!-- Models you use most -->
					<div class="rounded-xl border border-border bg-surface p-4 sm:p-5 flex flex-col justify-between gap-3">
						<div class="flex items-center justify-between">
							<h3 class="text-ui font-semibold text-text tracking-tight">Models you use most</h3>
							<span class="text-caption text-text-3">last 30 days</span>
						</div>
						<div class="flex flex-col gap-2.5">
							{#each topModels as model}
								<div class="flex flex-col gap-1">
									<div class="flex items-center justify-between text-caption">
										<span class="text-text font-mono truncate max-w-[120px]">{model.label}</span>
										<span class="text-text-2">{formatTokens(model.tokens)} · {model.percentage}%</span>
									</div>
									<div class="w-full h-1.5 rounded-full bg-surface-2 overflow-hidden">
										<div class="h-full rounded-full bg-accent" style="width: {Math.min(100, Math.max(4, model.percentage))}%;"></div>
									</div>
								</div>
							{/each}
						</div>
					</div>

					<!-- Your crew -->
					<div class="rounded-xl border border-border bg-surface p-4 sm:p-5 flex flex-col justify-between gap-3">
						<div class="flex items-center justify-between">
							<h3 class="text-ui font-semibold text-text tracking-tight">Your crew</h3>
							<span class="text-caption text-text-3">chats</span>
						</div>
						<div class="flex flex-col gap-2.5">
							{#each crewUsage.slice(0, showAllCrew ? undefined : 4) as member}
								<div class="flex items-center gap-2">
									<Mascot agent={member.agent_id as CrewId} size={20} />
									<span class="text-ui text-text font-medium w-14 truncate">{member.name}</span>
									<div class="flex-1 h-1.5 rounded-full bg-surface-2 overflow-hidden">
										<div
											class="h-full rounded-full"
											style="background: var(--crew-{member.agent_id}, var(--color-accent)); width: {Math.min(100, Math.max(6, (member.count / maxCrewCount) * 100))}%;"
										></div>
									</div>
									<span class="text-caption text-text-2 text-right min-w-[50px]">
										{member.count} {member.kind === 'projects' ? 'proj' : ''}
									</span>
								</div>
							{/each}
						</div>
						{#if crewUsage.length > 4 || !showAllCrew}
							<button
								type="button"
								class="text-left text-caption text-text-3 hover:text-text transition-colors pt-1 cursor-pointer"
								onclick={() => (showAllCrew = !showAllCrew)}
							>
								{showAllCrew ? 'Show less' : '+6 more agents'}
							</button>
						{/if}
					</div>

					<!-- Insights -->
					<div class="rounded-xl border border-border bg-surface p-4 sm:p-5 flex flex-col justify-between gap-3">
						<div>
							<h3 class="text-ui font-semibold text-text tracking-tight">Insights</h3>
						</div>
						<div class="flex flex-col gap-4 text-ui">
							<!-- Insight 1 -->
							<div class="flex flex-col gap-1">
								<div class="flex items-center gap-1.5 font-semibold text-text text-ui">
									<span class="text-warning"><Icon icon={Sparkles} size={15} /></span>
									<span>Save ~18% on Nomi</span>
								</div>
								<p class="text-caption text-text-2 leading-relaxed">
									gemini-flash answered 9 of your last 10 quick questions just as well.
								</p>
								<a href="/models" class="text-caption text-accent hover:underline font-medium inline-flex items-center gap-1 mt-0.5">
									Switch default →
								</a>
							</div>

							<!-- Insight 2 -->
							<div class="flex flex-col gap-1 pt-2 border-t border-border">
								<div class="flex items-center gap-1.5 font-semibold text-text text-ui">
									<span class="text-accent"><Icon icon={Moon} size={15} /></span>
									<span>You work late</span>
								</div>
								<p class="text-caption text-text-2 leading-relaxed">
									68% of your tokens land after 21:00 WIB. Kala can hold reminders until 08:00.
								</p>
								<a href="/reminders" class="text-caption text-accent hover:underline font-medium inline-flex items-center gap-1 mt-0.5">
									Set quiet hours →
								</a>
							</div>
						</div>
					</div>
				</div>
			</div>

			<!-- RIGHT COLUMN: Settings Form & Showcase -->
			<div class="lg:col-span-1 flex flex-col gap-6">
				<!-- PUBLIC PROFILE Card Form -->
				<div class="rounded-xl border border-border bg-surface p-5 flex flex-col gap-4">
					<h2 class="text-label font-semibold tracking-wider text-text-3 uppercase">PUBLIC PROFILE</h2>

					<!-- Photo row -->
					<div class="flex items-center gap-3">
						{#if avatarUrl}
							<img src={avatarUrl} alt="Avatar" class="w-12 h-12 rounded-xl object-cover border border-border bg-surface-2" />
						{:else}
							<div class="w-12 h-12 rounded-xl bg-surface-2 border border-border flex items-center justify-center text-ui font-bold text-text font-mono">
								{initials}
							</div>
						{/if}
						<div class="flex items-center gap-2">
							<input
								bind:this={fileInput}
								type="file"
								accept="image/png,image/jpeg,image/webp,image/gif"
								class="hidden"
								onchange={handleFileSelected}
							/>
							<Button
								variant="secondary"
								label={uploading ? 'Uploading...' : 'Change photo'}
								onclick={() => fileInput?.click()}
								disabled={uploading}
							/>
							{#if avatarUrl}
								<Button
									variant="ghost"
									label="Remove"
									onclick={removeAvatar}
								/>
							{/if}
						</div>
					</div>

					<form
						method="POST"
						action="?/updateProfile"
						use:enhance={() => {
							return async ({ update }) => {
								await update({ reset: false });
								await invalidateAll();
							};
						}}
						class="flex flex-col gap-4"
					>
						<input type="hidden" name="avatar_url" value={avatarUrl} />
						<input type="hidden" name="show_stats_publicly" value={showStatsPublicly ? 'true' : 'false'} />
						<input type="hidden" name="show_crew_publicly" value={showCrewPublicly ? 'true' : 'false'} />
						<input type="hidden" name="showcase_project_ids" value={JSON.stringify(showcaseProjects.map((p) => p.id))} />

						<!-- Display name -->
						<div class="flex flex-col gap-1.5">
							<label for="display_name" class="text-label font-medium text-text-2">Display name</label>
							<input
								id="display_name"
								name="display_name"
								type="text"
								bind:value={displayName}
								class="w-full rounded-md border border-border bg-surface-2 px-3 py-2 text-[16px] sm:text-ui text-text placeholder:text-text-3 focus:outline-none focus:border-border-strong focus:ring-1 focus:ring-border-strong transition-colors"
								placeholder="Your name"
							/>
						</div>

						<!-- Username -->
						<div class="flex flex-col gap-1.5">
							<label for="username" class="text-label font-medium text-text-2">Username</label>
							<input
								id="username"
								name="username"
								type="text"
								value={username ? `@${username}` : ''}
								oninput={handleUsernameInput}
								class="w-full rounded-md border border-border bg-surface-2 px-3 py-2 font-mono text-[16px] sm:text-ui text-text placeholder:text-text-3 focus:outline-none focus:border-border-strong focus:ring-1 focus:ring-border-strong transition-colors"
								placeholder="@username"
							/>
							<div class="text-caption text-text-3">
								nomi.app/@{username || 'username'} ·
								{#if usernameChecking}
									<span class="text-warning">checking...</span>
								{:else if usernameAvailable === false}
									<span class="text-danger">taken or invalid</span>
								{:else}
									<span class="text-accent">available</span>
								{/if}
							</div>
						</div>

						<!-- Location -->
						<div class="flex flex-col gap-1.5">
							<label for="location" class="text-label font-medium text-text-2">Location</label>
							<input
								id="location"
								name="location"
								type="text"
								bind:value={location}
								class="w-full rounded-md border border-border bg-surface-2 px-3 py-2 text-[16px] sm:text-ui text-text placeholder:text-text-3 focus:outline-none focus:border-border-strong focus:ring-1 focus:ring-border-strong transition-colors"
								placeholder="City / Country"
							/>
						</div>

						<!-- Toggle 1: Show stats publicly -->
						<div class="flex items-start justify-between gap-3 pt-3 border-t border-border">
							<div class="flex flex-col">
								<span class="text-ui font-medium text-text">Show stats publicly</span>
								<span class="text-caption text-text-3 leading-tight">Tokens, streaks and top models. Never cost or memory.</span>
							</div>
							<Toggle label="Show stats publicly" bind:checked={showStatsPublicly} />
						</div>

						<!-- Toggle 2: Show crew on profile -->
						<div class="flex items-start justify-between gap-3 pt-3 border-t border-border">
							<div class="flex flex-col">
								<span class="text-ui font-medium text-text">Show crew on profile</span>
								<span class="text-caption text-text-3 leading-tight">Which agents you work with most.</span>
							</div>
							<Toggle label="Show crew on profile" bind:checked={showCrewPublicly} />
						</div>

						<!-- Unsaved changes bar -->
						{#if isDirty}
							<div class="flex items-center justify-between pt-3 border-t border-border">
								<span class="text-caption text-text-2">{unsavedCount} unsaved change{unsavedCount > 1 ? 's' : ''}</span>
								<div class="flex items-center gap-2">
									<Button variant="ghost" label="Discard" onclick={discardChanges} />
									<Button variant="inverse" label="Save" type="submit" />
								</div>
							</div>
						{/if}
					</form>
				</div>

				<!-- SHOWCASE Section -->
				<div class="rounded-xl border border-border bg-surface p-5 flex flex-col gap-4">
					<div class="flex items-center justify-between">
						<h2 class="text-label font-semibold tracking-wider text-text-3 uppercase">SHOWCASE</h2>
						<span class="text-caption text-text-3">{showcaseProjects.length} of 3</span>
					</div>

					<!-- Featured project list -->
					<div class="flex flex-col gap-2">
						{#if showcaseProjects.length === 0}
							<div class="text-caption text-text-3 py-3 text-center">
								No projects featured yet.
							</div>
						{:else}
							{#each showcaseProjects as project (project.id)}
								<div class="rounded-lg border border-border bg-surface-2 p-3 flex items-center justify-between gap-3 group">
									<!-- Mini preview + Details -->
									<div class="flex items-center gap-3 min-w-0">
										<div class="w-10 h-10 rounded-md bg-surface border border-border flex flex-col items-center justify-center text-nano text-text-2 p-1 text-center font-mono shrink-0">
											<span class="text-text-3">APP</span>
											<span class="font-bold truncate w-full text-center text-accent">Live</span>
										</div>
										<div class="flex flex-col min-w-0">
											<div class="flex items-center gap-1.5">
												<span class="text-ui font-semibold text-text truncate">{project.name}</span>
											</div>
											<div class="flex items-center gap-1.5 text-caption text-text-3 mt-0.5">
												<Status kind="live" label="Live" small />
												<span>·</span>
												<span>built with Koda · {formatDuration(project.build_duration_secs ?? 252)}</span>
											</div>
										</div>
									</div>

									<!-- Action: remove from showcase -->
									<IconButton
										icon={X}
										label="Remove from showcase"
										variant="ghost"
										onclick={() => removeShowcaseProject(project.id)}
									/>
								</div>
							{/each}
						{/if}

						<!-- Add project button -->
						{#if showcaseProjects.length < 3}
							<button
								type="button"
								onclick={() => (showAddProjectModal = true)}
								class="w-full py-2.5 rounded-lg border border-dashed border-border hover:border-border-strong text-ui font-medium text-text-2 hover:text-text transition-colors flex items-center justify-center gap-1.5 cursor-pointer"
							>
								<Icon icon={Plus} size={15} />
								<span>Add a live project</span>
							</button>
						{/if}
					</div>

					<p class="text-caption text-text-3 leading-relaxed">
						Only live projects can be featured. Visitors see the live URL, never the code.
					</p>
				</div>
			</div>
		</div>
	</div>
</div>

<!-- Modal: Select Project to Showcase -->
<Dialog bind:open={showAddProjectModal} headline="Feature a live project">
	<div class="flex flex-col gap-2 max-h-60 overflow-y-auto">
		{#if availableProjects.length === 0}
			<p class="text-caption text-text-3 py-4 text-center">
				No other projects found. Build or deploy a project first!
			</p>
		{:else}
			{#each availableProjects as project}
				<button
					type="button"
					class="w-full text-left p-3 rounded-lg border border-border hover:border-border-strong hover:bg-surface-2 transition-colors flex items-center justify-between cursor-pointer"
					onclick={() => addShowcaseProject(project)}
				>
					<div class="flex flex-col">
						<span class="text-ui font-medium text-text">{project.name}</span>
						<span class="text-caption text-text-3">{project.description || 'No description'}</span>
					</div>
					<span class="text-ui text-accent font-semibold">+ Add</span>
				</button>
			{/each}
		{/if}
	</div>
	{#snippet actions()}
		<Button label="Cancel" variant="ghost" onclick={() => (showAddProjectModal = false)} />
	{/snippet}
</Dialog>
