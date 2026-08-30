<script lang="ts">
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	// No OAuth integration exists in this app yet — these are honest "not connected"
	// placeholders, not a functioning connect flow. See the connected-identities design note.
	const PROVIDERS = [
		{ id: 'google', label: 'Google' },
		{ id: 'facebook', label: 'Facebook' },
		{ id: 'github', label: 'GitHub' },
	];
</script>

<div class="h-full overflow-y-auto p-8">
	<div class="max-w-lg">
		<h1 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">Account settings</h1>

		<section class="mt-6">
			<h2 class="md-title-medium" style="color: var(--md-sys-color-on-surface)">Email</h2>
			<p class="md-body-large mt-1" style="color: var(--md-sys-color-on-surface-variant)">{data.profile?.email ?? '—'}</p>
		</section>

		<section class="mt-8">
			<h2 class="md-title-medium" style="color: var(--md-sys-color-on-surface)">Connected accounts</h2>
			<p class="md-body-medium mt-1" style="color: var(--md-sys-color-on-surface-variant)">
				Sign in with another provider. Not available yet.
			</p>
			<div class="mt-3 space-y-2">
				{#each PROVIDERS as provider (provider.id)}
					<div class="m3-connected-row">
						<span class="md-body-large" style="color: var(--md-sys-color-on-surface)">{provider.label}</span>
						<button type="button" class="m3-connect-btn" disabled>Coming soon</button>
					</div>
				{/each}
			</div>
		</section>
	</div>
</div>

<style>
	.m3-connected-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 16px;
		border-radius: var(--md-sys-shape-corner-small);
		border: 1px solid var(--md-sys-color-outline-variant);
	}
	.m3-connect-btn {
		padding: 6px 16px;
		border-radius: var(--md-sys-shape-corner-full);
		border: 1px solid var(--md-sys-color-outline-variant);
		background: transparent;
		color: var(--md-sys-color-outline);
		cursor: not-allowed;
		font-family: var(--md-sys-typescale-label-large-font);
		font-size: var(--md-sys-typescale-label-large-size);
	}
</style>
