<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { Check, Folder, FolderInput, Search, Users } from '@lucide/svelte';
	import BottomSheet from '../overlays/BottomSheet.svelte';
	import Icon from '../foundations/Icon.svelte';

	// Flow / Mobile Chat — Manage chat & long threads, step 3 (x73ZO): moving a chat into a project on
	// a phone. A 40-high search (surface-2, radius 12, 16pt so iOS doesn't zoom), the projects as
	// 54-high rows (folder, the name in mono at 14, its workspace at 12), the chosen one filled with
	// the accent check, the note saying who gains access, and "Move chat" (46, accent) at the foot.
	type Project = { id: string; name: string; workspace: string; /** The shared workspace's name, when others are in it. */ shared?: string };
	type Props = { open?: boolean; projects: Project[]; current?: string; onmove: (id: string) => void };
	let { open = $bindable(false), projects, current, onmove }: Props = $props();

	let query = $state('');
	let chosen = $state<string | undefined>(undefined);
	$effect(() => {
		if (open) {
			query = '';
			chosen = current;
		}
	});
	const shown = $derived(projects.filter((project) => project.name.toLowerCase().includes(query.trim().toLowerCase())));
	const target = $derived(projects.find((project) => project.id === chosen));
	const searchId = $props.id();

	function confirm() {
		if (!target || target.id === current) return;
		const id = target.id;
		open = false;
		onmove(id);
	}
</script>

<BottomSheet bind:open title={m.chat_move()}>
	<div class="flex flex-col gap-2 px-4 pb-2">
		<label for={searchId} class="flex h-10 items-center gap-2 rounded-lg bg-surface-2 px-3">
			<span class="inline-flex shrink-0 text-text-3"><Icon icon={Search} size={16} /></span>
			<input id={searchId} type="search" bind:value={query} placeholder={m.move_search()} aria-label={m.move_search()} class="min-w-0 flex-1 bg-transparent text-phone text-text outline-none placeholder:text-text-3" />
		</label>
		<div role="radiogroup" aria-label={m.chat_move()} class="flex flex-col gap-2">
			{#each shown as project (project.id)}
				{@const picked = project.id === chosen}
				<button
					type="button"
					role="radio"
					aria-checked={picked}
					class={['flex h-13.5 w-full cursor-pointer items-center gap-3 rounded-lg px-3 text-left outline-offset-2 focus-visible:outline-2 focus-visible:outline-focus', picked && 'bg-surface-2']}
					onclick={() => (chosen = project.id)}
				>
					<span class="inline-flex shrink-0 text-text-2"><Icon icon={Folder} size={18} /></span>
					<span class="flex min-w-0 flex-1 flex-col gap-px">
						<span class="truncate font-mono text-phone-sm text-text">{project.name}</span>
						<span class="truncate text-caption text-text-3">{project.workspace}</span>
					</span>
					{#if picked}<span class="inline-flex shrink-0 text-accent"><Icon icon={Check} size={18} /></span>{/if}
				</button>
			{:else}
				<p class="px-1 py-3 text-ui text-text-3">{m.move_none()}</p>
			{/each}
		</div>
		{#if target?.shared}
			<p class="flex gap-2 px-1 pt-1.5 pb-1 text-ui text-text-3">
				<span class="inline-flex shrink-0 pt-0.5"><Icon icon={Users} size={14} /></span>
				<span class="min-w-0 flex-1">{m.phone_move_shared({ workspace: target.shared })}</span>
			</p>
		{/if}
		<button type="button" disabled={!target || target.id === current} class="inline-flex h-11.5 w-full cursor-pointer items-center justify-center gap-2 rounded-lg bg-accent text-phone font-medium text-accent-ink outline-offset-2 focus-visible:outline-2 focus-visible:outline-focus disabled:cursor-default disabled:opacity-50" onclick={confirm}>
			{m.move_confirm()}
			<Icon icon={FolderInput} size={16} />
		</button>
	</div>
</BottomSheet>
