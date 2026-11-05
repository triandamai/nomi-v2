<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { enhance } from '$app/forms';
	import { invalidateAll } from '$app/navigation';
	import Mascot from '$lib/components/ds/agents/Mascot.svelte';
	import type { CrewId } from '$lib/components/ds/agents/crew';
	import TokenActivityHeatmap from '$lib/components/profile/TokenActivityHeatmap.svelte';
	import type { ActionData, PageData } from './$types';
	import type { ShowcaseProject, ProjectSummary } from '$lib/types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	// Form states
	let displayName = $state('');
	let username = $state('');
	let avatarUrl = $state('');
	let location = $state('');
	let showStatsPublicly = $state(true);
	let showCrewPublicly = $state(false);
	let showcaseProjects = $state<ShowcaseProject[]>([]);

	// Initial values for dirty tracking
	let initialDisplayName = $state('');
	let initialUsername = $state('');
	let initialAvatarUrl = $state('');
	let initialLocation = $state('');
	let initialShowStats = $state(true);
	let initialShowCrew = $state(false);
	let initialShowcaseIds = $state<string[]>([]);

	let showAddProjectModal = $state(false);
	let copiedLink = $state(false);
	let usernameAvailable = $state<boolean | null>(null);
	let usernameChecking = $state(false);
	let usernameDebounceTimer: ReturnType<typeof setTimeout> | null = null;
	let showAllCrew = $state(false);

	$effect(() => {
		displayName = data.profile?.display_name ?? '';
		username = data.profile?.username ?? '';
		avatarUrl = data.profile?.avatar_url ?? '';
		location = data.profile?.location ?? 'Jakarta';
		showStatsPublicly = data.profile?.show_stats_publicly ?? true;
		showCrewPublicly = data.profile?.show_crew_publicly ?? false;
		showcaseProjects = data.profile?.showcase_projects ?? [];

		initialDisplayName = displayName;
		initialUsername = username;
		initialAvatarUrl = avatarUrl;
		initialLocation = location;
		initialShowStats = showStatsPublicly;
		initialShowCrew = showCrewPublicly;
		initialShowcaseIds = showcaseProjects.map((p) => p.id);
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

<div class="min-h-[100dvh] bg-[#0c0d0f] text-[#ededed] px-4 py-5 sm:px-8 sm:py-7">
	<div class="max-w-6xl mx-auto flex flex-col gap-6">
		<!-- Top Navigation & Action Bar -->
		<div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
			<div class="flex items-center gap-2 text-sm text-[#9aa0a6]">
				<a href="/settings" class="hover:text-white transition-colors">Settings</a>
				<span class="text-xs text-[#5f6368]">›</span>
				<span class="text-white font-medium">Profile</span>
			</div>

			<div class="flex items-center gap-2">
				<a
					href="/@{username || 'me'}"
					target="_blank"
					rel="noreferrer"
					class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-[#2a2d34] bg-[#14161a] hover:bg-[#1e2127] text-xs font-medium text-[#d1d5db] transition-colors"
				>
					<svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z" />
						<circle cx="12" cy="12" r="3" />
					</svg>
					Preview public profile
				</a>
				<button
					type="button"
					onclick={copyProfileLink}
					class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-[#2a2d34] bg-[#14161a] hover:bg-[#1e2127] text-xs font-medium text-[#d1d5db] transition-colors"
				>
					<svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" />
						<path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" />
					</svg>
					{copiedLink ? 'Copied link!' : 'Copy link'}
				</button>
			</div>
		</div>

		<!-- Error banners if any -->
		{#if form?.error}
			<div class="rounded-lg bg-red-950/40 border border-red-800/60 p-3 text-xs text-red-200">
				{form.error}
			</div>
		{/if}
		{#if uploadError}
			<div class="rounded-lg bg-red-950/40 border border-red-800/60 p-3 text-xs text-red-200">
				{uploadError}
			</div>
		{/if}

		<!-- Main Layout Grid: Left (2/3) + Right (1/3) -->
		<div class="grid grid-cols-1 lg:grid-cols-3 gap-6 items-start">
			<!-- LEFT COLUMN: Profile info, stats, heatmap, models, crew, insights -->
			<div class="lg:col-span-2 flex flex-col gap-6">
				<!-- Profile Header -->
				<div class="flex items-center gap-4">
					{#if avatarUrl}
						<img
							src={avatarUrl}
							alt={displayName || 'Avatar'}
							class="w-16 h-16 rounded-2xl object-cover border border-[#2c3038] shadow-md flex-shrink-0"
						/>
					{:else}
						<div class="w-16 h-16 rounded-2xl bg-[#1b1e24] border border-[#2c3038] flex items-center justify-center text-lg font-bold text-white shadow-md flex-shrink-0">
							{initials}
						</div>
					{/if}
					<div class="flex flex-col min-w-0">
						<h1 class="text-2xl sm:text-3xl font-bold text-white tracking-tight truncate">
							{displayName || 'Mario Prasetyo'}
						</h1>
						<div class="flex flex-wrap items-center gap-1.5 text-xs text-[#9aa0a6] mt-1">
							<span>@{username || 'mario'}</span>
							<span class="text-[#5f6368]">·</span>
							<span>{location || 'Jakarta'}</span>
							<span class="text-[#5f6368]">·</span>
							<span>joined {joinedFormatted}</span>
							<span class="text-[#5f6368]">·</span>
							<span>{data.profile?.plan_name || 'Free plan'}</span>
						</div>
					</div>
				</div>

				<!-- 5 Metric Stat Cards -->
				<div class="grid grid-cols-2 sm:grid-cols-5 gap-3">
					<!-- 1. Lifetime tokens -->
					<div class="rounded-xl border border-[#22252a] bg-[#131518] p-3.5 sm:p-4 flex flex-col justify-between">
						<div>
							<div class="text-xl sm:text-2xl font-bold text-white tracking-tight">{formatTokens(lifetimeTokens)}</div>
							<div class="text-xs text-[#9aa0a6] mt-1 font-medium">Lifetime tokens</div>
						</div>
						<div class="text-[11px] text-[#5f6368] mt-2 truncate">since {joinedFormatted.slice(0, 6)}</div>
					</div>

					<!-- 2. Peak day -->
					<div class="rounded-xl border border-[#22252a] bg-[#131518] p-3.5 sm:p-4 flex flex-col justify-between">
						<div>
							<div class="text-xl sm:text-2xl font-bold text-white tracking-tight">{formatTokens(peakDayTokens)}</div>
							<div class="text-xs text-[#9aa0a6] mt-1 font-medium">Peak day</div>
						</div>
						<div class="text-[11px] text-[#5f6368] mt-2 truncate">{peakDayDate}</div>
					</div>

					<!-- 3. Longest task -->
					<div class="rounded-xl border border-[#22252a] bg-[#131518] p-3.5 sm:p-4 flex flex-col justify-between">
						<div>
							<div class="text-xl sm:text-2xl font-bold text-white tracking-tight">{formatDuration(longestTaskSecs)}</div>
							<div class="text-xs text-[#9aa0a6] mt-1 font-medium">Longest task</div>
						</div>
						<div class="text-[11px] text-[#5f6368] mt-2 truncate">Koda · {longestTaskName}</div>
					</div>

					<!-- 4. Longest streak -->
					<div class="rounded-xl border border-[#22252a] bg-[#131518] p-3.5 sm:p-4 flex flex-col justify-between">
						<div>
							<div class="text-xl sm:text-2xl font-bold text-white tracking-tight">{longestStreakDays} days</div>
							<div class="text-xs text-[#9aa0a6] mt-1 font-medium">Longest streak</div>
						</div>
						<div class="text-[11px] text-[#5f6368] mt-2 truncate">{longestStreakRange}</div>
					</div>

					<!-- 5. Current streak -->
					<div class="rounded-xl border border-[#22252a] bg-[#131518] p-3.5 sm:p-4 flex flex-col justify-between col-span-2 sm:col-span-1">
						<div>
							<div class="text-xl sm:text-2xl font-bold text-white tracking-tight flex items-center gap-1">
								{currentStreakDays} days <span class="text-base">🔥</span>
							</div>
							<div class="text-xs text-[#9aa0a6] mt-1 font-medium">Current streak</div>
						</div>
						<div class="text-[11px] text-[#5f6368] mt-2 truncate">
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
					<div class="rounded-xl border border-[#22252a] bg-[#131518] p-4 flex flex-col justify-between gap-3">
						<div class="flex items-center justify-between">
							<h3 class="text-xs font-semibold text-white tracking-tight">Models you use most</h3>
							<span class="text-[11px] text-[#70757a]">last 30 days</span>
						</div>
						<div class="flex flex-col gap-2.5">
							{#each topModels as model}
								<div class="flex flex-col gap-1">
									<div class="flex items-center justify-between text-xs">
										<span class="text-[#d1d5db] font-mono text-[11px] truncate max-w-[120px]">{model.label}</span>
										<span class="text-[#9aa0a6] text-[11px]">{formatTokens(model.tokens)} · {model.percentage}%</span>
									</div>
									<div class="w-full h-1.5 rounded-full bg-[#1e2126] overflow-hidden">
										<div class="h-full rounded-full bg-[#4a505b]" style="width: {Math.min(100, Math.max(4, model.percentage))}%;"></div>
									</div>
								</div>
							{/each}
						</div>
					</div>

					<!-- Your crew -->
					<div class="rounded-xl border border-[#22252a] bg-[#131518] p-4 flex flex-col justify-between gap-3">
						<div class="flex items-center justify-between">
							<h3 class="text-xs font-semibold text-white tracking-tight">Your crew</h3>
							<span class="text-[11px] text-[#70757a]">chats</span>
						</div>
						<div class="flex flex-col gap-2.5">
							{#each crewUsage.slice(0, showAllCrew ? undefined : 4) as member}
								<div class="flex items-center gap-2">
									<Mascot agent={member.agent_id as CrewId} size={20} />
									<span class="text-xs text-white font-medium w-12 truncate">{member.name}</span>
									<div class="flex-1 h-1.5 rounded-full bg-[#1e2126] overflow-hidden">
										<div
											class="h-full rounded-full {member.agent_id === 'nomi'
												? 'bg-emerald-500'
												: member.agent_id === 'dana'
												? 'bg-orange-400'
												: member.agent_id === 'kala'
												? 'bg-amber-400'
												: 'bg-lime-500'}"
											style="width: {Math.min(100, Math.max(6, (member.count / maxCrewCount) * 100))}%;"
										></div>
									</div>
									<span class="text-[11px] text-[#9aa0a6] text-right min-w-[50px]">
										{member.count} {member.kind === 'projects' ? 'projects' : ''}
									</span>
								</div>
							{/each}
						</div>
						{#if crewUsage.length > 4 || !showAllCrew}
							<button
								type="button"
								class="text-left text-[11px] text-[#70757a] hover:text-[#9aa0a6] transition-colors pt-1"
								onclick={() => (showAllCrew = !showAllCrew)}
							>
								{showAllCrew ? 'Show less' : '+6 more agents'}
							</button>
						{/if}
					</div>

					<!-- Insights -->
					<div class="rounded-xl border border-[#22252a] bg-[#131518] p-4 flex flex-col justify-between gap-3">
						<div>
							<h3 class="text-xs font-semibold text-white tracking-tight">Insights</h3>
						</div>
						<div class="flex flex-col gap-4 text-xs">
							<!-- Insight 1 -->
							<div class="flex flex-col gap-1">
								<div class="flex items-center gap-1.5 font-semibold text-white text-xs">
									<svg class="w-3.5 h-3.5 text-amber-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
										<path d="M12 2v1" />
										<path d="M12 21v1" />
										<path d="M4.93 4.93l.7.7" />
										<path d="M18.36 18.36l.7.7" />
										<path d="M2 12h1" />
										<path d="M21 12h1" />
										<path d="M4.93 19.07l.7-.7" />
										<path d="M18.36 5.64l.7-.7" />
										<circle cx="12" cy="12" r="5" />
									</svg>
									Save ~18% on Nomi
								</div>
								<p class="text-[11px] text-[#9aa0a6] leading-relaxed">
									gemini-flash answered 9 of your last 10 quick questions just as well.
								</p>
								<a href="/models" class="text-[11px] text-[#d1d5db] hover:text-white font-medium inline-flex items-center gap-1 mt-0.5">
									Switch default →
								</a>
							</div>

							<!-- Insight 2 -->
							<div class="flex flex-col gap-1 pt-2 border-t border-[#1e2126]">
								<div class="flex items-center gap-1.5 font-semibold text-white text-xs">
									<svg class="w-3.5 h-3.5 text-blue-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
										<path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" />
									</svg>
									You work late
								</div>
								<p class="text-[11px] text-[#9aa0a6] leading-relaxed">
									68% of your tokens land after 21:00 WIB. Kala can hold reminders until 08:00.
								</p>
								<a href="/reminders" class="text-[11px] text-[#d1d5db] hover:text-white font-medium inline-flex items-center gap-1 mt-0.5">
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
				<div class="rounded-xl border border-[#22252a] bg-[#131518] p-5 flex flex-col gap-4">
					<h2 class="text-xs font-semibold tracking-wider text-[#70757a] uppercase">PUBLIC PROFILE</h2>

					<!-- Photo row -->
					<div class="flex items-center gap-3">
						{#if avatarUrl}
							<img src={avatarUrl} alt="Avatar" class="w-11 h-11 rounded-xl object-cover border border-[#2c3038]" />
						{:else}
							<div class="w-11 h-11 rounded-xl bg-[#1b1e24] border border-[#2c3038] flex items-center justify-center text-xs font-bold text-white">
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
							<button
								type="button"
								onclick={() => fileInput?.click()}
								disabled={uploading}
								class="px-2.5 py-1.5 rounded-lg border border-[#2e323b] bg-[#1a1d22] hover:bg-[#252830] text-xs font-medium text-white transition-colors disabled:opacity-50"
							>
								{uploading ? 'Uploading...' : 'Change photo'}
							</button>
							{#if avatarUrl}
								<button
									type="button"
									onclick={removeAvatar}
									class="px-2.5 py-1.5 text-xs text-[#9aa0a6] hover:text-white transition-colors"
								>
									Remove
								</button>
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
							<label for="display_name" class="text-xs font-medium text-[#d1d5db]">Display name</label>
							<input
								id="display_name"
								name="display_name"
								type="text"
								bind:value={displayName}
								class="w-full rounded-lg border border-[#26292f] bg-[#0e1013] px-3 py-2 text-[16px] sm:text-xs text-white placeholder-[#50545c] focus:outline-none focus:border-emerald-500 focus:ring-1 focus:ring-emerald-500 transition-colors"
								placeholder="Your name"
							/>
						</div>

						<!-- Username -->
						<div class="flex flex-col gap-1.5">
							<label for="username" class="text-xs font-medium text-[#d1d5db]">Username</label>
							<div class="relative">
								<input
									id="username"
									name="username"
									type="text"
									value={username ? `@${username}` : ''}
									oninput={handleUsernameInput}
									class="w-full rounded-lg border border-[#26292f] bg-[#0e1013] px-3 py-2 text-[16px] sm:text-xs text-white placeholder-[#50545c] focus:outline-none focus:border-emerald-500 focus:ring-1 focus:ring-emerald-500 transition-colors"
									placeholder="@username"
								/>
							</div>
							<div class="text-[11px] text-[#70757a]">
								nomi.app/@{username || 'username'} ·
								{#if usernameChecking}
									<span class="text-amber-400">checking...</span>
								{:else if usernameAvailable === false}
									<span class="text-red-400">taken or invalid</span>
								{:else}
									<span class="text-emerald-400">available</span>
								{/if}
							</div>
						</div>

						<!-- Location -->
						<div class="flex flex-col gap-1.5">
							<label for="location" class="text-xs font-medium text-[#d1d5db]">Location</label>
							<input
								id="location"
								name="location"
								type="text"
								bind:value={location}
								class="w-full rounded-lg border border-[#26292f] bg-[#0e1013] px-3 py-2 text-[16px] sm:text-xs text-white placeholder-[#50545c] focus:outline-none focus:border-emerald-500 focus:ring-1 focus:ring-emerald-500 transition-colors"
								placeholder="City / Country"
							/>
						</div>

						<!-- Toggle 1: Show stats publicly -->
						<div class="flex items-start justify-between gap-3 pt-2 border-t border-[#1e2126]">
							<div class="flex flex-col">
								<span class="text-xs font-medium text-white">Show stats publicly</span>
								<span class="text-[11px] text-[#70757a] leading-tight">Tokens, streaks and top models. Never cost or memory.</span>
							</div>
							<button
								type="button"
								role="switch"
								aria-label="Show stats publicly"
								aria-checked={showStatsPublicly}
								class="w-10 h-5 rounded-full transition-colors relative flex-shrink-0 mt-0.5 {showStatsPublicly ? 'bg-emerald-500' : 'bg-[#2a2e36]'}"
								onclick={() => (showStatsPublicly = !showStatsPublicly)}
							>
								<span
									class="absolute top-0.5 w-4 h-4 rounded-full bg-white transition-transform {showStatsPublicly ? 'right-0.5' : 'left-0.5'}"
								></span>
							</button>
						</div>

						<!-- Toggle 2: Show crew on profile -->
						<div class="flex items-start justify-between gap-3 pt-2 border-t border-[#1e2126]">
							<div class="flex flex-col">
								<span class="text-xs font-medium text-white">Show crew on profile</span>
								<span class="text-[11px] text-[#70757a] leading-tight">Which agents you work with most.</span>
							</div>
							<button
								type="button"
								role="switch"
								aria-label="Show crew on profile"
								aria-checked={showCrewPublicly}
								class="w-10 h-5 rounded-full transition-colors relative flex-shrink-0 mt-0.5 {showCrewPublicly ? 'bg-emerald-500' : 'bg-[#2a2e36]'}"
								onclick={() => (showCrewPublicly = !showCrewPublicly)}
							>
								<span
									class="absolute top-0.5 w-4 h-4 rounded-full bg-white transition-transform {showCrewPublicly ? 'right-0.5' : 'left-0.5'}"
								></span>
							</button>
						</div>

						<!-- Unsaved changes bar -->
						{#if isDirty}
							<div class="flex items-center justify-between pt-3 border-t border-[#1e2126]">
								<span class="text-xs text-[#9aa0a6]">{unsavedCount} unsaved change{unsavedCount > 1 ? 's' : ''}</span>
								<div class="flex items-center gap-2">
									<button
										type="button"
										onclick={discardChanges}
										class="px-2.5 py-1 text-xs text-[#9aa0a6] hover:text-white transition-colors"
									>
										Discard
									</button>
									<button
										type="submit"
										class="px-3.5 py-1.5 rounded-lg bg-white hover:bg-neutral-200 text-black text-xs font-semibold shadow-sm transition-colors"
									>
										Save
									</button>
								</div>
							</div>
						{/if}
					</form>
				</div>

				<!-- SHOWCASE Section -->
				<div class="rounded-xl border border-[#22252a] bg-[#131518] p-5 flex flex-col gap-4">
					<div class="flex items-center justify-between">
						<h2 class="text-xs font-semibold tracking-wider text-[#70757a] uppercase">SHOWCASE</h2>
						<span class="text-xs text-[#70757a]">{showcaseProjects.length} of 3</span>
					</div>

					<!-- Featured project list -->
					<div class="flex flex-col gap-2">
						{#if showcaseProjects.length === 0}
							<div class="text-xs text-[#70757a] py-3 text-center">
								No projects featured yet.
							</div>
						{:else}
							{#each showcaseProjects as project (project.id)}
								<div class="rounded-lg border border-[#22252a] bg-[#171a1f] p-3 flex items-center justify-between gap-3 group">
									<!-- Mini preview + Details -->
									<div class="flex items-center gap-3 min-w-0">
										<div class="w-10 h-10 rounded-md bg-[#0e1013] border border-[#282c34] flex flex-col items-center justify-center text-[10px] text-white p-1 text-center font-mono">
											<span class="text-[8px] text-[#70757a]">APP</span>
											<span class="font-bold truncate w-full text-center">Live</span>
										</div>
										<div class="flex flex-col min-w-0">
											<div class="flex items-center gap-1.5">
												<span class="text-xs font-semibold text-white truncate">{project.name}</span>
											</div>
											<div class="flex items-center gap-1.5 text-[11px] text-[#70757a] mt-0.5">
												<span class="inline-flex items-center gap-1 text-emerald-400 font-medium">
													<span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
													Live
												</span>
												<span>·</span>
												<span>built with Koda · {formatDuration(project.build_duration_secs ?? 252)}</span>
											</div>
										</div>
									</div>

									<!-- Action: remove from showcase -->
									<button
										type="button"
										onclick={() => removeShowcaseProject(project.id)}
										aria-label="Remove from showcase"
										class="text-[#70757a] hover:text-white p-1 opacity-60 hover:opacity-100 transition-opacity"
									>
										<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
											<path d="M18 6 6 18" />
											<path d="m6 6 12 12" />
										</svg>
									</button>
								</div>
							{/each}
						{/if}

						<!-- Add project button -->
						{#if showcaseProjects.length < 3}
							<button
								type="button"
								onclick={() => (showAddProjectModal = true)}
								class="w-full py-2.5 rounded-lg border border-dashed border-[#2b2f38] hover:border-[#3f4552] text-xs font-medium text-[#9aa0a6] hover:text-white transition-colors flex items-center justify-center gap-1.5"
							>
								<span>+</span> Add a live project
							</button>
						{/if}
					</div>

					<p class="text-[11px] text-[#70757a] leading-relaxed">
						Only live projects can be featured. Visitors see the live URL, never the code.
					</p>
				</div>
			</div>
		</div>
	</div>
</div>

<!-- Modal: Select Project to Showcase -->
{#if showAddProjectModal}
	<div
		class="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm flex items-center justify-center p-4"
		role="presentation"
		tabindex="-1"
		onkeydown={(e) => {
			if (e.key === 'Escape') showAddProjectModal = false;
		}}
	>
		<button
			type="button"
			class="absolute inset-0 w-full h-full cursor-default"
			onclick={() => (showAddProjectModal = false)}
			aria-label="Close modal overlay"
		></button>
		<div class="relative w-full max-w-md rounded-xl border border-[#2b2f38] bg-[#14161a] p-5 shadow-2xl flex flex-col gap-4 z-10" role="dialog" aria-modal="true" tabindex="-1">
			<div class="flex items-center justify-between">
				<h3 class="text-sm font-semibold text-white">Feature a live project</h3>
				<button
					type="button"
					onclick={() => (showAddProjectModal = false)}
					class="text-[#70757a] hover:text-white p-1"
				>
					✕
				</button>
			</div>

			<div class="flex flex-col gap-2 max-h-60 overflow-y-auto">
				{#if availableProjects.length === 0}
					<p class="text-xs text-[#70757a] py-4 text-center">
						No other projects found. Build or deploy a project first!
					</p>
				{:else}
					{#each availableProjects as project}
						<button
							type="button"
							class="w-full text-left p-3 rounded-lg border border-[#22252a] hover:border-emerald-500/50 hover:bg-[#1a1d22] transition-colors flex items-center justify-between"
							onclick={() => addShowcaseProject(project)}
						>
							<div class="flex flex-col">
								<span class="text-xs font-medium text-white">{project.name}</span>
								<span class="text-[11px] text-[#70757a]">{project.description || 'No description'}</span>
							</div>
							<span class="text-xs text-emerald-400 font-semibold">+ Add</span>
						</button>
					{/each}
				{/if}
			</div>

			<div class="flex justify-end pt-2 border-t border-[#22252a]">
				<button
					type="button"
					onclick={() => (showAddProjectModal = false)}
					class="px-3 py-1.5 rounded-lg text-xs text-[#9aa0a6] hover:text-white"
				>
					Cancel
				</button>
			</div>
		</div>
	</div>
{/if}
