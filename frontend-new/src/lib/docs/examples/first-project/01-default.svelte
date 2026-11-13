<!-- Default: Koda says what to do, and each starter fills the composer. -->
<script lang="ts">
	import { CalendarCheck, FileText, Store } from '@lucide/svelte';
	import FirstProject from '$lib/ds/projects/FirstProject.svelte';

	const onpick = () => {};
</script>

<FirstProject
	starters={[
		{ icon: CalendarCheck, label: 'Booking page for my barbershop', meta: 'web app · database', onpick },
		{ icon: Store, label: 'Menu and pre-order page for a warung', meta: 'web app · database', onpick },
		{ icon: FileText, label: 'One-page portfolio with a contact form', meta: 'site · no database', onpick },
	]}
/>
