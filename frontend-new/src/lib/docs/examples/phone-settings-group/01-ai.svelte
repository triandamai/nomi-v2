<!-- A group: values only where the app already knows them. -->
<script lang="ts">
	import { Brain, Cpu, Users } from '@lucide/svelte';
	import PhoneSettingsGroup from '$lib/ds/settings/PhoneSettingsGroup.svelte';
</script>

<div class="w-97.5 max-w-full">
	<PhoneSettingsGroup
		label="AI"
		rows={[
			{ href: '#', icon: Users, label: 'Crew', meta: '11' },
			{ href: '#', icon: Cpu, label: 'Models & providers' },
			{ href: '#', icon: Brain, label: 'Memory & context' },
		]}
	/>
</div>
