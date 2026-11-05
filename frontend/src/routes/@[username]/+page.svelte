<script lang="ts">
	import Mascot from '$lib/components/ds/agents/Mascot.svelte';
	import type { CrewId } from '$lib/components/ds/agents/crew';
	import TokenActivityHeatmap from '$lib/components/profile/TokenActivityHeatmap.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
	const profile = $derived(data.profile);

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

	const joinedFormatted = $derived(formatJoinedDate(profile.joined_at));

	const initials = $derived.by(() => {
		const name = profile.display_name || profile.username || 'User';
		return name
			.trim()
			.split(/\s+/)
			.slice(0, 2)
			.map((p) => p[0]?.toUpperCase() ?? '')
			.join('') || 'MP';
	});

	let copiedLink = $state(false);
	async function copyLink() {
		try {
			await navigator.clipboard.writeText(window.location.href);
			copiedLink = true;
			setTimeout(() => {
				copiedLink = false;
			}, 2000);
		} catch {
			// fallback
		}
	}

	const maxCrewCount = $derived(Math.max(1, ...(profile.crew_usage ?? []).map((c) => c.count)));
</script>

<svelte:head>
	<title>{profile.display_name || `@${profile.username}`} on Nomi</title>
	<meta name="description" content="Check out {profile.display_name || `@${profile.username}`}'s profile, stats, and live apps on Nomi." />
</svelte:head>

