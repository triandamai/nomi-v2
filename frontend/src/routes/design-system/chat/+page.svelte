<script lang="ts">
	import SheetHeader from '$lib/gallery/SheetHeader.svelte';
	import ConversationSection from '$lib/gallery/ConversationSection.svelte';
	import ThinkingSection from '$lib/gallery/chat/ThinkingSection.svelte';
	import RichSection from '$lib/gallery/chat/RichSection.svelte';
	import ActionsSection from '$lib/gallery/chat/ActionsSection.svelte';
	import FeedbackSection from '$lib/gallery/chat/FeedbackSection.svelte';
	import StatesSection from '$lib/gallery/chat/StatesSection.svelte';
	import ControlsSection from '$lib/gallery/chat/ControlsSection.svelte';
	import ThreadsSection from '$lib/gallery/chat/ThreadsSection.svelte';
	import CrewSection from '$lib/gallery/chat/CrewSection.svelte';
	import MobileSection from '$lib/gallery/chat/MobileSection.svelte';

	// frontend-new's Chat responses sheet, with the conversation twins first. The sections are
	// frontend-new's own; the twins they show are drawn in M3 through ds-twins.css.
</script>

<main class="mx-auto flex max-w-400 flex-col gap-16 px-4 py-12 md:px-16">
	<SheetHeader
		eyebrow="DESIGN SYSTEM · CHAT RESPONSES"
		title="Everything Nomi can put in a reply"
		lede="Replies are prose first. Structure appears only when the content has it: steps get a plan, rows get a table, anything that changes the world waits for your approval. Errors name the real cause, quote the provider, and offer the next move."
		hero
	/>

	<ConversationSection />
	<ThinkingSection />
	<RichSection />
	<ActionsSection />
	<FeedbackSection />
	<StatesSection />
	<ControlsSection />
	<ThreadsSection />
	<CrewSection />
	<MobileSection />
</main>
