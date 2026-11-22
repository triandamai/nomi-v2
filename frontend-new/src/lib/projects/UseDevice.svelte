<script lang="ts">
	import { Laptop } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Button from '$lib/ds/actions/Button.svelte';

	// The preview's way out when the sandbox is off or can't be used: run the app on this device
	// instead (WebContainer, in this tab). A full page load, so the server can isolate the page.
	let { href, onclick }: { href: string; onclick?: () => void } = $props();
</script>

<div data-part="use-device" class="flex flex-col items-start gap-1.5">
	<Button variant="secondary" iconStart={Laptop} label={m.device_use()} {href} {onclick} data-sveltekit-reload />
	<p class="text-caption text-text-3">{m.device_use_note()}</p>
</div>
