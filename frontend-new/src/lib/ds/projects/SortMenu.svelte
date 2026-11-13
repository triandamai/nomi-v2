<script lang="ts" module>
	import { ArrowDownAZ, CalendarPlus, Clock, Activity } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';

	export type ProjectSort = 'updated' | 'name' | 'status' | 'created';

	/** The orders the projects list offers, in the pen's order. */
	export const SORTS: ProjectSort[] = ['updated', 'name', 'status', 'created'];

	export function sortLabel(sort: ProjectSort): string {
		return { updated: m.projects_sort_updated, name: m.projects_sort_name, status: m.projects_sort_status, created: m.projects_sort_created }[sort]();
	}

	const ICONS = { updated: Clock, name: ArrowDownAZ, status: Activity, created: CalendarPlus };
</script>

<script lang="ts">
	import Icon from '../foundations/Icon.svelte';

	// Design System / Projects → Project / Sort Menu (MLYM5): the panel SortButton opens. 200 wide,
	// 4 in, its title in mono capitals (6 / 8 / 4 in), then the four orders as 30 rows, 2 apart, the
	// chosen one filled. The pen draws the rows as Menu Items; here they're menuitemradio rows
	// drawn the same way (MenuRow is a plain menuitem), so the chosen order is said, not only shown.
	type Props = {
		value: ProjectSort;
		onchoose: (sort: ProjectSort) => void;
		id?: string;
		element?: HTMLElement;
		[attribute: string]: unknown;
	};
	let { value, onchoose, id, element = $bindable(), ...rest }: Props = $props();

	const uid = $props.id();
</script>

<div
	bind:this={element}
	{id}
	role="menu"
	tabindex="-1"
	aria-labelledby="{uid}-title"
	{...rest}
	class={['flex w-50 flex-col gap-0.5 rounded-md bg-surface p-1 text-text shadow-float outline -outline-offset-1 outline-border-strong', rest.class as string | undefined]}
>
	<p id="{uid}-title" role="presentation" class="px-2 pt-1.5 pb-1 font-mono text-label tracking-label text-text-3 uppercase">{m.projects_sort_by()}</p>
	{#each SORTS as sort (sort)}
		<button
			type="button"
			role="menuitemradio"
			tabindex="-1"
			aria-checked={sort === value}
			onclick={() => onchoose(sort)}
			class={[
				'flex h-7.5 w-full cursor-pointer items-center gap-2 rounded-xs px-2 text-left text-ui text-text outline-none hover:bg-surface-2 focus-visible:bg-surface-2 pointer-coarse:min-h-11',
				sort === value && 'bg-surface-2',
			]}
		>
			<span class="inline-flex text-text-2"><Icon icon={ICONS[sort]} size={14} /></span>
			<span class="min-w-0 flex-1 truncate">{sortLabel(sort)}</span>
		</button>
	{/each}
</div>
