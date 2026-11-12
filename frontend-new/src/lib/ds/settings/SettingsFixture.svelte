<script lang="ts">
	import Toggle from '../inputs/Toggle.svelte';
	import SettingsGroup from './SettingsGroup.svelte';
	import SettingRow from './SettingRow.svelte';

	// For the tests: a group with one row and its switch, 880 wide as in App / Settings / General.
</script>

<div class="w-220">
	<SettingsGroup id="appearance" title="Appearance">
		<SettingRow title="Reduce motion" description="Fewer animations in chat and deploys">
			{#snippet control()}<Toggle label="Reduce motion" />{/snippet}
		</SettingRow>
	</SettingsGroup>
</div>
