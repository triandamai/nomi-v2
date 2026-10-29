<script lang="ts">
	import { CalendarDays, Send } from '@lucide/svelte';
	import Handoff from '$lib/ds/chat/Handoff.svelte';
	import Boundary from '$lib/ds/chat/Boundary.svelte';
	import MemoryUsed from '$lib/ds/chat/MemoryUsed.svelte';
	import MemorySaved from '$lib/ds/chat/MemorySaved.svelte';
	import ConnectPrompt from '$lib/ds/chat/ConnectPrompt.svelte';
	import ApprovalCard from '$lib/ds/chat/ApprovalCard.svelte';
	import BackgroundNote from '$lib/ds/chat/BackgroundNote.svelte';
	import CrewWorking from '$lib/ds/chat/CrewWorking.svelte';
	import SupervisorStopped from '$lib/ds/chat/SupervisorStopped.svelte';
	import SessionExpired from '$lib/ds/chat/SessionExpired.svelte';
	import SectionLabel from '$lib/ds/status/SectionLabel.svelte';
	import GallerySection from '../GallerySection.svelte';
	import GalleryBoard from '../GalleryBoard.svelte';

	const noop = () => {};
</script>

<GallerySection
	title="Crew behaviour"
	description="Hand-offs are visible, limits are explained with an alternative, memory is never silent, and long work can run without you watching."
>
	<div class="grid items-start gap-6 lg:grid-cols-2">
		<GalleryBoard title="Hand-off & limits" note="who answers · what we won't do">
			<div class="flex flex-col gap-4">
				<SectionLabel label="Hand-off" />
				<Handoff from={{ agent: 'nomi', name: 'Nomi' }} to={{ agent: 'dana', name: 'Dana' }} reason="she has your transaction records" onback={noop} />
				<SectionLabel label="Can't do that" />
				<Boundary
					title="I can't cancel subscriptions for you"
					reason="Nomi only records money. It never signs in to Netflix or your bank, so cancelling stays with you. Here's what I can do instead:"
					alternatives={[
						{ agent: 'kala', title: 'Remind me 3 days before the next charge', detail: 'Kala · 2 Nov, 09:00', onclick: noop },
						{ agent: 'nomi', title: "Open Netflix's cancel page", detail: 'netflix.com/cancelplan', href: 'https://www.netflix.com/cancelplan', external: true },
						{ agent: 'dana', title: 'Stop tracking it as a subscription', detail: 'Dana · keeps past records', onclick: noop },
					]}
				/>
			</div>
		</GalleryBoard>

		<GalleryBoard title="Memory & connections" note="never silent">
			<div class="flex flex-col gap-4">
				<SectionLabel label="Memory used" />
				<MemoryUsed fact="Sari is vegetarian" onmanage={noop} />
				<SectionLabel label="Memory saved" />
				<MemorySaved fact="you and Sari prefer trains to driving" onedit={noop} onundo={noop} />
				<SectionLabel label="Connect a tool" />
				<ConnectPrompt
					agent="tara"
					icon={CalendarDays}
					service="Google Calendar"
					why="Tara needs it to check your weekends"
					reads="Read your events and free/busy times"
					alternative="Or tell me your free weekends"
					href="https://accounts.google.com"
					onlater={noop}
				/>
			</div>
		</GalleryBoard>

		<GalleryBoard title="Approval paths" note="edit the draft · rejected">
			<ApprovalCard
				agent="tara"
				title="Editing Tara's email"
				status="editing"
				fields={[
					{ key: 'to', value: 'bu.sri.kosmawar@gmail.com' },
					{ key: 'subject', value: 'Transfer sewa November sudah masuk' },
				]}
				body="Selamat siang Bu Sri, sewa kamar 3 untuk November (Rp 2.200.000) sudah saya transfer ke BCA. Bukti terlampir. Mohon maaf telat dua hari ya, Bu. Terima kasih."
				approveLabel="Save & send"
				approveIcon={Send}
				onsave={noop}
				oncancel={noop}
			/>
		</GalleryBoard>

		<GalleryBoard title="Background work & session" note="leave · stop · signed out">
			<div class="flex flex-col gap-4">
				<SectionLabel label="Running in the background" />
				<BackgroundNote eta="~3 min" />
				<SectionLabel label="Crew working" />
				<CrewWorking crew={['nomi', 'dana', 'tara']} onstop={noop} />
				<SectionLabel label="Stopped by supervisor" />
				<SupervisorStopped
					meta="14:31 · you asked"
					tasks={[
						{ agent: 'nomi', title: 'Find Whoosh trains', detail: 'done before stop', state: 'done' },
						{ agent: 'nomi', title: 'Shortlist guesthouses', detail: 'stopped at 2 of 3 found', state: 'stopped' },
						{ agent: 'tara', title: 'Share itinerary with Sari', detail: 'not started · nothing sent', state: 'waiting' },
					]}
					ondiscard={noop}
					onresume={noop}
				/>
				<SectionLabel label="Signed out" />
				<SessionExpired draft="Find Whoosh trains for Sat 1 Nov…" signInHref="/sign-in" />
			</div>
		</GalleryBoard>
	</div>
</GallerySection>
