<script lang="ts">
	import { Check, Copy, Hourglass, Link2, X } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import Segmented from '../inputs/Segmented.svelte';
	import Toggle from '../inputs/Toggle.svelte';
	import Chip from '../inputs/Chip.svelte';
	import { createCopier } from '../actions/copier.svelte';
	import type { CrewId } from '../agents/crew';

	// Chat responses → Share Dialog, its content: this reply or the whole chat, a preview of what
	// others will see, amounts and names hidden by default, who can open it, when it expires, and
	// the link. ShareDialog shows it as a modal.
	export type ShareSettings = { scope: 'reply' | 'chat'; mask: boolean; anyone: boolean };
	type Props = {
		/** Ids the heading, for the dialog that wraps it. */
		titleId?: string;
		/** What the link shows, already masked when mask is on. */
		preview: { agent: CrewId; name: string; time: string; excerpt: string };
		chatMessages: number;
		url: string;
		/** e.g. "in 7 days". */
		expiry: string;
		settings?: ShareSettings;
		onchange?: (settings: ShareSettings) => void;
		onexpiry?: () => void;
		oncopy?: () => void;
		onclose: () => void;
	};
	let { titleId, preview, chatMessages, url, expiry, settings, onchange, onexpiry, oncopy, onclose }: Props = $props();

	const initial = () => settings ?? { scope: 'reply' as const, mask: true, anyone: true };
	let current = $state<ShareSettings>(initial());
	const copier = createCopier();

	function update(patch: Partial<ShareSettings>) {
		current = { ...current, ...patch };
		onchange?.({ ...current });
	}

	async function copy() {
		await copier.copy(url);
		if (copier.copied) oncopy?.();
	}
</script>

<div class="flex flex-col gap-4 p-5">
	<div class="flex items-center gap-2">
		<h2 id={titleId} class="min-w-0 flex-1 text-body font-medium text-text">{m.share_title()}</h2>
		<button type="button" aria-label={m.action_close()} class="inline-flex size-6 cursor-pointer items-center justify-center rounded-xs text-text-3 outline-offset-1 hover:text-text pointer-coarse:size-11" onclick={onclose}>
			<Icon icon={X} size={14} />
		</button>
	</div>
	<Segmented
		surface="bg"
		label={m.share_scope()}
		value={current.scope}
		options={[
			{ value: 'reply', label: m.share_reply() },
			{ value: 'chat', label: m.share_chat(), count: m.share_msgs({ count: String(chatMessages) }) },
		]}
		onchange={(scope) => update({ scope: scope as ShareSettings['scope'] })}
	/>
	<figure class="flex flex-col gap-1.5 rounded-sm border border-border bg-bg p-3">
		<figcaption class="flex items-center gap-1.5">
			<Mascot agent={preview.agent} size={20} />
			<span class="text-caption font-medium text-text">{preview.name}</span>
			<span class="font-mono text-label text-text-3">{preview.time}</span>
		</figcaption>
		<p class="text-caption leading-normal text-text-2">{preview.excerpt}</p>
	</figure>
	<div class="flex flex-col gap-3">
		<div class="flex items-center gap-3">
			<span class="flex min-w-0 flex-1 flex-col gap-0.5">
				<span class="text-ui text-text">{m.share_mask()}</span>
				<span class="text-caption text-text-3">{m.share_mask_help()}</span>
			</span>
			<Toggle label={m.share_mask()} checked={current.mask} onchange={(mask) => update({ mask })} />
		</div>
		<div class="flex items-center gap-3">
			<span class="flex min-w-0 flex-1 flex-col gap-0.5">
				<span class="text-ui text-text">{m.share_anyone()}</span>
				<span class="text-caption text-text-3">{m.share_anyone_help()}</span>
			</span>
			<Toggle label={m.share_anyone()} checked={current.anyone} onchange={(anyone) => update({ anyone })} />
		</div>
		<div class="flex items-center gap-3">
			<span class="min-w-0 flex-1 text-ui text-text">{m.share_expires()}</span>
			<Chip kind="option" icon={Hourglass} label={expiry} onclick={onexpiry} />
		</div>
	</div>
	<div class="flex flex-wrap items-center gap-2">
		<span class="flex h-7.5 min-w-0 flex-1 basis-48 items-center gap-2 rounded-sm border border-border-strong bg-bg px-2.5">
			<span class="inline-flex shrink-0 text-text-3"><Icon icon={Link2} size={12} /></span>
			<span class="min-w-0 truncate font-mono text-caption text-text" title={url}>{url.replace(/^https?:\/\//, '')}</span>
		</span>
		<span class="ml-auto"><Button label={m.share_copy()} iconEnd={copier.copied ? Check : Copy} onclick={copy} /></span>
	</div>
</div>
<p class="border-t border-border px-5 py-2.5 text-caption text-text-3">{m.share_revoke()}</p>
