<script lang="ts">
	import { ArrowUpRight, RefreshCw, Send, Sparkles, Trash2 } from '@lucide/svelte';
	import NavBar from '$lib/ds/mobile/NavBar.svelte';
	import Composer from '$lib/ds/mobile/Composer.svelte';
	import UserMessage from '$lib/ds/mobile/UserMessage.svelte';
	import Byline from '$lib/ds/mobile/Byline.svelte';
	import ActionBar from '$lib/ds/mobile/ActionBar.svelte';
	import StatList from '$lib/ds/mobile/StatList.svelte';
	import PhoneButton from '$lib/ds/mobile/PhoneButton.svelte';
	import ErrorCard from '$lib/ds/mobile/ErrorCard.svelte';
	import SheetHeader from '$lib/ds/mobile/SheetHeader.svelte';
	import SheetRow from '$lib/ds/mobile/SheetRow.svelte';
	import PhoneToast from '$lib/ds/mobile/PhoneToast.svelte';
	import Progress from '$lib/ds/mobile/Progress.svelte';
	import PhoneOption from '$lib/ds/mobile/PhoneOption.svelte';
	import ApprovalCard from '$lib/ds/mobile/ApprovalCard.svelte';
	import ChatRow from '$lib/ds/mobile/ChatRow.svelte';
	import GallerySection from '../GallerySection.svelte';
	import GalleryBoard from '../GalleryBoard.svelte';

	const noop = () => {};
	let picked = $state(false);
</script>

<GallerySection
	title="Mobile chat"
	description="Native patterns at 390 pt: sheets instead of popovers, 44 pt targets, 16 pt body text, the composer above the keyboard, banners at the top. Desktop cards are reused only where they fit without losing anything. The phone's own status bar, keyboard, home indicator and notifications aren't drawn by the app."
>
	<div class="grid items-start gap-x-8 gap-y-6 lg:grid-cols-2">
		<GalleryBoard title="Shell" note="nav · composer">
			<div class="flex w-97.5 max-w-full flex-col gap-12">
				<NavBar title="Food spending in October" who={{ agent: 'dana', name: 'Dana' }} onback={noop} onmenu={noop} />
				<Composer onsend={noop} onattach={noop} onrecord={noop} />
			</div>
		</GalleryBoard>

		<GalleryBoard title="Messages" note="bubble · byline · actions · data">
			<div class="flex w-89.5 max-w-full flex-col gap-5">
				<UserMessage text="How much did I spend on food in October?" />
				<Byline agent="dana" name="Dana" meta="14:02" />
				<ActionBar text="Food is the jump." onlike={noop} ondislike={noop} onshare={noop} onmore={noop} />
				<StatList
					title="Food by week"
					sub="Oct · Rp"
					rows={[
						{ label: '1–7 Oct', value: '480rb', delta: { value: '+12%', tone: 'bad' }, detail: 'Groceries 312rb · GoFood 168rb' },
						{ label: '8–14 Oct', value: '527rb', delta: { value: '+31%', tone: 'bad' }, detail: 'Groceries 286rb · GoFood 241rb' },
						{ label: '15–21 Oct', value: '623rb', delta: { value: '+48%', tone: 'bad' }, detail: 'Groceries 301rb · GoFood 322rb' },
						{ label: '22–28 Oct', value: '470rb', delta: { value: '−6%', tone: 'good' }, detail: 'Groceries 274rb · GoFood 196rb' },
					]}
					total={{ label: 'October', value: 'Rp 2,1 jt', delta: { value: '+20%', tone: 'bad' } }}
				/>
			</div>
		</GalleryBoard>

		<GalleryBoard title="Buttons & cards" note="44 pt targets">
			<div class="flex w-89.5 max-w-full flex-col gap-5">
				<PhoneButton label="Approve & send" icon={Send} onclick={noop} />
				<PhoneButton label="Don't send" variant="secondary" onclick={noop} />
				<PhoneButton label="Delete reply" variant="danger" icon={Trash2} onclick={noop} />
				<ErrorCard
					title="Out of credits"
					code="402"
					provider="DeepSeek"
					reason="Insufficient Balance"
					meta="deepseek-flash · key ••a91f · req_9d41e0b2"
					primary={{ label: 'Switch to Nomi credits', icon: Sparkles, onclick: noop }}
					secondary={{ label: 'Add credits on DeepSeek', icon: ArrowUpRight, href: 'https://platform.deepseek.com', external: true }}
				/>
			</div>
		</GalleryBoard>

		<GalleryBoard title="Sheets & banners" note="bottom sheet · toast">
			<div class="flex w-97.5 max-w-full flex-col gap-5">
				<div class="flex flex-col">
					<SheetHeader title="Reply" subtitle="Dana · 14:02" onclose={noop} />
					<SheetRow icon={RefreshCw} label="Regenerate" onclick={noop} />
				</div>
				<div class="px-4"><PhoneToast kind="success" title="Link copied" detail="Expires in 7 days" action={{ label: 'Undo', onclick: noop }} /></div>
			</div>
		</GalleryBoard>

		<GalleryBoard title="Decide on mobile" note="options · progress · approval">
			<div class="flex w-89.5 max-w-full flex-col gap-5">
				<Progress current={1} total={4} />
				<PhoneOption name="trip" key={1} title="Whoosh high-speed train" detail="45 min from Halim · about Rp 250rb each way" checked={picked} onselect={() => (picked = true)} />
				<ApprovalCard
					agent="tara"
					title="Tara wants to send an email"
					to="bu.sri.kosmawar@gmail.com"
					subject="Transfer sewa November sudah masuk"
					excerpt="Selamat siang Bu Sri, sewa kamar 3 untuk November (Rp 2.200.000) sudah saya transfer ke BCA. Bukti terlampir. Terima kasih."
					attachment="bukti-transfer-nov.jpg"
					approveLabel="Approve"
					approveIcon={Send}
					rejectLabel="Don't send"
					onapprove={noop}
					onreject={noop}
					onopen={noop}
				/>
			</div>
		</GalleryBoard>

		<GalleryBoard title="Chats list" note="the mobile home for conversations">
			<div class="w-97.5 max-w-full">
				<ChatRow href="#chat" agent="dana" title="Food spending in October" time="14:02" preview="Dana: Rp 2,1 jt on food, up 20% from September…" />
			</div>
		</GalleryBoard>
	</div>
</GallerySection>
