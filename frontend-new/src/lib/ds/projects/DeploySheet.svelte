<script lang="ts">
	import { Rocket } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import BottomSheet from '../overlays/BottomSheet.svelte';

	// Mobile / Deploy Sheet (Flow / Mobile Projects — Act from the card, step 6): before a draft's
	// first deploy on a phone. The sheet's title and how long it takes, then [4,16,0,16], 16 apart:
	// what happens (15, text-2), the facts in 42 rows (a 72 key at 14 in text-3, the value in mono at
	// 13), and Deploy, the whole width, 46 tall in accent with 12 corners. Uses is left out where the
	// plan's allowance isn't known, as on a desktop (DeployDialog).
	type Props = { open?: boolean; name: string; files: number; url: string; data: string; uses?: string; ondeploy: () => void; onclose?: () => void };
	let { open = $bindable(false), name, files, url, data, uses, ondeploy, onclose }: Props = $props();
	const facts = $derived([[m.projects_fact_url(), url], [m.projects_fact_data(), data], ...(uses ? [[m.projects_fact_uses(), uses]] : [])]);
</script>

<BottomSheet bind:open title={m.projects_deploy_title({ name })} subtitle={m.projects_deploy_sheet_time()} {onclose}>
	<div class="flex flex-col gap-4 px-4 pt-1">
		<p class="text-body text-text-2">{m.projects_deploy_body({ files })}</p>
		<dl class="flex flex-col">
			{#each facts as [key, value] (key)}
				<div class="flex gap-3 border-b border-border pt-3 pb-2.75 last:border-b-0">
					<dt class="w-18 shrink-0 text-phone-sm text-text-3">{key}</dt>
					<dd class="min-w-0 flex-1 truncate font-mono text-ui text-text">{value}</dd>
				</div>
			{/each}
		</dl>
		<button type="button" class="inline-flex h-11.5 w-full cursor-pointer items-center justify-center gap-2 rounded-lg bg-accent text-phone font-medium text-accent-ink outline-offset-2" onclick={() => ((open = false), ondeploy())}>
			{m.projects_action_deploy()}<Icon icon={Rocket} size={16} />
		</button>
	</div>
</BottomSheet>
