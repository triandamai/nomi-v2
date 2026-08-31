<script lang="ts">
	import Card from '$lib/components/m3/Card.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	const STATUS_LABEL: Record<string, string> = {
		planning: 'Planning',
		building: 'Building…',
		ready: 'Ready',
	};
</script>

<h1 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">Projects</h1>
<p class="md-body-large mt-2" style="color: var(--md-sys-color-on-surface-variant)">
	Apps nomi has built or is building for you. Ask nomi to build something in chat to start a new one.
</p>

<div class="mt-6 space-y-3">
	{#each data.projects as project (project.id)}
		<a href="/projects/{project.id}" class="block">
			<Card variant="outlined" class="p-4">
				<div class="flex items-center justify-between">
					<div>
						<p class="md-title-medium" style="color: var(--md-sys-color-on-surface)">{project.name}</p>
						{#if project.description}
							<p class="md-body-medium mt-1" style="color: var(--md-sys-color-on-surface-variant)">{project.description}</p>
						{/if}
					</div>
					<span
						class="md-label-medium rounded-full px-3 py-1"
						style="background: var(--md-sys-color-secondary-container); color: var(--md-sys-color-on-secondary-container)"
					>
						{STATUS_LABEL[project.status] ?? project.status}
					</span>
				</div>
			</Card>
		</a>
	{/each}
	{#if data.projects.length === 0}
		<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">No projects yet.</p>
	{/if}
</div>
