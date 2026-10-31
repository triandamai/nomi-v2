<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import Link from '../actions/Link.svelte';
	import Meter from '../status/Meter.svelte';

	/** The plan and how much of it is used this month, at the foot of the drawer. */
	type Props = { plan: string; used: number; caption: string; href: string };
	let { plan, used, caption, href }: Props = $props();
</script>

<div class="card">
	<Meter label={plan} value={used} {caption} roomy />
	<Link {href} label={m.nav_plan_usage()} />
</div>

<style>
	.card {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 12px;
		padding: 16px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-low);
	}
</style>
