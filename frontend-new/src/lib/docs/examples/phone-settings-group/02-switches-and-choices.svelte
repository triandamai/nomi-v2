<!-- Rows that hold a switch, and a sheet's choices with the chosen one ticked. -->
<script lang="ts">
	import { CornerDownLeft, Trash2 } from '@lucide/svelte';
	import PhoneSettingsGroup from '$lib/ds/settings/PhoneSettingsGroup.svelte';

	let enterSends = $state(true);
	let language = $state('id');
</script>

<div class="flex w-97.5 max-w-full flex-col gap-5">
	<PhoneSettingsGroup label="Chat" rows={[{ icon: CornerDownLeft, label: 'Enter sends', sub: enterSends ? 'Shift+Enter adds a new line' : 'Enter adds a new line', toggle: { checked: enterSends, onchange: (on) => (enterSends = on) } }]} />
	<PhoneSettingsGroup
		rows={[
			{ label: 'English', chosen: language === 'en', onclick: () => (language = 'en') },
			{ label: 'Bahasa Indonesia', chosen: language === 'id', onclick: () => (language = 'id') },
		]}
	/>
	<PhoneSettingsGroup rows={[{ icon: Trash2, label: 'Delete account', danger: true, onclick: () => {} }]} />
</div>
