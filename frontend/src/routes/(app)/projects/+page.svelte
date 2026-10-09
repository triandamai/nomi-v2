<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { enhance } from '$app/forms';
	import Card from '$lib/components/m3/Card.svelte';
	import IconPlus from '$lib/components/icons/IconPlus.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	const STATUS_LABEL: Record<string, string> = {
		planning: m.projects_planning(),
		building: m.projects_building(),
		ready: m.projects_ready(),
	};
</script>

<div class="h-full overflow-y-auto px-4 py-8 md:px-10">
	<h1 class="md-display-small" style="color: var(--md-sys-color-on-surface)">{m.projects_title()}</h1>
	<p class="md-body-large mt-2" style="color: var(--md-sys-color-on-surface-variant)">
		{m.projects_lede()}
	</p>

	<div class="mt-6 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
		<form method="POST" action="?/createProject" use:enhance>
			<button type="submit" class="m3-add-project-tile">
				<IconPlus size={28} />
				<span class="md-title-medium">{m.projects_add()}</span>
			</button>
		</form>

		{#each data.projects as project (project.id)}
			<a href="/projects/session/{project.session_id}" class="block" data-sveltekit-reload>
				<Card variant="outlined" class="h-full p-4">
					<div class="flex items-start justify-between gap-2">
						<div class="min-w-0">
							<p class="md-title-medium truncate" style="color: var(--md-sys-color-on-surface)">{project.name}</p>
							{#if project.description}
								<p class="md-body-medium mt-1" style="color: var(--md-sys-color-on-surface-variant)">{project.description}</p>
							{/if}
						</div>
						<span
							class="md-label-medium shrink-0 rounded-full px-3 py-1"
							style="background: var(--md-sys-color-secondary-container); color: var(--md-sys-color-on-secondary-container)"
						>
							{STATUS_LABEL[project.status] ?? project.status}
						</span>
					</div>
				</Card>
			</a>
		{/each}
	</div>
</div>

<style>
	.m3-add-project-tile {
		display: flex;
		height: 100%;
		width: 100%;
		min-height: 96px;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 8px;
		border-radius: var(--md-sys-shape-corner-medium);
		border: 1px dashed var(--md-sys-color-outline);
		background: none;
		color: var(--md-sys-color-on-surface-variant);
		cursor: pointer;
	}
	.m3-add-project-tile:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
</style>
