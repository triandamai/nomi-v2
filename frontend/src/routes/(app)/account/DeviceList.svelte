<script lang="ts">
	import { enhance } from '$app/forms';
	import Button from '$lib/components/m3/Button.svelte';
	import { timeAgo } from '$lib/i18n';
	import { m } from '$lib/paraglide/messages';
	import { onDevicePresence } from '$lib/userActivity';
	import type { Device } from './+page.server';

	let { devices }: { devices: Device[] } = $props();

	// Presence changes arrive live (MQTT, through the activity feed) on top of what the page loaded.
	let live = $state<Record<string, Pick<Device, 'presence' | 'last_seen_at'>>>({});
	$effect(() => onDevicePresence((event) => (live[event.device_id] = { presence: event.presence, last_seen_at: event.last_seen_at })));
	const shown = $derived(devices.map((device) => ({ ...device, ...live[device.id.toLowerCase()] })));

	function label(device: Device): string {
		return device.name ?? device.model ?? device.platform;
	}

	/** Brand and model, then OS and app version: "Google · Pixel 9 · Android 15 · 1.4.0". */
	function details(device: Device): string {
		const os = [device.os_name, device.os_version].filter(Boolean).join(' ');
		const hardware = device.model && device.model !== device.name ? device.model : null;
		return [device.brand, hardware, os || null, device.app_version].filter(Boolean).join(' · ');
	}

	function status(device: Device): string {
		if (device.presence === 'online') return m.account_device_online();
		if (device.presence === 'away') return m.account_device_away();
		return m.account_device_seen({ when: timeAgo(device.last_seen_at) });
	}
</script>

{#if shown.length === 0}
	<p class="empty">{m.account_devices_empty()}</p>
{:else}
	<ul class="rows">
		{#each shown as device (device.id)}
			<li class="row">
				<span class="row__icon" aria-hidden="true">
					{#if device.platform === 'android' || device.platform === 'ios'}
						<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="6" y="2" width="12" height="20" rx="2.5" /><path d="M11 18h2" /></svg>
					{:else if device.platform === 'web'}
						<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9" /><path d="M3 12h18M12 3a14 14 0 0 1 0 18M12 3a14 14 0 0 0 0 18" /></svg>
					{:else}
						<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="4" width="18" height="12" rx="2" /><path d="M2 20h20" /></svg>
					{/if}
				</span>
				<span class="row__text">
					<span class="row__headline">
						{label(device)}
						{#if device.current}<span class="this">{m.account_device_this()}</span>{/if}
					</span>
					<span class="row__supporting">{details(device)}</span>
					<span class="presence presence--{device.presence}">
						<span class="presence__dot" aria-hidden="true"></span>{status(device)}
					</span>
				</span>
				{#if !device.current}
					<form method="POST" action="?/signOutDevice" use:enhance>
						<input type="hidden" name="id" value={device.id} />
						<input type="hidden" name="name" value={label(device)} />
						<Button variant="text" size="s" type="submit">{m.account_device_sign_out()}</Button>
					</form>
				{/if}
			</li>
		{/each}
	</ul>
{/if}

<style>
	.rows {
		display: flex;
		flex-direction: column;
		gap: 2px;
		margin: 0;
		padding: 0;
		overflow: hidden;
		border-radius: var(--md-sys-shape-corner-extra-large);
		list-style: none;
	}
	.row {
		display: flex;
		align-items: center;
		gap: 12px;
		min-height: 64px;
		padding: 10px 12px 10px 14px;
		background: var(--md-sys-color-surface-container-lowest);
	}
	.row__icon {
		display: flex;
		flex-shrink: 0;
		align-items: center;
		justify-content: center;
		width: 40px;
		height: 40px;
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-primary);
	}
	.row__text {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.row__headline {
		display: flex;
		flex-wrap: wrap;
		align-items: baseline;
		gap: 8px;
		color: var(--md-sys-color-on-surface);
		font-size: 0.9375rem;
		font-weight: 600;
	}
	.this {
		color: var(--md-sys-color-primary);
		font-size: 0.75rem;
		font-weight: 600;
	}
	.row__supporting {
		overflow-wrap: anywhere;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.presence {
		display: flex;
		align-items: center;
		gap: 6px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.presence__dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--md-sys-color-outline);
	}
	.presence--online .presence__dot {
		background: var(--md-sys-color-primary);
	}
	.presence--away .presence__dot {
		background: transparent;
		box-shadow: inset 0 0 0 2px var(--md-sys-color-primary);
	}
	.empty {
		margin: 0 8px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
</style>
