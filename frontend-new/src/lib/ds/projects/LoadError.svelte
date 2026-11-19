<script lang="ts">
	import { CloudOff, RotateCw } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import PhoneProjectEmpty from './PhoneProjectEmpty.svelte';

	// Design System / Projects → Project / Load Error (vkXCA): the list couldn't load. On the danger
	// tint (14 / 12 in, no stroke): a 16px cloud-off in danger, that the apps themselves are fine,
	// the provider's own reason in the same line, and Retry.
	type Props = {
		/** What failed and when, as the server said it: "503 from api.nomi.app · 09:14". */
		reason: string;
		onretry: () => void;
		/** Flow / Mobile Projects — Loading, offline & errors, step 2 (UULGe): PhoneProjectEmpty in danger, with the reason and Retry. */
		phone?: boolean;
	};
	let { reason, onretry, phone = false }: Props = $props();
</script>

{#if phone}
	<PhoneProjectEmpty tone="danger" icon={CloudOff} title={m.projects_phone_load_error_title()} body={m.projects_phone_load_error_body()} code={reason} action={{ label: m.action_retry(), icon: RotateCw, onclick: onretry }} />
{:else}
<div role="alert" class="flex w-full items-center gap-3 rounded-md bg-danger-bg px-3.5 py-3">
	<span class="inline-flex text-danger"><Icon icon={CloudOff} size={16} /></span>
	<div class="flex min-w-0 flex-1 flex-col gap-0.5">
		<p class="text-ui font-medium text-text">{m.projects_load_error_title()}</p>
		<p class="text-caption text-text-2">{m.projects_load_error_body()} {reason}</p>
	</div>
	<Button variant="secondary" label={m.action_retry()} iconEnd={RotateCw} onclick={onretry} />
</div>
{/if}
