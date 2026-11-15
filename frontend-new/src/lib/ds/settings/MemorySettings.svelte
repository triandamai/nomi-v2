<script lang="ts" module>
	export type MemorySettingsValues = {
		learn_from_chats: boolean;
		keep_profile: boolean;
		search_past_chats: boolean;
		memory_budget: number;
		profile_budget: number;
		memory_used: number;
		profile_used: number;
	};
	export type MemorySettingsChange = Partial<Pick<MemorySettingsValues, 'learn_from_chats' | 'keep_profile' | 'search_past_chats' | 'memory_budget' | 'profile_budget'>>;
</script>

<script lang="ts">
	import Toggle from '../inputs/Toggle.svelte';

	// App / Settings / Memory (hANTE) → Settings Column: 380 on the page's ground with a stroke on its
	// left. Each group is [14,20], 12 apart, over a bottom stroke, its name a mono label:
	// - Learning: Learn from chats, Keep a profile, Search past chats, each its title (13, medium) and
	//   line (12, text-3) beside a switch, 16 apart.
	// - Budget: Memory and Profile, each its name beside a 92 × 28 number field in tokens, a 4px
	//   track filled by what's used (text-2, or warning from 80%) and "1,812 / 2,200 used · loaded
	//   into every reply" under it in mono (warning too, from 80%).
	// The fields are text with a numeric keyboard (the pen draws no spinner); "2,200" reads as 2200.
	// A budget is said once it's in the backend's range (memory 500–100,000, profile 100–4,000), on
	// Enter or when the field is left; outside it, the field says the range and nothing is sent.
	// The pen's Storage (Provider), Context & compression and Import from another AI are left out:
	// nothing in the backend is behind them.
	type Props = {
		values: MemorySettingsValues;
		/** For the numbers: "1,812" or "1.812". */
		language: string;
		labels: {
			learning: string;
			budget: string;
			learn: { title: string; body: string };
			profile: { title: string; body: string };
			search: { title: string; body: string };
			memoryBudget: string;
			profileBudget: string;
			unit: string;
			used: (used: string, budget: string) => string;
			outOfRange: (min: string, max: string) => string;
		};
		onchange?: (change: MemorySettingsChange) => void;
	};
	let { values, language, labels, onchange }: Props = $props();

	const RANGES = { memory_budget: [500, 100_000], profile_budget: [100, 4_000] } as const;
	const number = (value: number) => value.toLocaleString(language === 'id' ? 'id-ID' : 'en-US');
	const id = $props.id();
	let wrong = $state<Partial<Record<keyof typeof RANGES, boolean>>>({});

	function setBudget(key: keyof typeof RANGES, raw: string) {
		const value = Number(raw.replace(/[\s.,]/g, ''));
		const [min, max] = RANGES[key];
		if (!Number.isInteger(value) || value < min || value > max) return void (wrong = { ...wrong, [key]: true });
		wrong = { ...wrong, [key]: false };
		if (value !== values[key]) onchange?.({ [key]: value });
	}

	const switches = $derived([
		{ key: 'learn_from_chats', ...labels.learn },
		{ key: 'keep_profile', ...labels.profile },
		{ key: 'search_past_chats', ...labels.search },
	] as const);
	const budgets = $derived([
		{ key: 'memory_budget', label: labels.memoryBudget, used: values.memory_used },
		{ key: 'profile_budget', label: labels.profileBudget, used: values.profile_used },
	] as const);
</script>

<div data-memory-settings class="flex w-95 max-w-full flex-col bg-bg">
	<section aria-labelledby="{id}-learning" class="flex flex-col gap-3 border-b border-border px-5 pt-3.5 pb-3.25">
		<h2 id="{id}-learning" class="font-mono text-label tracking-label text-text-3 uppercase">{labels.learning}</h2>
		{#each switches as row (row.key)}
			<div class="flex items-center gap-4">
				<span class="flex min-w-0 flex-1 flex-col gap-0.5">
					<span class="text-ui font-medium text-text">{row.title}</span>
					<span class="text-caption text-text-3">{row.body}</span>
				</span>
				<Toggle label={row.title} checked={values[row.key]} onchange={(on) => onchange?.({ [row.key]: on })} />
			</div>
		{/each}
	</section>
	<section aria-labelledby="{id}-budget" class="flex flex-col gap-3 border-b border-border px-5 pt-3.5 pb-3.25">
		<h2 id="{id}-budget" class="font-mono text-label tracking-label text-text-3 uppercase">{labels.budget}</h2>
		{#each budgets as row (row.key)}
			{@const budget = values[row.key]}
			{@const share = Math.min(1, row.used / budget)}
			{@const warn = share >= 0.8}
			{@const [min, max] = RANGES[row.key]}
			<div class="flex flex-col gap-1.5">
				<div class="flex items-center gap-3">
					<label for="{id}-{row.key}" class="min-w-0 flex-1 text-ui font-medium text-text">{row.label}</label>
					<span class="flex h-7 w-23 shrink-0 items-center gap-1.5 rounded-sm border border-border-strong bg-surface px-1.75 focus-within:border-accent focus-within:shadow-focus">
						<input
							id="{id}-{row.key}"
							type="text"
							inputmode="numeric"
							autocomplete="off"
							value={number(budget)}
							aria-invalid={wrong[row.key] ?? false}
							aria-describedby="{id}-{row.key}-note"
							onchange={(event) => setBudget(row.key, event.currentTarget.value)}
							onkeydown={(event) => {
								if (event.key === 'Enter') setBudget(row.key, event.currentTarget.value);
							}}
							class="w-full min-w-0 bg-transparent font-mono text-caption text-text outline-none"
						/>
						<span aria-hidden="true" class="font-mono text-label text-text-3">{labels.unit}</span>
					</span>
				</div>
				<span data-track aria-hidden="true" class="relative h-1 w-full overflow-hidden rounded-full bg-surface-2">
					<span class={['absolute inset-y-0 left-0 rounded-full', warn ? 'bg-warning' : 'bg-text-2']} style:width="{share * 100}%"></span>
				</span>
				<span id="{id}-{row.key}-note" class={['font-mono text-label', wrong[row.key] ? 'text-danger' : warn ? 'text-warning' : 'text-text-3']}>
					{wrong[row.key] ? labels.outOfRange(number(min), number(max)) : labels.used(number(row.used), number(budget))}
				</span>
			</div>
		{/each}
	</section>
</div>
