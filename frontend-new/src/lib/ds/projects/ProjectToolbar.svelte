<script lang="ts" module>
	export type ProjectFilter = 'all' | 'live' | 'building' | 'attention';
</script>

<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import FilterTabs from './FilterTabs.svelte';
	import ProjectSearch from './ProjectSearch.svelte';
	import SortButton from './SortButton.svelte';
	import type { ProjectSort } from './SortMenu.svelte';

	// Design System / Projects → Project / Toolbar (TnDHg): the filter tabs with their counts, then,
	// at the end, the search (220) and the sort, 8 apart, 30 tall. It spans the grid it sits over.
	type Props = {
		counts: Record<ProjectFilter, number>;
		filter?: ProjectFilter;
		query?: string;
		sort?: ProjectSort;
		onfilter?: (filter: ProjectFilter) => void;
		onsearch?: (query: string) => void;
		onsort?: (sort: ProjectSort) => void;
	};
	let { counts, filter = $bindable('all'), query = $bindable(''), sort = $bindable('updated'), onfilter, onsearch, onsort }: Props = $props();

	const options = $derived([
		{ value: 'all', label: m.projects_filter_all(), count: counts.all },
		{ value: 'live', label: m.projects_filter_live(), count: counts.live },
		{ value: 'building', label: m.projects_filter_building(), count: counts.building },
		{ value: 'attention', label: m.projects_filter_attention(), count: counts.attention },
	]);
</script>

<div class="flex w-full flex-wrap items-center gap-2">
	<FilterTabs
		label={m.projects_filter()}
		{options}
		value={filter}
		onchange={(value) => {
			filter = value as ProjectFilter;
			onfilter?.(filter);
		}}
	/>
	<span class="ml-auto flex items-center gap-2">
		<ProjectSearch bind:value={query} oninput={onsearch} />
		<SortButton bind:value={sort} onchange={onsort} />
	</span>
</div>
