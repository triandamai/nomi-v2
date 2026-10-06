<script lang="ts">
	import type { Snippet } from 'svelte';
	import AdminSidebar from '$lib/components/AdminSidebar.svelte';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import IconMenu from '$lib/components/icons/IconMenu.svelte';
	import type { LayoutData } from './$types';

	let { data, children }: { data: LayoutData; children: Snippet } = $props();

	let mobileNavOpen = $state(false);

	$effect(() => {
		document.documentElement.dataset.theme = data.preferences.theme;
		document.documentElement.dataset.color = data.preferences.accent_color;
	});
</script>

<div class="app-shell flex" style="background: var(--md-sys-color-surface)">
	<AdminSidebar canManageSystemConfig={data.canManageSystemConfig} canViewUsers={data.canViewUsers} bind:mobileOpen={mobileNavOpen} />
	<div class="flex flex-1 flex-col overflow-hidden">
		<header class="flex items-center gap-2 px-2 py-2 md:hidden">
			<IconButton onclick={() => (mobileNavOpen = true)} aria-label="Open menu"><IconMenu /></IconButton>
			<a href="/admin" class="mobile-brand">
				<AgentShape size={28} face />
				<span class="mobile-brand__word">nomi</span>
				<span class="mobile-brand__tag">Admin</span>
			</a>
		</header>
		<main class="admin-main">
			<div class="admin-main__inner">
				{@render children()}
			</div>
		</main>
	</div>
</div>

<style>
	.mobile-brand {
		display: flex;
		align-items: center;
		gap: 8px;
		color: var(--md-sys-color-on-surface);
		text-decoration: none;
	}
	.mobile-brand__word {
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.5rem;
		font-weight: 800;
		letter-spacing: -0.045em;
	}
	.mobile-brand__tag {
		padding: 2px 8px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-tertiary-container);
		color: var(--md-sys-color-on-tertiary-container);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.625rem;
		letter-spacing: 0.08em;
		text-transform: uppercase;
	}
	.admin-main {
		flex: 1;
		overflow-y: auto;
		padding: 32px clamp(16px, 4vw, 56px) 56px;
	}
	.admin-main__inner {
		display: flex;
		flex-direction: column;
		gap: 28px;
		max-width: 1180px;
		margin: 0 auto;
	}
</style>
