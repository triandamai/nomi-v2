<script lang="ts">
	import { ArrowRight, ArrowUpRight, CircleAlert, CircleCheck, Eye, EyeOff, X } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import IconButton from '../actions/IconButton.svelte';
	import Toggle from '../inputs/Toggle.svelte';
	import ProviderTile from './ProviderTile.svelte';

	// App / Settings / Models & providers / Connect (voUxB) → Connect Pane: connecting one provider.
	// The header (its tile, "Connect Anthropic", who bills it in mono, close); the key (or the local
	// address) with Get a key, shown or hidden, and what checking it found; once it works, its
	// models to show in the picker; the foot with how the key is kept, Cancel and the one action.
	export type PaneModel = { id: string; label: string; shown: boolean };
	type Props = {
		title: string;
		name: string;
		billing: string;
		closeLabel: string;
		field: { label: string; placeholder?: string; secret: boolean; getKey?: { href: string; label: string } };
		value?: string;
		check?: { kind: 'ok' | 'failed'; text: string };
		models?: { label: string; note: string; items: PaneModel[] };
		foot: string;
		cancel: string;
		action: string;
		busy?: boolean;
		disabled?: boolean;
		titleId?: string;
		onclose?: () => void;
		onaction?: () => void;
		onshow?: (id: string, shown: boolean) => void;
	};
	let { title, name, billing, closeLabel, field, value = $bindable(''), check, models, foot, cancel, action, busy = false, disabled = false, titleId, onclose, onaction, onshow }: Props = $props();

	const id = $props.id();
	let reveal = $state(false);
</script>

<header class="flex shrink-0 items-center gap-3 border-b border-border px-5 pt-4 pb-3.75">
	<ProviderTile {name} />
	<div class="flex min-w-0 flex-1 flex-col gap-0.5">
		<h2 id={titleId} class="truncate text-body font-medium text-text">{title}</h2>
		<p class="truncate font-mono text-label text-text-3">{billing}</p>
	</div>
	<IconButton icon={X} label={closeLabel} variant="ghost" onclick={onclose} />
</header>
<form
	class="flex min-h-0 flex-1 flex-col"
	onsubmit={(event) => {
		event.preventDefault();
		if (!busy && !disabled) onaction?.();
	}}
>
	<div class="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto p-5">
		<div class="flex flex-col gap-1.5">
			<div class="flex items-center justify-between gap-2">
				<label for="{id}-key" class="text-ui font-medium text-text">{field.label}</label>
				{#if field.getKey}
					<a href={field.getKey.href} target="_blank" rel="noopener noreferrer" class="inline-flex items-center gap-1 rounded-xs text-caption text-text-2 outline-offset-2 hover:text-text">
						{field.getKey.label}<Icon icon={ArrowUpRight} size={12} />
					</a>
				{/if}
			</div>
			<div class={['flex h-9 items-center gap-2 rounded-sm border bg-bg px-2.25 focus-within:shadow-focus', check?.kind === 'failed' ? 'border-danger' : 'border-border-strong focus-within:border-accent']}>
				<input
					id="{id}-key"
					bind:value
					type={field.secret && !reveal ? 'password' : 'text'}
					placeholder={field.placeholder}
					autocomplete="off"
					spellcheck="false"
					aria-invalid={check?.kind === 'failed' ? 'true' : undefined}
					aria-describedby={check ? `${id}-check` : undefined}
					class="min-w-0 flex-1 bg-transparent font-mono text-ui text-text caret-accent outline-none placeholder:font-sans placeholder:text-text-3"
				/>
				{#if field.secret}
					<button type="button" aria-label={reveal ? m.models_key_hide() : m.models_key_show()} aria-pressed={reveal} onclick={() => (reveal = !reveal)} class="inline-flex cursor-pointer rounded-xs text-text-3 outline-offset-2 hover:text-text">
						<Icon icon={reveal ? EyeOff : Eye} size={14} />
					</button>
				{/if}
			</div>
			{#if check}
				<p id="{id}-check" role={check.kind === 'failed' ? 'alert' : 'status'} class={['flex items-center gap-1.5 text-caption', check.kind === 'ok' ? 'text-accent' : 'text-danger']}>
					<Icon icon={check.kind === 'ok' ? CircleCheck : CircleAlert} size={14} />{check.text}
				</p>
			{/if}
		</div>
		{#if models}
			<div class="flex flex-col gap-2">
				<p class="flex items-center justify-between gap-2">
					<span class="text-ui font-medium text-text">{models.label}</span>
					<span class="font-mono text-label text-text-3">{models.note}</span>
				</p>
				<ul class="flex flex-col rounded-sm outline -outline-offset-1 outline-border">
					{#each models.items as model (model.id)}
						<li class="flex items-center gap-2.5 border-b border-border px-3 pt-2 pb-1.75 last:border-b-0 last:pb-2">
							<span class="min-w-0 flex-1 truncate font-mono text-ui text-text">{model.label}</span>
							<Toggle label={model.label} checked={model.shown} onchange={(shown) => onshow?.(model.id, shown)} />
						</li>
					{/each}
				</ul>
			</div>
		{/if}
	</div>
	<footer class="flex shrink-0 items-center gap-2 border-t border-border px-5 pt-2.75 pb-3">
		<p class="min-w-0 flex-1 font-mono text-label text-text-3">{foot}</p>
		<Button label={cancel} variant="ghost" onclick={onclose} />
		<Button label={action} variant="primary" type="submit" iconEnd={ArrowRight} loading={busy} {disabled} />
	</footer>
</form>
