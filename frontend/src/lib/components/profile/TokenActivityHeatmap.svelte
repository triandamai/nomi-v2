<script lang="ts">
	type DayActivity = { date: string; tokens: number };

	let {
		activity = [],
		joinedDate = '',
		totalTokens = 0,
	}: {
		activity?: DayActivity[];
		joinedDate?: string;
		totalTokens?: number;
	} = $props();

	let mode: 'daily' | 'weekly' | 'cumulative' = $state('daily');
	let hoveredDay = $state<{ date: string; tokens: number; x: number; y: number } | null>(null);
	let selectedDate = $state<string | null>(null);

	// Map activity by date
	const activityMap = $derived(new Map(activity.map((a) => [a.date, a.tokens])));

	// Calculate 52 weeks (364 days) grid ending today
	const gridData = $derived.by(() => {
		const now = new Date();
		// Find end of current week (Saturday)
		const dayOfWeek = now.getDay();
		const endDate = new Date(now);
		endDate.setDate(now.getDate() + (6 - dayOfWeek));

		const startDate = new Date(endDate);
		startDate.setDate(endDate.getDate() - (52 * 7 - 1));

		const weeks: {
			weekIndex: number;
			monthLabel?: string;
			days: { dateStr: string; date: Date; tokens: number; cumulative: number; level: number }[];
		}[] = [];

		let runningCumulative = 0;
		const maxDaily = Math.max(1, ...activity.map((a) => a.tokens));

		let curDate = new Date(startDate);
		let currentMonth = -1;

		for (let w = 0; w < 52; w++) {
			const days = [];
			let weekMonthLabel: string | undefined = undefined;

			for (let d = 0; d < 7; d++) {
				const dateStr = curDate.toISOString().slice(0, 10);
				const tokens = activityMap.get(dateStr) ?? 0;
				runningCumulative += tokens;

				if (curDate.getMonth() !== currentMonth && d === 0) {
					currentMonth = curDate.getMonth();
					weekMonthLabel = curDate.toLocaleDateString('en-US', { month: 'short' });
				}

				// Level 0 to 4
				let level = 0;
				if (tokens > 0) {
					const ratio = tokens / maxDaily;
					if (ratio > 0.6) level = 4;
					else if (ratio > 0.3) level = 3;
					else if (ratio > 0.1) level = 2;
					else level = 1;
				}

				days.push({
					dateStr,
					date: new Date(curDate),
					tokens,
					cumulative: runningCumulative,
					level,
				});

				curDate.setDate(curDate.getDate() + 1);
			}

			weeks.push({
				weekIndex: w,
				monthLabel: weekMonthLabel,
				days,
			});
		}

		return weeks;
	});

	function formatNumber(num: number): string {
		if (num >= 1_000_000) return (num / 1_000_000).toFixed(1) + 'M';
		if (num >= 1_000) return (num / 1_000).toFixed(1) + 'K';
		return num.toLocaleString();
	}

	function formatDate(dateStr: string): string {
		const d = new Date(dateStr + 'T00:00:00');
		return d.toLocaleDateString('en-US', { day: 'numeric', month: 'short', year: 'numeric' });
	}

	function getColorClass(day: { level: number; tokens: number; cumulative: number }): string {
		if (mode === 'daily') {
			switch (day.level) {
				case 4:
					return 'bg-[#4ade80]'; // bright emerald
				case 3:
					return 'bg-[#22c55e]';
				case 2:
					return 'bg-[#15803d]';
				case 1:
					return 'bg-[#14532d]';
				default:
					return 'bg-[#191c21]';
			}
		} else if (mode === 'cumulative') {
			const maxCum = Math.max(1, totalTokens);
			const ratio = day.cumulative / maxCum;
			if (ratio > 0.75) return 'bg-[#4ade80]';
			if (ratio > 0.5) return 'bg-[#22c55e]';
			if (ratio > 0.25) return 'bg-[#15803d]';
			if (ratio > 0.05) return 'bg-[#14532d]';
			return 'bg-[#191c21]';
		} else {
			// weekly
			return day.level > 0 ? 'bg-[#22c55e]' : 'bg-[#191c21]';
		}
	}
</script>

