<!-- Header: the workspace, its people, Invite, and New. -->
<script lang="ts">
	import PhoneProjectsHeader from '$lib/ds/projects/PhoneProjectsHeader.svelte';
</script>

<div class="w-97.5 max-w-full">
	<PhoneProjectsHeader
		title="Projects"
		workspace="Trian's space"
		members={3}
		labels={{ workspace: 'Workspace: Trian’s space', members: 'People', invite: 'Invite people', new: 'New' }}
		onworkspace={() => {}}
		oninvite={() => {}}
		onnew={() => {}}
	/>
</div>
