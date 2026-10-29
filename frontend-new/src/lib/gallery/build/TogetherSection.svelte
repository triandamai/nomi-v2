<script lang="ts">
	import TeammateMessage from '$lib/ds/build/TeammateMessage.svelte';
	import Typing from '$lib/ds/build/Typing.svelte';
	import JoinedNote from '$lib/ds/build/JoinedNote.svelte';
	import Presence from '$lib/ds/build/Presence.svelte';
	import RequestQueue from '$lib/ds/build/RequestQueue.svelte';
	import PublishRequest from '$lib/ds/build/PublishRequest.svelte';
	import RoleMenu from '$lib/ds/build/RoleMenu.svelte';
	import BuildInvite from '$lib/ds/build/BuildInvite.svelte';
	import PeerCursor from '$lib/ds/build/PeerCursor.svelte';
	import PresenceMenu from '$lib/ds/build/PresenceMenu.svelte';
	import FollowBar from '$lib/ds/build/FollowBar.svelte';
	import GallerySection from '../GallerySection.svelte';
	import GalleryBoard from '../GalleryBoard.svelte';

	const noop = () => {};
	const sari = { name: 'Sari Wulandari', initials: 'SW', peer: 1 as const };
	const mario = { name: 'Mario Prasetyo', initials: 'MP' };
	let role = $state<'build' | 'prompt' | 'view'>('build');
</script>

<GallerySection
	title="Build together"
	description="Invite people into a project's chat. Everyone prompts the same Koda; asks queue so changes never collide; roles decide who can publish."
>
	<div class="flex flex-col gap-6">
		<GalleryBoard title="People in the chat" note="teammate message · typing · joined note · presence" inline>
			<div class="flex flex-col gap-5">
				<TeammateMessage who={sari} time="09:14" to="Koda" text="Can we add a Makanan section? Nasi uduk and roti bakar, same pickup times." />
				<Typing who={sari} />
				<JoinedNote name="Sari" role="prompt" publisher="Mario" />
				<div><Presence people={[mario, { ...sari, online: true }]} /></div>
			</div>
		</GalleryBoard>
		<GalleryBoard title="Asks in line" note="two people, one Koda: one change at a time" inline>
			<div class="flex flex-col gap-4">
				<RequestQueue
					asks={[
						{ text: 'Make the Pre-order button green', who: mario, state: 'running', meta: 'Mario · running · 00:21' },
						{ text: 'Add a Makanan section with nasi uduk', who: sari, state: 'next', meta: 'Sari · next' },
					]}
					onfirst={noop}
				/>
				<PublishRequest who={sari} meta="Makanan section · 2 files · built and tested in the preview" onpublish={noop} onreview={noop} />
			</div>
		</GalleryBoard>
		<div class="grid items-start gap-x-8 gap-y-6 lg:grid-cols-2">
			<GalleryBoard title="Roles" note="picked when inviting · changeable later in People" inline>
				<RoleMenu value={role} onchange={(next) => (role = next)} />
			</GalleryBoard>
			<GalleryBoard title="Invite email" note="what the invited person gets · one clear button" inline>
				<BuildInvite
					from="Mario Prasetyo"
					project="warung-kopi-menu"
					detail="Menu and pre-order page · live · deploy #2"
					role="prompt"
					href="#invite"
					sentTo="sari.w@gmail.com"
					expires="in 7 days"
				/>
			</GalleryBoard>
		</div>
		<GalleryBoard title="Live sessions" note="see who's here, what they're looking at, and follow them · like Figma" inline>
			<div class="flex flex-wrap items-start gap-8">
				<PeerCursor name="Sari" peer={1} />
				<PresenceMenu
					people={[
						{ ...sari, where: 'Viewing Code · +page.svelte · active now' },
						{ ...mario, where: 'you · Preview', you: true },
					]}
					onfollow={noop}
				/>
				<div class="w-130 max-w-full"><FollowBar name="Sari" peer={1} where="Code · +page.svelte" onstop={noop} /></div>
			</div>
		</GalleryBoard>
	</div>
</GallerySection>
