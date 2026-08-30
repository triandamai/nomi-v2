<script lang="ts">
	import { enhance } from '$app/forms';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	const THEME_OPTIONS = [
		{ value: 'light', label: 'Light' },
		{ value: 'dark', label: 'Dark' },
		{ value: 'system', label: 'System' },
	];
</script>

<div class="h-full overflow-y-auto p-8">
	<div class="max-w-lg">
		<h1 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">Preferences</h1>

		{#if form?.error}
			<p class="md-body-medium mt-2" style="color: var(--md-sys-color-error)">{form.error}</p>
		{/if}

		<section class="mt-6">
			<h2 class="md-title-medium" style="color: var(--md-sys-color-on-surface)">Theme</h2>
			<p class="md-body-medium mt-1" style="color: var(--md-sys-color-on-surface-variant)">
				Choose how Nomi looks. "System" follows your device's setting.
			</p>
			<form method="POST" action="?/updateTheme" use:enhance class="mt-3 flex gap-2">
				{#each THEME_OPTIONS as option (option.value)}
					<button
						type="submit"
						name="theme"
						value={option.value}
						class="m3-theme-option"
						class:m3-theme-option--active={data.preferences.theme === option.value}
					>
						{option.label}
					</button>
				{/each}
			</form>
		</section>
	</div>
</div>

<style>
	.m3-theme-option {
		padding: 8px 20px;
		border-radius: var(--md-sys-shape-corner-full);
		border: 1px solid var(--md-sys-color-outline);
		background: transparent;
		color: var(--md-sys-color-on-surface);
		cursor: pointer;
		font-family: var(--md-sys-typescale-label-large-font);
		font-size: var(--md-sys-typescale-label-large-size);
	}
	.m3-theme-option--active {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
		border-color: transparent;
	}
</style>
