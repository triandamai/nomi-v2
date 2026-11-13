<!-- Rows: a value in mono with a button, and one that can't be undone. -->
<script lang="ts">
	import SettingRow from '$lib/ds/settings/SettingRow.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
</script>

<div class="flex w-full max-w-220 flex-col border-t border-border">
	<SettingRow title="Email" description="ko••••••••@gmail.com" mono />
	<SettingRow title="Signed in on 2 devices" description="MacBook Air (this one) · iPhone 15 · 2h">
		{#snippet control()}<Button label="Sign out others" variant="ghost" />{/snippet}
	</SettingRow>
	<SettingRow title="Delete account" description="Removes chats, memory and projects." danger />
</div>
