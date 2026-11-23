<!-- The crew as rows: each member's mark, what they do, and how they are now. -->
<script lang="ts">
	import PhoneSettingsGroup from '$lib/ds/settings/PhoneSettingsGroup.svelte';
</script>

<div class="w-97.5 max-w-full px-4">
	<PhoneSettingsGroup
		label="Specialists"
		note="3"
		rows={[
			{ crew: { agent: 'koda', initial: 'K' }, label: 'Koda', sub: 'Builds and edits project files', meta: 'working', tone: 'accent', href: '#' },
			{ crew: { agent: 'kala', initial: 'K' }, label: 'Kala', sub: 'Reminds you on time', meta: 'waiting on you', tone: 'warning', href: '#' },
			{ crew: { initial: 'R' }, label: 'riset', sub: 'Finds and checks sources', meta: 'on', href: '#' },
		]}
	/>
</div>