<div class="min-h-[100dvh] bg-[#0c0d0f] text-[#ededed] flex flex-col justify-between">
	<!-- Public Top Navigation Bar -->
	<header class="border-b border-[#1b1e24] bg-[#0f1114]/80 backdrop-blur-md sticky top-0 z-40 px-4 py-3 sm:px-8">
		<div class="max-w-5xl mx-auto flex items-center justify-between">
			<a href="/" class="flex items-center gap-2 text-white font-bold tracking-tight text-base hover:opacity-90 transition-opacity">
				<span class="w-6 h-6 rounded-lg bg-emerald-500 flex items-center justify-center text-black font-mono font-black text-xs">n</span>
				<span>nomi</span>
			</a>

			<div class="flex items-center gap-3">
				<button
					type="button"
					onclick={copyLink}
					class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-[#2a2d34] bg-[#14161a] hover:bg-[#1e2127] text-xs font-medium text-[#d1d5db] transition-colors"
				>
					<svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
						<path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" />
						<path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" />
					</svg>
					{copiedLink ? 'Copied profile link' : 'Share'}
				</button>
				<a
					href="/register"
					class="px-3.5 py-1.5 rounded-lg bg-white hover:bg-neutral-200 text-black text-xs font-semibold shadow-sm transition-colors"
				>
					Start building
				</a>
			</div>
		</div>
	</header>

	<!-- Main Profile Content -->
	<main class="flex-1 max-w-5xl w-full mx-auto px-4 py-8 sm:px-8 flex flex-col gap-8">
		<!-- Profile Card Header -->
		<div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 p-6 rounded-2xl border border-[#22252a] bg-[#131518]">
			<div class="flex items-center gap-4">
				{#if profile.avatar_url}
					<img
						src={profile.avatar_url}
						alt={profile.display_name || profile.username}
						class="w-16 h-16 sm:w-20 sm:h-20 rounded-2xl object-cover border border-[#2c3038] shadow-md flex-shrink-0"
					/>
				{:else}
					<div class="w-16 h-16 sm:w-20 sm:h-20 rounded-2xl bg-[#1b1e24] border border-[#2c3038] flex items-center justify-center text-xl sm:text-2xl font-bold text-white shadow-md flex-shrink-0">
						{initials}
					</div>
				{/if}

				<div class="flex flex-col min-w-0">
					<h1 class="text-2xl sm:text-3xl font-bold text-white tracking-tight truncate">
						{profile.display_name || profile.username}
					</h1>
					<div class="flex flex-wrap items-center gap-1.5 text-xs text-[#9aa0a6] mt-1.5">
						<span class="text-white font-medium">@{profile.username}</span>
						{#if profile.location}
							<span class="text-[#5f6368]">·</span>
							<span>{profile.location}</span>
						{/if}
						<span class="text-[#5f6368]">·</span>
						<span>joined {joinedFormatted}</span>
						<span class="text-[#5f6368]">·</span>
						<span class="px-2 py-0.5 rounded-md bg-[#1d2026] text-[#c9cdd4] text-[11px] font-medium border border-[#2d313a]">
							{profile.plan_name}
						</span>
					</div>
				</div>
			</div>

			<div class="flex items-center gap-2 self-start sm:self-auto">
				<a
					href="/register"
					class="px-4 py-2 rounded-xl bg-emerald-500 hover:bg-emerald-400 text-black font-semibold text-xs transition-colors flex items-center gap-1.5 shadow-sm"
				>
					<span>Build with Nomi</span>
					<span>→</span>
				</a>
			</div>
		</div>

		<!-- Showcase Live Projects (Always visible if any) -->
		{#if profile.showcase_projects && profile.showcase_projects.length > 0}
			<section class="flex flex-col gap-3">
				<div class="flex items-center justify-between">
					<h2 class="text-sm font-semibold tracking-wide text-white uppercase">Featured Projects</h2>
					<span class="text-xs text-[#70757a]">{profile.showcase_projects.length} live app{profile.showcase_projects.length > 1 ? 's' : ''}</span>
				</div>

				<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
					{#each profile.showcase_projects as project}
						<div class="rounded-xl border border-[#22252a] bg-[#131518] p-4 flex flex-col justify-between gap-4 hover:border-[#333742] transition-colors group">
							<div class="flex items-start justify-between gap-2">
								<div class="flex items-center gap-3">
									<div class="w-10 h-10 rounded-lg bg-[#1a1d22] border border-[#282c34] flex flex-col items-center justify-center font-mono text-[9px] text-emerald-400">
										<span class="w-1.5 h-1.5 rounded-full bg-emerald-400 mb-0.5"></span>
										LIVE
									</div>
									<div class="flex flex-col min-w-0">
										<h3 class="text-sm font-semibold text-white group-hover:text-emerald-400 transition-colors truncate">
											{project.name}
										</h3>
										<span class="text-[11px] text-[#70757a]">
											built with Koda · {formatDuration(project.build_duration_secs ?? 252)}
										</span>
									</div>
								</div>
							</div>

							{#if project.description}
								<p class="text-xs text-[#9aa0a6] line-clamp-2 leading-relaxed">
									{project.description}
								</p>
							{/if}

							<div class="pt-2 border-t border-[#1e2126] flex items-center justify-between">
								<span class="text-[11px] text-[#70757a]">Production</span>
								{#if project.deploy_url}
									<a
										href={project.deploy_url}
										target="_blank"
										rel="noreferrer"
										class="inline-flex items-center gap-1 text-xs text-emerald-400 hover:text-emerald-300 font-medium"
									>
										<span>Visit live app</span>
										<span>↗</span>
									</a>
								{:else}
									<span class="text-xs text-[#70757a]">Live app</span>
								{/if}
							</div>
						</div>
					{/each}
				</div>
			</section>
		{/if}

		<!-- Public Stats & Activity Section (If enabled) -->
		{#if profile.show_stats_publicly && profile.stats}
			<section class="flex flex-col gap-6">
				<!-- 5 Stat Cards -->
				<div class="grid grid-cols-2 sm:grid-cols-5 gap-3">
					<div class="rounded-xl border border-[#22252a] bg-[#131518] p-3.5 sm:p-4 flex flex-col justify-between">
						<div>
							<div class="text-xl sm:text-2xl font-bold text-white tracking-tight">{formatTokens(profile.stats.lifetime_tokens)}</div>
							<div class="text-xs text-[#9aa0a6] mt-1 font-medium">Lifetime tokens</div>
						</div>
						<div class="text-[11px] text-[#5f6368] mt-2 truncate">since {joinedFormatted.slice(0, 6)}</div>
					</div>

					<div class="rounded-xl border border-[#22252a] bg-[#131518] p-3.5 sm:p-4 flex flex-col justify-between">
						<div>
							<div class="text-xl sm:text-2xl font-bold text-white tracking-tight">{formatTokens(profile.stats.peak_day_tokens)}</div>
							<div class="text-xs text-[#9aa0a6] mt-1 font-medium">Peak day</div>
						</div>
						<div class="text-[11px] text-[#5f6368] mt-2 truncate">{profile.stats.peak_day_date || 'Peak'}</div>
					</div>

					<div class="rounded-xl border border-[#22252a] bg-[#131518] p-3.5 sm:p-4 flex flex-col justify-between">
						<div>
							<div class="text-xl sm:text-2xl font-bold text-white tracking-tight">{formatDuration(profile.stats.longest_task_secs ?? 252)}</div>
							<div class="text-xs text-[#9aa0a6] mt-1 font-medium">Longest task</div>
						</div>
						<div class="text-[11px] text-[#5f6368] mt-2 truncate">Koda · {profile.stats.longest_task_name || 'build'}</div>
					</div>

					<div class="rounded-xl border border-[#22252a] bg-[#131518] p-3.5 sm:p-4 flex flex-col justify-between">
						<div>
							<div class="text-xl sm:text-2xl font-bold text-white tracking-tight">{profile.stats.longest_streak_days} days</div>
							<div class="text-xs text-[#9aa0a6] mt-1 font-medium">Longest streak</div>
						</div>
						<div class="text-[11px] text-[#5f6368] mt-2 truncate">{profile.stats.longest_streak_range || 'Personal best'}</div>
					</div>

					<div class="rounded-xl border border-[#22252a] bg-[#131518] p-3.5 sm:p-4 flex flex-col justify-between col-span-2 sm:col-span-1">
						<div>
							<div class="text-xl sm:text-2xl font-bold text-white tracking-tight flex items-center gap-1">
								{profile.stats.current_streak_days} days <span class="text-base">🔥</span>
							</div>
							<div class="text-xs text-[#9aa0a6] mt-1 font-medium">Current streak</div>
						</div>
						<div class="text-[11px] text-[#5f6368] mt-2 truncate">
							best is {profile.stats.best_streak_target}
						</div>
					</div>
				</div>

				<!-- Activity Heatmap -->
				<TokenActivityHeatmap
					activity={profile.activity ?? []}
					joinedDate={joinedFormatted}
					totalTokens={profile.stats.lifetime_tokens}
				/>

				<!-- Top Models Breakdown -->
				{#if profile.top_models && profile.top_models.length > 0}
					<div class="rounded-xl border border-[#22252a] bg-[#131518] p-5 flex flex-col gap-3">
						<div class="flex items-center justify-between">
							<h3 class="text-xs font-semibold text-white tracking-tight">Models used most</h3>
							<span class="text-[11px] text-[#70757a]">last 30 days</span>
						</div>
						<div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-4">
							{#each profile.top_models as model}
								<div class="flex flex-col gap-1.5 p-3 rounded-lg bg-[#181b20] border border-[#23262d]">
									<div class="flex items-center justify-between text-xs">
										<span class="text-[#d1d5db] font-mono text-xs truncate">{model.label}</span>
										<span class="text-emerald-400 font-semibold">{model.percentage}%</span>
									</div>
									<div class="w-full h-1.5 rounded-full bg-[#202328] overflow-hidden">
										<div class="h-full rounded-full bg-emerald-500" style="width: {model.percentage}%;"></div>
									</div>
									<span class="text-[11px] text-[#70757a]">{formatTokens(model.tokens)} tokens</span>
								</div>
							{/each}
						</div>
					</div>
				{/if}
			</section>
		{/if}

		<!-- Crew Section (If enabled) -->
		{#if profile.show_crew_publicly && profile.crew_usage && profile.crew_usage.length > 0}
			<section class="rounded-xl border border-[#22252a] bg-[#131518] p-5 flex flex-col gap-4">
				<div class="flex items-center justify-between">
					<h3 class="text-xs font-semibold text-white tracking-tight">Agent Crew</h3>
					<span class="text-[11px] text-[#70757a]">Collaborators</span>
				</div>
				<div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-3">
					{#each profile.crew_usage as member}
						<div class="flex items-center gap-3 p-3 rounded-lg bg-[#181b20] border border-[#23262d]">
							<Mascot agent={member.agent_id as CrewId} size={32} />
							<div class="flex flex-col min-w-0 flex-1">
								<span class="text-xs font-bold text-white truncate">{member.name}</span>
								<div class="w-full h-1.5 rounded-full bg-[#242831] overflow-hidden mt-1">
									<div
										class="h-full rounded-full {member.agent_id === 'nomi'
											? 'bg-emerald-500'
											: member.agent_id === 'dana'
											? 'bg-orange-400'
											: member.agent_id === 'kala'
											? 'bg-amber-400'
											: 'bg-lime-500'}"
										style="width: {Math.min(100, Math.max(8, (member.count / maxCrewCount) * 100))}%;"
									></div>
								</div>
								<span class="text-[10px] text-[#70757a] mt-1">
									{member.count} {member.kind === 'projects' ? 'projects' : 'chats'}
								</span>
							</div>
						</div>
					{/each}
				</div>
			</section>
		{/if}
	</main>

	<!-- Public Footer -->
	<footer class="border-t border-[#1b1e24] bg-[#0c0d0f] py-8 px-4 text-center text-xs text-[#70757a]">
		<div class="max-w-5xl mx-auto flex flex-col sm:flex-row items-center justify-between gap-4">
			<div class="flex items-center gap-2">
				<span class="w-4 h-4 rounded-md bg-emerald-500/20 text-emerald-400 font-mono font-bold text-[10px] flex items-center justify-center">n</span>
				<span class="text-[#9aa0a6]">Built with Nomi</span>
			</div>
			<div>A small crew of agents that remembers, builds, and works for you.</div>
			<a href="/register" class="text-white hover:text-emerald-400 font-medium transition-colors">
				Create your crew →
			</a>
		</div>
	</footer>
</div>
