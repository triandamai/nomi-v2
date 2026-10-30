<script lang="ts">
	import { Check, Folder, FolderInput, Users } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import Dialog from '../overlays/Dialog.svelte';
	import Search from '../inputs/Search.svelte';

	// Chat responses → Move Dialog: pick the project a chat belongs to. Moving into a shared
	// workspace says who will see it, and what stays yours.
	type Project = { id: string; name: string; workspace: string; /** The shared workspace's name, when others are in it. */ shared?: string };
	type Props = { open?: boolean; inline?: boolean; projects: Project[]; initialChosen?: string; onmove: (id: string) => void };
	let { open = $bindable(false), inline = false, projects, initialChosen, onmove }: Props = $props();

	const id = $props.id();
	let query = $state('');
	const initial = () => initialChosen;
	let chosen = $state<string | undefined>(initial());

	const shown = $derived(projects.filter((project) => project.name.toLowerCase().includes(query.trim().toLowerCase())));
	const target = $derived(projects.find((project) => project.id === chosen));
</script>

<Dialog bind:open {inline} wide title={m.move_title()}>
	<div class="flex flex-col gap-3 pt-1">
		<Search placeholder={m.move_search()} bind:value={query} />
		<div role="radiogroup" aria-label={m.move_title()} class="flex flex-col gap-0.5">
			{#each shown as project (project.id)}
				<label class="relative flex h-9 cursor-pointer items-center gap-2.5 rounded-xs px-2 hover:bg-surface-2 has-checked:bg-surface-2 has-focus-visible:outline-2 has-focus-visible:outline-focus pointer-coarse:min-h-11">
					<input type="radio" name="{id}-project" value={project.id} bind:group={chosen} class="absolute inset-0 cursor-pointer appearance-none rounded-xs outline-none" />
					<span class="inline-flex shrink-0 text-text-2"><Icon icon={Folder} size={14} /></span>
					<span class="min-w-0 flex-1 truncate font-mono text-caption text-text">{project.name}</span>
					<span class="font-mono text-label text-text-3">{project.workspace}</span>
					{#if chosen === project.id}<span class="inline-flex text-accent"><Icon icon={Check} size={14} /></span>{/if}
				</label>
			{:else}
				<p class="px-2 py-2 text-caption text-text-3">{m.move_none()}</p>
			{/each}
		</div>
		{#if target?.shared}
			<p class="flex gap-2 text-caption text-text-3">
				<span class="inline-flex h-4 shrink-0 items-center"><Icon icon={Users} size={12} /></span>{m.move_shared({ workspace: target.shared })}
			</p>
		{/if}
	</div>
	{#snippet actions()}
		<Button label={m.action_cancel()} variant="secondary" onclick={() => (open = false)} />
		<Button
			label={m.move_confirm()}
			iconEnd={FolderInput}
			disabled={!chosen}
			onclick={() => {
				if (!chosen) return;
				onmove(chosen);
				open = false;
			}}
		/>
	{/snippet}
</Dialog>
