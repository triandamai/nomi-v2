<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { dayPart, greeting } from '$lib/adapters/time';
	import { clock } from '$lib/clock';
	import Greeting from '$lib/ds/chrome/Greeting.svelte';

	// App / Home → Primary Column (jNZ30): for now the greeting (Z7EVR) alone, centred in the column
	// the pen pads 32 / 56 / 48. The composer, suggestions and Today strip come with the Chat phase.
	let { data } = $props();

	const time = clock();
	const title = $derived(greeting(data.person.firstName, dayPart(time.now, time.timeZone)));
</script>

<svelte:head><title>{m.app_name()}</title></svelte:head>

<!-- The shell pads the column 32 top and bottom; the pen's bottom is 48. -->
<div class="flex min-h-full flex-col justify-center lg:pb-4">
	<Greeting {title} intro={m.home_intro()} />
</div>
