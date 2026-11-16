<!-- Profile: what the crew keeps about the person, with its size and when it changed. -->
<script lang="ts">
	import MemoryProfile from '$lib/ds/settings/MemoryProfile.svelte';

	const labels = {
		title: 'Profile',
		section: 'You',
		edit: 'Edit the profile',
		clear: 'Clear the profile',
		save: 'Save',
		cancel: 'Cancel',
		field: 'Your profile',
		empty: 'Nomi writes this about once a day from what it learns. You can write it yourself.',
		tooLong: 'That’s longer than the profile budget. Shorten it, or raise the budget.',
	};
</script>

<div class="w-full max-w-205">
	<MemoryProfile
		{labels}
		initials="MP"
		text="Mario Prasetyo, Jakarta. Founder and designer at Alif, building Nomi. Writes in English and Bahasa; likes short, direct answers."
		updated="updated 21 Oct"
		count="312 / 1,375 tok"
		budget={1375}
	/>
</div>
