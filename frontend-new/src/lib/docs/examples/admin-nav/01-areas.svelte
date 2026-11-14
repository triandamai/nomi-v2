<!-- The console's sidebar: only the areas the person may open, the open one current. -->
<script lang="ts">
	import { Activity, ChartColumn, CreditCard, LayoutDashboard, Users } from '@lucide/svelte';
	import AdminNav from '$lib/ds/admin/AdminNav.svelte';
</script>

<div class="h-130">
	<AdminNav
		label="Admin"
		mark="admin"
		homeHref="#admin"
		backHref="#app"
		backLabel="Back to Nomi"
		active="#users"
		groups={[
			{
				label: 'Dashboard',
				items: [
					{ href: '#overview', icon: LayoutDashboard, label: 'Overview' },
					{ href: '#live', icon: Activity, label: 'Live agents' },
					{ href: '#usage', icon: ChartColumn, label: 'Usage' },
				],
			},
			{
				label: 'People',
				items: [
					{ href: '#users', icon: Users, label: 'Users & access' },
					{ href: '#plans', icon: CreditCard, label: 'Plans' },
				],
			},
		]}
	/>
</div>
