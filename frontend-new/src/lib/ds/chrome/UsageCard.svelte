<script lang="ts">
	import { ArrowUpRight } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Meter from '../status/Meter.svelte';

	/** The plan and how much of it is used this month, at the foot of the sidebar. */
	type Props = { plan: string; used: number; caption: string; href: string };
	let { plan, used, caption, href }: Props = $props();
</script>

<div class="flex flex-col gap-2.5 rounded-md border border-border p-3">
	<Meter label={plan} value={used} {caption} roomy />
	<a {href} class="inline-flex items-center gap-1.5 self-start rounded-xs text-ui font-medium text-text outline-offset-2 hover:underline">
		{m.nav_plan_usage()}
		<Icon icon={ArrowUpRight} size={14} />
	</a>
</div>
