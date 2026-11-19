<script lang="ts">
	import { Check, Lock, Users } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Mobile / Workspace Sheet's body (Flow / Mobile Projects — Workspaces & people, a9NrP): the
	// workspaces as cards, [4,16,0,16], 8 apart. Each card 64 tall, [12,14], 14 corners and no
	// stroke: a 36 tile on the page's ground (9 corners, the initial at 15 / 600), the name at 16 /
	// 500 over its line at 13 in text-3; the one projects come from filled surface-2 with the 20
	// accent check at its end. Under them the boundary (C8evkZ: the page's ground, 12 corners, 14 in,
	// 10 apart): what's shared in a workspace (users) and what's always personal (lock), each a 14 /
	// 500 title over a 13 line, the 16 icon 10 before them. Left out (§9): the PERSONAL tag and Join.
	type Workspace = { id: string; name: string; meta: string };
	type Props = { workspaces: Workspace[]; chosen?: string | null; onchoose: (id: string) => void };
	let { workspaces, chosen = null, onchoose }: Props = $props();
</script>

{#snippet rule(icon: typeof Users, title: string, body: string)}
	<div class="flex gap-2.5">
		<span class="inline-flex shrink-0 text-text-3"><Icon {icon} size={16} /></span>
		<span class="flex min-w-0 flex-1 flex-col gap-0.5">
			<span class="text-phone-sm font-medium text-text">{title}</span>
			<span class="text-ui text-text-3">{body}</span>
		</span>
	</div>
{/snippet}

<div class="flex flex-col gap-2 px-4 pt-1">
	{#each workspaces as workspace (workspace.id)}
		{@const here = workspace.id === chosen}
		<button
			type="button"
			aria-pressed={here}
			class={['flex h-16 w-full cursor-pointer items-center gap-3 rounded-xl px-3.5 text-left -outline-offset-2', here && 'bg-surface-2']}
			onclick={() => onchoose(workspace.id)}
		>
			<span aria-hidden="true" class="inline-flex size-9 shrink-0 items-center justify-center rounded-workspace bg-bg text-body font-semibold text-text">{workspace.name.charAt(0).toUpperCase()}</span>
			<span class="flex min-w-0 flex-1 flex-col gap-0.5">
				<span class="truncate text-phone font-medium text-text">{workspace.name}</span>
				<span class="truncate text-ui text-text-3">{workspace.meta}</span>
			</span>
			{#if here}<span class="inline-flex shrink-0 text-accent"><Icon icon={Check} size={20} /></span>{/if}
		</button>
	{/each}
	<div class="flex flex-col gap-2.5 rounded-lg bg-bg p-3.5">
		{@render rule(Users, m.workspace_shared_title(), m.workspace_shared_body())}
		{@render rule(Lock, m.workspace_personal_title(), m.workspace_personal_body())}
	</div>
</div>
