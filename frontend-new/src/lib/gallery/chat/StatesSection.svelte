<script lang="ts">
	import type { ComponentProps } from 'svelte';
	import { ArrowLeftRight, ArrowUpRight, Cpu, Hourglass, KeyRound, Link2, Paperclip, Pencil, Plug, RotateCcw, Sparkles, Zap } from '@lucide/svelte';
	import MessageUser from '$lib/ds/conversation/MessageUser.svelte';
	import ErrorCallout from '$lib/ds/conversation/ErrorCallout.svelte';
	import ReplyText from '$lib/ds/chat/ReplyText.svelte';
	import Banner from '$lib/ds/chat/Banner.svelte';
	import SectionLabel from '$lib/ds/status/SectionLabel.svelte';
	import GallerySection from '../GallerySection.svelte';
	import GalleryBoard from '../GalleryBoard.svelte';

	type Case = { label: string; prose: string; callout: ComponentProps<typeof ErrorCallout> };
	const noop = () => {};
	const reason = (value: string, provider = 'api.deepseek.com') => [
		{ key: 'reason', value, strong: true },
		{ key: 'provider', value: provider },
	];

	const keyAndBilling: Case[] = [
		{
			label: '401 · Key rejected',
			prose: "DeepSeek didn't accept your API key, so nothing was sent. It may have been revoked or mistyped.",
			callout: {
				title: 'Key rejected',
				code: '401 authentication_error',
				requestId: 'req_2b7f19aa',
				details: reason('Authentication Fails, your api key ****a91f is invalid'),
				actions: [{ label: 'Update key', icon: KeyRound, onclick: noop }, { label: 'Use Nomi credits', onclick: noop }],
			},
		},
		{
			label: '402 · Out of credits',
			prose: 'Your DeepSeek account is out of credits for this request. Add credits with DeepSeek, then send again.',
			callout: {
				title: 'Provider error',
				code: '402 insufficient_balance',
				requestId: 'req_9d41e0b2',
				details: reason('Insufficient Balance'),
				actions: [
					{ label: 'Retry', icon: RotateCcw, onclick: noop },
					{ label: 'Add credits on DeepSeek', icon: ArrowUpRight, href: 'https://platform.deepseek.com', external: true },
				],
			},
		},
		{
			label: '404 · Model retired',
			prose: "DeepSeek no longer offers deepseek-flash-0624. Pick a replacement and I'll use it for this chat.",
			callout: {
				title: 'Model not found',
				code: '404 model_not_found',
				requestId: 'req_6a3e7c90',
				details: reason('The model deepseek-flash-0624 does not exist'),
				actions: [{ label: 'Choose a model', icon: Cpu, onclick: noop }],
			},
		},
	];

	const capacity: Case[] = [
		{
			label: '429 · Rate limited',
			prose: "DeepSeek is limiting requests on your key. I'll retry on my own when the window resets.",
			callout: {
				title: 'Rate limited',
				code: '429 rate_limit_exceeded',
				requestId: 'req_51c0de7e',
				details: reason('Rate limit reached for requests'),
				actions: [{ label: 'Retrying in 0:24', icon: Hourglass, onclick: noop }, { label: 'Try another model', icon: ArrowLeftRight, onclick: noop }],
			},
		},
		{
			label: '503 · Provider down',
			prose: "DeepSeek's servers are overloaded. Your message is saved; retry or switch models.",
			callout: {
				title: 'Provider unavailable',
				code: '503 service_unavailable',
				requestId: 'req_8e2a4410',
				details: reason('Server overloaded, please try again later'),
				actions: [{ label: 'Retry', icon: RotateCcw, onclick: noop }, { label: 'Try Claude Sonnet', icon: ArrowLeftRight, onclick: noop }],
			},
		},
		{
			label: '504 · Timed out',
			prose: 'No reply arrived within 60 seconds, so I stopped waiting.',
			callout: {
				title: 'Timed out',
				code: '504 gateway_timeout',
				requestId: 'req_e19b0d3c',
				details: reason('No first token after 60s'),
				actions: [{ label: 'Retry', icon: RotateCcw, onclick: noop }, { label: 'Try a faster model', icon: Zap, onclick: noop }],
			},
		},
	];

	const content: Case[] = [
		{
			label: '400 · Chat too long',
			prose: 'This chat is longer than deepseek-flash can read at once. I can summarise the older part and keep going.',
			callout: {
				title: 'Context limit',
				code: '400 context_length_exceeded',
				requestId: 'req_c41a9b03',
				details: reason('Maximum context length is 65536 tokens; you requested 71204'),
				actions: [{ label: 'Summarise & continue', icon: Sparkles, onclick: noop }, { label: 'Use a 128K model', icon: ArrowLeftRight, onclick: noop }],
			},
		},
		{
			label: '400 · Blocked by provider',
			prose: "DeepSeek's safety filter blocked this reply. Rephrasing usually helps; another model may answer.",
			callout: {
				title: 'Blocked by provider',
				code: '400 content_filter',
				requestId: 'req_0f9d2e61',
				details: reason('Content Exists Risk'),
				actions: [{ label: 'Rephrase', icon: Pencil, onclick: noop }, { label: 'Try another model', icon: ArrowLeftRight, onclick: noop }],
			},
		},
		{
			label: '413 · File too large',
			prose: 'bank-statements-2026.zip is 48 MB; chats take files up to 20 MB. Split it or share a Drive link.',
			callout: {
				title: 'Attachment rejected',
				code: '413 payload_too_large',
				details: reason('File exceeds the 20 MB limit (48.2 MB)', 'nomi.app'),
				actions: [{ label: 'Choose another file', icon: Paperclip, onclick: noop }, { label: 'Share a Drive link', icon: Link2, onclick: noop }],
			},
		},
	];

	const connection: Case[] = [
		{
			label: 'Network · Offline',
			prose: "I couldn't reach Nomi. Your message is in the outbox and sends when you reconnect.",
			callout: { title: "Can't connect", code: 'network_error', details: reason('Failed to fetch: no internet connection', 'nomi.app'), actions: [{ label: 'Retry now', icon: RotateCcw, onclick: noop }] },
		},
		{
			label: 'Tool · Gmail signed out',
			prose: "Tara couldn't send the email because Gmail signed Nomi out. The draft is saved.",
			callout: {
				title: 'Tool failed · gmail.send',
				code: '401 invalid_grant',
				requestId: 'req_t77e1',
				details: reason('Token has been expired or revoked.', 'gmail.googleapis.com'),
				actions: [{ label: 'Reconnect Gmail', icon: Plug, onclick: noop }, { label: 'Skip this step', onclick: noop }],
			},
		},
		{
			label: 'Nomi · Credits used',
			prose: "You've used this month's Nomi credits. Upgrade, or use your own provider key to keep going.",
			callout: {
				title: 'Out of Nomi credits',
				code: 'quota_exceeded',
				details: reason('1M of 1M tokens used · resets 1 Nov', 'nomi.app'),
				actions: [{ label: 'Upgrade plan', icon: ArrowUpRight, onclick: noop }, { label: 'Use my own key', icon: KeyRound, onclick: noop }],
			},
		},
	];

	const reply = "Food is October's biggest jump: Rp 2,1 jt against Rp 1,4 jt in September. Most of the difference is GoFood after";
