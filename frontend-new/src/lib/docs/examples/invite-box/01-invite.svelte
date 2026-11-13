<!-- Inviting people as members. -->
<script lang="ts">
	import InviteBox from '$lib/ds/settings/InviteBox.svelte';
</script>

<div class="w-full max-w-220">
	<InviteBox
		labels={{ field: 'Email addresses', placeholder: 'Add more, separated by commas', remove: 'Remove {email}', role: 'Role', note: 'they won’t see your chats, memory or money', send: (count) => (count === 1 ? 'Send 1 invite' : `Send ${count} invites`) }}
		roles={[
			{ value: 'member', title: 'Member', body: 'Works on projects and their chats' },
			{ value: 'admin', title: 'Admin', body: 'Also invites, removes people and renames' },
		]}
	/>
</div>
