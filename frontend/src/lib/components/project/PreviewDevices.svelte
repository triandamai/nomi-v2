<script lang="ts">
	import { Monitor, Smartphone } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '$lib/components/ds/foundations/Icon.svelte';

	// Desktop or phone: how wide the preview is shown. Two icon buttons, the chosen one filled.
	type Props = { device: 'desktop' | 'phone'; onchange: (device: 'desktop' | 'phone') => void };
	let { device, onchange }: Props = $props();
</script>

<div class="devices" role="group" aria-label={m.pj_preview_device()}>
	<button type="button" class="devices__btn" aria-pressed={device === 'desktop'} aria-label={m.device_desktop()} title={m.device_desktop()} onclick={() => onchange('desktop')}><Icon icon={Monitor} size={20} /></button>
	<button type="button" class="devices__btn" aria-pressed={device === 'phone'} aria-label={m.device_phone()} title={m.device_phone()} onclick={() => onchange('phone')}><Icon icon={Smartphone} size={20} /></button>
</div>

<style>
	.devices {
		display: inline-flex;
		flex: none;
		gap: 2px;
	}
	.devices__btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 40px;
		height: 40px;
		border: 0;
		border-radius: var(--md-sys-shape-corner-medium);
		background: none;
		color: var(--md-sys-color-on-surface-variant);
		cursor: pointer;
	}
	.devices__btn:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
	.devices__btn[aria-pressed='true'] {
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface);
	}
	.devices__btn:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: -2px;
	}
	@media (pointer: coarse) {
		.devices__btn {
			width: 44px;
			height: 44px;
		}
	}
</style>
