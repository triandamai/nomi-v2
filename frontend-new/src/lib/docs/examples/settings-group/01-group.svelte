<!-- A group: its rows, each with its control. -->
<script lang="ts">
	import SettingsGroup from '$lib/ds/settings/SettingsGroup.svelte';
	import SettingRow from '$lib/ds/settings/SettingRow.svelte';
	import Segmented from '$lib/ds/inputs/Segmented.svelte';
	import Toggle from '$lib/ds/inputs/Toggle.svelte';
</script>

<div class="w-full max-w-220">
	<SettingsGroup id="example-appearance" title="Appearance" description="How Nomi looks on this account.">
		<SettingRow title="Theme" description="System follows your device">
			{#snippet control()}<Segmented label="Theme" options={[{ value: 'system', label: 'System' }, { value: 'light', label: 'Light' }, { value: 'dark', label: 'Dark' }]} value="dark" />{/snippet}
		</SettingRow>
		<SettingRow title="Product news" description="Occasional updates from the Nomi team">
			{#snippet control()}<Toggle label="Product news" />{/snippet}
		</SettingRow>
	</SettingsGroup>
</div>