<div class="rounded-xl border border-[#22252a] bg-[#131518] p-4 sm:p-5 flex flex-col gap-4">
	<!-- Top Bar -->
	<div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
		<div class="flex items-baseline gap-2">
			<h2 class="text-sm font-semibold text-white tracking-tight">Token activity</h2>
			<span class="text-xs text-[#9aa0a6]">{formatNumber(totalTokens)} tokens in the last year</span>
		</div>
		<!-- Segment Tabs -->
		<div class="inline-flex rounded-lg border border-[#26292f] bg-[#1a1d22] p-0.5 self-start sm:self-auto text-xs">
			<button
				type="button"
				class="rounded-md px-2.5 py-1 font-medium transition-colors {mode === 'daily'
					? 'bg-[#2a2e36] text-white shadow-sm'
					: 'text-[#9aa0a6] hover:text-white'}"
				onclick={() => (mode = 'daily')}
			>
				Daily
			</button>
			<button
				type="button"
				class="rounded-md px-2.5 py-1 font-medium transition-colors {mode === 'weekly'
					? 'bg-[#2a2e36] text-white shadow-sm'
					: 'text-[#9aa0a6] hover:text-white'}"
				onclick={() => (mode = 'weekly')}
			>
				Weekly
			</button>
			<button
				type="button"
				class="rounded-md px-2.5 py-1 font-medium transition-colors {mode === 'cumulative'
					? 'bg-[#2a2e36] text-white shadow-sm'
					: 'text-[#9aa0a6] hover:text-white'}"
				onclick={() => (mode = 'cumulative')}
			>
				Cumulative
			</button>
		</div>
	</div>

	<!-- Heatmap Scroll Container -->
	<div class="relative overflow-x-auto pb-2 -mx-1 px-1">
		<div class="inline-flex flex-col gap-1.5 min-w-[680px]">
			<div class="flex gap-1.5 items-start">
				<!-- Day of Week Labels -->
				<div class="flex flex-col justify-between text-[10px] text-[#70757a] pr-1.5 select-none pt-0.5" style="height: 98px;">
					<span>Mon</span>
					<span>Wed</span>
					<span>Fri</span>
				</div>

				<!-- Weeks Columns -->
				<div class="flex gap-[3px]">
					{#each gridData as week}
						<div class="flex flex-col gap-[3px]">
							{#each week.days as day}
								{@const isSelected = selectedDate === day.dateStr}
								<button
									type="button"
									aria-label="{day.tokens} tokens on {day.dateStr}"
									class="w-[11px] h-[11px] rounded-[2px] transition-all {getColorClass(day)} {isSelected
										? 'ring-2 ring-white ring-offset-1 ring-offset-[#131518] z-10'
										: 'hover:ring-1 hover:ring-white/80'}"
									onmouseenter={(e) => {
										const rect = e.currentTarget.getBoundingClientRect();
										hoveredDay = {
											date: day.dateStr,
											tokens: mode === 'cumulative' ? day.cumulative : day.tokens,
											x: rect.left + rect.width / 2,
											y: rect.top,
										};
									}}
									onmouseleave={() => {
										hoveredDay = null;
									}}
									onclick={() => {
										selectedDate = selectedDate === day.dateStr ? null : day.dateStr;
									}}
								></button>
							{/each}
						</div>
					{/each}
				</div>
			</div>

			<!-- Month Labels Row -->
			<div class="flex pl-8 text-[10px] text-[#70757a] select-none gap-[3px]">
				{#each gridData as week}
					<div class="w-[11px] text-left truncate">
						{#if week.monthLabel}
							<span>{week.monthLabel}</span>
						{/if}
					</div>
				{/each}
			</div>
		</div>
	</div>

	<!-- Tooltip Portal (Floating) -->
	{#if hoveredDay}
		<div
			class="fixed pointer-events-none z-50 -translate-x-1/2 -translate-y-full mb-2 px-2.5 py-1.5 rounded-lg bg-[#22252a] text-white text-xs shadow-xl border border-[#30343c] whitespace-nowrap"
			style="left: {hoveredDay.x}px; top: {hoveredDay.y - 6}px;"
		>
			<span class="font-semibold text-emerald-400">{hoveredDay.tokens.toLocaleString()} tokens</span>
			<span class="text-[#9aa0a6] ml-1">on {formatDate(hoveredDay.date)}</span>
		</div>
	{/if}

	<!-- Footer Bar -->
	<div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-2 pt-2 border-t border-[#1e2126] text-[11px] text-[#9aa0a6]">
		<div>
			Most active Tue · 21:00-23:00 WIB
			{#if joinedDate}
				· joined {joinedDate}
			{/if}
		</div>
		<div class="flex items-center gap-1.5 select-none self-end sm:self-auto">
			<span>Less</span>
			<span class="w-2.5 h-2.5 rounded-[2px] bg-[#191c21]"></span>
			<span class="w-2.5 h-2.5 rounded-[2px] bg-[#14532d]"></span>
			<span class="w-2.5 h-2.5 rounded-[2px] bg-[#15803d]"></span>
			<span class="w-2.5 h-2.5 rounded-[2px] bg-[#22c55e]"></span>
			<span class="w-2.5 h-2.5 rounded-[2px] bg-[#4ade80]"></span>
			<span>More</span>
		</div>
	</div>
</div>
