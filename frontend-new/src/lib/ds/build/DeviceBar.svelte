<script lang="ts">
	import { ChevronDown, Send, Smartphone } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Build → Phone preview: the device the preview is shown at, and opening the app on your own
	// phone (a QR or a link sent to you).
	type Props = { device: string; ondevice: () => void; onphone?: () => void };
	let { device, ondevice, onphone }: Props = $props();
</script>

<div class="flex flex-wrap items-center gap-2">
	<button type="button" aria-haspopup="menu" class="inline-flex h-7 cursor-pointer items-center gap-1.5 rounded-sm border border-border-strong px-2.5 text-caption text-text outline-offset-0 hover:bg-surface-2 focus-visible:outline-3 pointer-coarse:min-h-11" onclick={ondevice}>
		<Icon icon={Smartphone} size={14} />{device}<span class="inline-flex text-text-3"><Icon icon={ChevronDown} size={12} /></span>
	</button>
	{#if onphone}
		<button type="button" class="inline-flex h-7 cursor-pointer items-center gap-1.5 rounded-sm px-2.5 text-caption text-text-2 outline-offset-0 hover:bg-surface-2 hover:text-text focus-visible:outline-3 pointer-coarse:min-h-11" onclick={onphone}>
			<Icon icon={Send} size={14} />{m.device_open_phone()}
		</button>
	{/if}
</div>
