<script lang="ts" module>
	export type Repo = {
		id: string;
		/** "mario/kasir-toko". */
		name: string;
		/** "SvelteKit · private · updated 3d ago", or why Nomi can't see it. */
		meta: string;
		/** Set when Nomi can't see the repo yet: where on GitHub to grant it. */
		grantHref?: string;
	};
</script>

<script lang="ts">
	import { ArrowRight, ArrowUpRight, BookMarked, Check, GitBranch, Lock } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import Search from '../inputs/Search.svelte';
	import ProjectDialog from './ProjectDialog.svelte';

	// Design System / Projects → Project / Repo Picker Dialog (tzhsm): which repo to import. 460
	// wide: the Search (with ⌘K, which focuses it), the repos as a radio group (44 rows, 8 in, mono
	// name over mono meta; the chosen one filled and checked in mint), a repo Nomi can't see yet
	// locked in text-3 with Grant access (12px, medium: smaller than Link, as the pen sets it), and
	// what happens next. Import <name> is the primary action.
	type Props = {
		open?: boolean;
		inline?: boolean;
		/** Where the project will live. */
		workspace: string;
		/** Whose repos these are: "mario". */
		owner: string;
		repos: Repo[];
		value?: string;
		query?: string;
		onimport: (id: string) => void;
		onclose?: () => void;
	};
	let { open = $bindable(false), inline = false, workspace, owner, repos, value = $bindable(), query = $bindable(''), onimport, onclose }: Props = $props();

	let buttons: HTMLButtonElement[] = $state([]);
	let list: HTMLElement | undefined = $state();
	const shown = $derived(repos.filter((repo) => repo.name.toLowerCase().includes(query.trim().toLowerCase())));
	const choosable = $derived(shown.filter((repo) => !repo.grantHref));
	const chosen = $derived(choosable.find((repo) => repo.id === value));

	function onradiokey(event: KeyboardEvent) {
		const step = event.key === 'ArrowDown' || event.key === 'ArrowRight' ? 1 : event.key === 'ArrowUp' || event.key === 'ArrowLeft' ? -1 : 0;
		if (!step || !choosable.length) return;
		event.preventDefault();
		const at = choosable.findIndex((repo) => repo.id === value);
		const next = choosable[(at + step + choosable.length) % choosable.length];
		value = next.id;
		buttons[shown.indexOf(next)]?.focus();
	}

	function onkeydown(event: KeyboardEvent) {
		if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
			event.preventDefault();
			list?.parentElement?.querySelector<HTMLInputElement>('input[type="search"]')?.focus();
		}
	}
</script>

<ProjectDialog bind:open {inline} {onclose} {onkeydown} width={460} title={m.projects_repos_title({ workspace })}>
	<Search placeholder={m.projects_repos_search({ owner })} shortcut="⌘K" bind:value={query} />
	<div bind:this={list} role="radiogroup" aria-label={m.projects_repos_list()} tabindex="-1" onkeydown={onradiokey} class="flex flex-col gap-0.5">
		{#each shown as repo, index (repo.id)}
			{#if repo.grantHref}
				<div class="flex h-11 items-center gap-2.5 px-2">
					<span class="inline-flex text-text-3"><Icon icon={Lock} size={14} /></span>
					<span class="flex min-w-0 flex-1 flex-col gap-0.5">
						<span class="truncate font-mono text-caption text-text-3">{repo.name}</span>
						<span class="truncate font-mono text-label text-text-3">{repo.meta}</span>
					</span>
					<a
						href={repo.grantHref}
						target="_blank"
						rel="noopener noreferrer"
						class="inline-flex shrink-0 items-center gap-1 rounded-xs text-caption font-medium text-text outline-offset-2 hover:underline hover:decoration-border-strong hover:underline-offset-2"
					>
						{m.projects_repos_grant()}
						<span class="inline-flex text-text-2"><Icon icon={ArrowUpRight} size={14} /></span>
					</a>
				</div>
			{:else}
				{@const checked = repo.id === chosen?.id}
				<button
					bind:this={buttons[index]}
					type="button"
					role="radio"
					aria-checked={checked}
					tabindex={checked || (!chosen && repo === choosable[0]) ? 0 : -1}
					onclick={() => (value = repo.id)}
					class={['flex h-11 w-full cursor-pointer items-center gap-2.5 rounded-xs px-2 text-left outline-offset-1 hover:bg-surface-2', checked && 'bg-surface-2']}
				>
					<span class="inline-flex text-text-2"><Icon icon={BookMarked} size={14} /></span>
					<span class="flex min-w-0 flex-1 flex-col gap-0.5">
						<span class="truncate font-mono text-caption text-text">{repo.name}</span>
						<span class="truncate font-mono text-label text-text-3">{repo.meta}</span>
					</span>
					{#if checked}<span class="inline-flex text-accent"><Icon icon={Check} size={14} /></span>{/if}
				</button>
			{/if}
		{/each}
	</div>
	<p class="flex gap-2 text-caption leading-cozy text-text-3">
		<span class="inline-flex shrink-0"><Icon icon={GitBranch} size={12} /></span>
		<span>{m.projects_repos_note()}</span>
	</p>
	{#snippet actions()}
		<Button variant="secondary" label={m.action_cancel()} onclick={() => (open = false)} />
		<Button
			label={m.projects_repos_import({ name: chosen?.name.split('/').pop() ?? '' }).trim()}
			iconEnd={ArrowRight}
			disabled={!chosen}
			onclick={() => {
				if (!chosen) return;
				open = false;
				onimport(chosen.id);
			}}
		/>
	{/snippet}
</ProjectDialog>
