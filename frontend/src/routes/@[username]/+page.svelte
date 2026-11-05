<script lang="ts">
	import { ArrowRight, ExternalLink, Flame, Link2 } from '@lucide/svelte';
	import Mascot from '$lib/components/ds/agents/Mascot.svelte';
	import Status from '$lib/components/ds/status/Status.svelte';
	import Button from '$lib/components/ds/actions/Button.svelte';
	import Icon from '$lib/components/ds/foundations/Icon.svelte';
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

<div class="min-h-[100dvh] bg-bg text-text flex flex-col justify-between font-sans pb-[calc(env(safe-area-inset-bottom,0px)+2rem)]">
	<!-- Public Top Navigation Bar -->
	<header class="border-b border-border bg-surface/80 backdrop-blur-md sticky top-0 z-40 px-4 py-3 sm:px-8">
		<div class="max-w-5xl mx-auto flex items-center justify-between">
			<a href="/" class="flex items-center gap-2 text-text font-bold tracking-tight text-ui hover:opacity-90 transition-opacity">
				<span class="w-6 h-6 rounded-md bg-accent flex items-center justify-center text-accent-ink font-mono font-bold text-caption">n</span>
				<span>nomi</span>
			</a>

			<div class="flex items-center gap-2 sm:gap-3">
				<Button
					variant="secondary"
					label={copiedLink ? 'Copied link!' : 'Share'}
					iconStart={Link2}
					onclick={copyLink}
				/>
				<Button
					variant="primary"
					label="Start building"
					href="/register"
				/>
			</div>
		</div>
	</header>

	<!-- Main Profile Content -->
	<main class="flex-1 max-w-5xl w-full mx-auto px-4 py-8 sm:px-8 flex flex-col gap-8">
		<!-- Profile Card Header -->
		<div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 p-6 rounded-2xl border border-border bg-surface">
			<div class="flex items-center gap-4">
				{#if profile.avatar_url}
					<img
						src={profile.avatar_url}
						alt={profile.display_name || profile.username}
						class="w-16 h-16 sm:w-20 sm:h-20 rounded-2xl object-cover border border-border bg-surface-2 shadow-sm shrink-0"
					/>
				{:else}
					<div class="w-16 h-16 sm:w-20 sm:h-20 rounded-2xl bg-surface-2 border border-border flex items-center justify-center text-heading font-bold text-text shadow-sm shrink-0 font-mono">
						{initials}
					</div>
				{/if}

				<div class="flex flex-col min-w-0">
					<h1 class="text-display font-bold text-text tracking-tight truncate">
						{profile.display_name || profile.username}
					</h1>
					<div class="flex flex-wrap items-center gap-1.5 text-caption text-text-2 mt-1.5">
						<span class="text-text font-medium">@{profile.username}</span>
						{#if profile.location}
							<span class="text-text-3">·</span>
							<span>{profile.location}</span>
						{/if}
						<span class="text-text-3">·</span>
						<span>joined {joinedFormatted}</span>
						<span class="text-text-3">·</span>
						<span class="px-2 py-0.5 rounded-md bg-surface-2 text-text-2 text-micro font-medium border border-border">
							{profile.plan_name}
						</span>
					</div>
				</div>
			</div>

			<div class="flex items-center gap-2 self-start sm:self-auto">
				<Button
					variant="primary"
					label="Build with Nomi"
					iconEnd={ArrowRight}
					href="/register"
				/>
			</div>
		</div>

		<!-- Showcase Live Projects (Always visible if any) -->
		{#if profile.showcase_projects && profile.showcase_projects.length > 0}
			<section class="flex flex-col gap-3">
				<div class="flex items-center justify-between">
					<h2 class="text-ui font-semibold tracking-wide text-text uppercase">Featured Projects</h2>
					<span class="text-caption text-text-3">{profile.showcase_projects.length} live app{profile.showcase_projects.length > 1 ? 's' : ''}</span>
				</div>

				<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
					{#each profile.showcase_projects as project}
						<div class="rounded-xl border border-border bg-surface p-4 flex flex-col justify-between gap-4 hover:border-border-strong transition-colors group">
							<div class="flex items-start justify-between gap-2">
								<div class="flex items-center gap-3">
									<div class="w-10 h-10 rounded-lg bg-surface-2 border border-border flex flex-col items-center justify-center font-mono text-[9px] text-accent shrink-0">
										<span class="w-1.5 h-1.5 rounded-full bg-accent mb-0.5"></span>
										LIVE
									</div>
									<div class="flex flex-col min-w-0">
										<h3 class="text-ui font-semibold text-text group-hover:text-accent transition-colors truncate">
											{project.name}
										</h3>
										<span class="text-caption text-text-3">
											built with Koda · {formatDuration(project.build_duration_secs ?? 252)}
										</span>
									</div>
								</div>
							</div>

							{#if project.description}
								<p class="text-caption text-text-2 line-clamp-2 leading-relaxed">
									{project.description}
								</p>
							{/if}

							<div class="pt-2 border-t border-border flex items-center justify-between text-caption">
								<span class="text-text-3">Production</span>
								{#if project.deploy_url}
									<a
										href={project.deploy_url}
										target="_blank"
										rel="noreferrer"
										class="inline-flex items-center gap-1 text-caption text-accent hover:underline font-medium"
									>
										<span>Visit live app</span>
										<Icon icon={ExternalLink} size={12} />
									</a>
								{:else}
									<span class="text-caption text-text-3">Live app</span>
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
					<div class="rounded-xl border border-border bg-surface p-3.5 sm:p-4 flex flex-col justify-between">
						<div>
							<div class="text-display font-bold text-text tracking-tight">{formatTokens(profile.stats.lifetime_tokens)}</div>
							<div class="text-ui text-text-2 mt-1 font-medium">Lifetime tokens</div>
						</div>
						<div class="text-caption text-text-3 mt-2 truncate">since {joinedFormatted.slice(0, 6)}</div>
					</div>

					<div class="rounded-xl border border-border bg-surface p-3.5 sm:p-4 flex flex-col justify-between">
						<div>
							<div class="text-display font-bold text-text tracking-tight">{formatTokens(profile.stats.peak_day_tokens)}</div>
							<div class="text-ui text-text-2 mt-1 font-medium">Peak day</div>
						</div>
						<div class="text-caption text-text-3 mt-2 truncate">{profile.stats.peak_day_date || 'Peak'}</div>
					</div>

					<div class="rounded-xl border border-border bg-surface p-3.5 sm:p-4 flex flex-col justify-between">
						<div>
							<div class="text-display font-bold text-text tracking-tight">{formatDuration(profile.stats.longest_task_secs ?? 252)}</div>
							<div class="text-ui text-text-2 mt-1 font-medium">Longest task</div>
						</div>
						<div class="text-caption text-text-3 mt-2 truncate">Koda · {profile.stats.longest_task_name || 'build'}</div>
					</div>

					<div class="rounded-xl border border-border bg-surface p-3.5 sm:p-4 flex flex-col justify-between">
						<div>
							<div class="text-display font-bold text-text tracking-tight">{profile.stats.longest_streak_days} days</div>
							<div class="text-ui text-text-2 mt-1 font-medium">Longest streak</div>
						</div>
						<div class="text-caption text-text-3 mt-2 truncate">{profile.stats.longest_streak_range || 'Personal best'}</div>
					</div>

					<div class="rounded-xl border border-border bg-surface p-3.5 sm:p-4 flex flex-col justify-between col-span-2 sm:col-span-1">
						<div>
							<div class="text-display font-bold text-text tracking-tight flex items-center gap-1.5">
								{profile.stats.current_streak_days} days <span class="text-accent text-body inline-flex"><Icon icon={Flame} size={20} /></span>
							</div>
							<div class="text-ui text-text-2 mt-1 font-medium">Current streak</div>
						</div>
						<div class="text-caption text-text-3 mt-2 truncate">
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
					<div class="rounded-xl border border-border bg-surface p-5 flex flex-col gap-3">
						<div class="flex items-center justify-between">
							<h3 class="text-ui font-semibold text-text tracking-tight">Models used most</h3>
							<span class="text-caption text-text-3">last 30 days</span>
						</div>
						<div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-4">
							{#each profile.top_models as model}
								<div class="flex flex-col gap-1.5 p-3 rounded-lg bg-surface-2 border border-border">
									<div class="flex items-center justify-between text-caption">
										<span class="text-text font-mono truncate">{model.label}</span>
										<span class="text-accent font-semibold">{model.percentage}%</span>
									</div>
									<div class="w-full h-1.5 rounded-full bg-surface overflow-hidden">
										<div class="h-full rounded-full bg-accent" style="width: {model.percentage}%;"></div>
									</div>
									<span class="text-caption text-text-3">{formatTokens(model.tokens)} tokens</span>
								</div>
							{/each}
						</div>
					</div>
				{/if}
			</section>
		{/if}

		<!-- Crew Section (If enabled) -->
		{#if profile.show_crew_publicly && profile.crew_usage && profile.crew_usage.length > 0}
			<section class="rounded-xl border border-border bg-surface p-5 flex flex-col gap-4">
				<div class="flex items-center justify-between">
					<h3 class="text-ui font-semibold text-text tracking-tight">Agent Crew</h3>
					<span class="text-caption text-text-3">Collaborators</span>
				</div>
				<div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-3">
					{#each profile.crew_usage as member}
						<div class="flex items-center gap-3 p-3 rounded-lg bg-surface-2 border border-border">
							<Mascot agent={member.agent_id as CrewId} size={32} />
							<div class="flex flex-col min-w-0 flex-1">
								<span class="text-ui font-bold text-text truncate">{member.name}</span>
								<div class="w-full h-1.5 rounded-full bg-surface overflow-hidden mt-1">
									<div
										class="h-full rounded-full"
										style="background: var(--crew-{member.agent_id}, var(--color-accent)); width: {Math.min(100, Math.max(8, (member.count / maxCrewCount) * 100))}%;"
									></div>
								</div>
								<span class="text-nano text-text-3 mt-1">
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
	<footer class="border-t border-border bg-bg py-8 px-4 text-center text-caption text-text-3">
		<div class="max-w-5xl mx-auto flex flex-col sm:flex-row items-center justify-between gap-4">
			<div class="flex items-center gap-2">
				<span class="w-4 h-4 rounded-md bg-accent-bg text-accent font-mono font-bold text-nano flex items-center justify-center">n</span>
				<span class="text-text-2">Built with Nomi</span>
			</div>
			<div>A small crew of agents that remembers, builds, and works for you.</div>
			<a href="/register" class="text-text hover:text-accent font-medium transition-colors">
				Create your crew →
			</a>
		</div>
	</footer>
</div>
