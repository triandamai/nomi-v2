<script lang="ts">
	import { CloudOff } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import PhoneButton from '../mobile/PhoneButton.svelte';

	// Flow / Mobile — You & Settings, step 23 (couldn't load) → Error: a page that couldn't load says
	// what happened and offers Try again, under its own title, so the back bar to You stays: the
	// icon at 26, what couldn't load at 17, why (and that nothing is lost) at 14, then Try again.
	type Props = { title: string; body: string; retry: string; busy?: boolean; onretry: () => void };
	let { title, body, retry, busy = false, onretry }: Props = $props();
</script>

<div role="alert" class="flex flex-col items-start gap-2 px-5 pt-2">
	<span class="inline-flex text-text-3"><Icon icon={CloudOff} size={26} /></span>
	<p class="text-phone-title font-semibold text-text">{title}</p>
	<p class="text-phone-sm leading-firm text-text-2">{body}</p>
	<div class="flex w-full flex-col pt-2"><PhoneButton label={retry} variant="secondary" disabled={busy} onclick={onretry} /></div>
</div>
