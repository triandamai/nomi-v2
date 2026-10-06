<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Button from '$lib/components/m3/Button.svelte';

	// Upgrading isn't open yet: this says what Pro will bring, and that it's on its way.
	let { open = $bindable(false) }: { open?: boolean } = $props();

	const PERKS = $derived([m.pro_perk_tokens(), m.pro_perk_models(), m.pro_perk_priority()]);
</script>

<BottomSheet bind:open>
	<div class="pro">
		<div class="pro__art" aria-hidden="true">
			<AgentShape agent="nomi" face size={64} working />
			<span class="pro__badge">{m.pro_coming_soon()}</span>
		</div>
		<h2 class="pro__title">{m.pro_title()}</h2>
		<p class="pro__lede">{m.pro_lede()}</p>
		<ul class="pro__perks">
			{#each PERKS as perk (perk)}
				<li>
					<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m5 12 5 5 9-10" /></svg>
					{perk}
				</li>
			{/each}
		</ul>
		<div class="pro__actions">
			<Button variant="filled" size="m" onclick={() => (open = false)}>{m.pro_got_it()}</Button>
		</div>
	</div>
</BottomSheet>

<style>
	.pro {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.pro__art {
		display: flex;
		align-items: center;
		gap: 12px;
	}
	.pro__badge {
		padding: 4px 12px;
		border-radius: 999px;
		background: var(--md-sys-color-tertiary-container);
		color: var(--md-sys-color-on-tertiary-container);
		font-family: var(--md-ref-typeface-plain);
		font-size: 0.75rem;
		font-weight: 700;
		letter-spacing: 0.04em;
		text-transform: uppercase;
	}
	.pro__title {
		margin: 4px 0 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.625rem;
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}
	.pro__lede {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.pro__perks {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin: 4px 0 0;
		padding: 0;
		list-style: none;
		color: var(--md-sys-color-on-surface);
	}
	.pro__perks li {
		display: flex;
		align-items: center;
		gap: 10px;
	}
	.pro__perks svg {
		flex: none;
		color: var(--md-sys-color-primary);
	}
	.pro__actions {
		display: flex;
		justify-content: flex-end;
		padding-top: 8px;
	}
</style>
