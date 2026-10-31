<script lang="ts">
	import SheetHeader from '$lib/gallery/SheetHeader.svelte';
	import ThinkingSection from '$lib/gallery/chat/ThinkingSection.svelte';
	import RichSection from '$lib/gallery/chat/RichSection.svelte';
	import ActionsSection from '$lib/gallery/chat/ActionsSection.svelte';
	import FeedbackSection from '$lib/gallery/chat/FeedbackSection.svelte';
	import StatesSection from '$lib/gallery/chat/StatesSection.svelte';
	import ControlsSection from '$lib/gallery/chat/ControlsSection.svelte';
	import ThreadsSection from '$lib/gallery/chat/ThreadsSection.svelte';
	import CrewSection from '$lib/gallery/chat/CrewSection.svelte';
	import MobileSection from '$lib/gallery/chat/MobileSection.svelte';
</script>

<SheetHeader
	eyebrow="DESIGN SYSTEM · CHAT RESPONSES"
	title="Everything Nomi can put in a reply"
	lede="Replies are prose first. Structure appears only when the content has it: steps get a plan, rows get a table, anything that changes the world waits for your approval. Errors name the real cause, quote the provider, and offer the next move."
	hero
/>

<ThinkingSection />
<RichSection />
<ActionsSection />
<FeedbackSection />
<StatesSection />
<ControlsSection />
<ThreadsSection />
<CrewSection />
<MobileSection />