</script>

{#snippet cases(list: Case[])}
	<div class="flex flex-col gap-7">
		{#each list as item (item.label)}
			<div class="flex flex-col gap-2.5">
				<SectionLabel label={item.label} />
				<p class="text-ui leading-relaxed text-text-2">{item.prose}</p>
				<ErrorCallout {...item.callout} />
			</div>
		{/each}
	</div>
{/snippet}

<GallerySection
	title="States & errors"
	description="Errors say what happened in plain words, quote the provider's own reason, and offer the next move. Cost is never guessed: providers don't report a charge on failed calls."
>
	<div class="grid items-start gap-x-8 gap-y-6 lg:grid-cols-2">
		<GalleryBoard title="Message states" note="user · assistant">
			<div class="flex flex-col gap-4">
				<SectionLabel label="User" />
				<MessageUser text="Kirim bukti transfer ke Bu Sri ya" status="sending" />
				<MessageUser text="Kirim bukti transfer ke Bu Sri ya" status="queued" oncancel={noop} />
				<MessageUser text="Kirim bukti transfer ke Bu Sri ya" status="failed" onretry={noop} onedit={noop} />
				<SectionLabel label="Assistant" />
				<ReplyText text={reply} status="streaming" />
				<ReplyText text={reply} status="stopped" elapsed="00:03" oncontinue={noop} />
				<ReplyText text={reply} status="interrupted" reason="Reply cut off after 18s. DeepSeek closed the stream." onrestart={noop} oncontinue={noop} />
			</div>
		</GalleryBoard>

		<GalleryBoard title="Banners" note="above the thread, one at a time">
			<div class="flex flex-col gap-4">
				<SectionLabel label="Connection" />
				<Banner kind="offline" text="You're offline. New messages wait in the outbox and send when you reconnect." action={{ label: 'Retry now', onclick: noop }} />
				<Banner kind="online" text="Back online. 1 queued message sent." />
				<SectionLabel label="Provider" />
				<Banner kind="slow" text="DeepSeek is slow right now (replies ~14s). Your chats still work." action={{ label: 'DeepSeek status', href: 'https://status.deepseek.com', external: true }} />
				<Banner kind="expiring" text="Your DeepSeek key expires in 3 days." action={{ label: 'Update key', onclick: noop }} />
				<SectionLabel label="Plan" />
				<Banner kind="quota" text="82% of this month's allowance used · resets 1 Nov." action={{ label: 'See usage', onclick: noop }} />
			</div>
		</GalleryBoard>

		<GalleryBoard title="Provider · key & billing" note="401 · 402 · 404">{@render cases(keyAndBilling)}</GalleryBoard>
		<GalleryBoard title="Provider · capacity" note="429 · 503 · 504">{@render cases(capacity)}</GalleryBoard>
		<GalleryBoard title="Content & context" note="400 · 400 · 413">{@render cases(content)}</GalleryBoard>
		<GalleryBoard title="Connection, tools & plan" note="network · tool · quota">{@render cases(connection)}</GalleryBoard>
	</div>
</GallerySection>
