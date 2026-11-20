<script lang="ts">
	import '$lib/landing/landing.css';
	import { m } from '$lib/paraglide/messages';
	import LandingNav from '$lib/landing/LandingNav.svelte';
	import LandingHero, { type HeroChip, type HeroToast } from '$lib/landing/LandingHero.svelte';
	import LandingSteps, { type StepsCopy } from '$lib/landing/LandingSteps.svelte';
	import LandingCrew, { type CrewCard } from '$lib/landing/LandingCrew.svelte';
	import LandingExport, { type ExportCopy } from '$lib/landing/LandingExport.svelte';
	import LandingFaq, { type FaqItem } from '$lib/landing/LandingFaq.svelte';
	import LandingCta from '$lib/landing/LandingCta.svelte';
	import LandingFooter from '$lib/landing/LandingFooter.svelte';
	import { getLocale, setLocale, type Locale } from '$lib/paraglide/runtime';

	// The landing page (Figma, Nomi Builder → 05 Marketing → "Landing v2 · Playful (ID)", 170:2766),
	// for people who aren't signed in: the nav, the hero, How it works, the crew, Own everything, the
	// FAQ, the closing call to action and the footer. The waitlist is left out (the backend has no
	// public waitlist: BACKEND_TODO.md); the dark and 390 frames wait for their design to be read
	// (docs/ux/work/landing).
	const HOW = 'cara-kerja';
	const CREW = 'kru';
	const FAQ = 'tanya';

	const faq: FaqItem[] = [
		{ question: m.landing_faq_q1(), answer: m.landing_faq_a1() },
		{ question: m.landing_faq_q2(), answer: m.landing_faq_a2() },
		{ question: m.landing_faq_q3(), answer: m.landing_faq_a3() },
		{ question: m.landing_faq_q4(), answer: m.landing_faq_a4() },
		{ question: m.landing_faq_q5(), answer: m.landing_faq_a5() },
		{ question: m.landing_faq_q6(), answer: m.landing_faq_a6() },
	];
	const locale = getLocale();
	const languages = [
		{ locale: 'id', label: 'Bahasa Indonesia', current: locale === 'id' },
		{ locale: 'en', label: 'English', current: locale === 'en' },
	];

	let draft = $state(m.landing_composer_example());

	const chips: HeroChip[] = [
		{ agent: 'dana', label: m.landing_chip_rent() },
		{ agent: 'koda', label: m.landing_chip_menu() },
		{ agent: 'kala', label: m.landing_chip_prayer() },
		{ agent: 'tara', label: m.landing_chip_invoice() },
	];
	const toasts: HeroToast[] = [
		{ agent: 'dana', name: 'Dana', line: m.landing_toast_dana() },
		{ agent: 'kala', name: 'Kala', line: m.landing_toast_kala() },
		{ agent: 'maya', name: 'Maya', line: m.landing_toast_maya() },
		{ agent: 'koda', name: 'Koda', line: m.landing_toast_koda() },
		{ agent: 'tara', name: 'Tara', line: m.landing_toast_tara() },
		{ agent: 'rena', name: 'Rena', line: m.landing_toast_rena() },
	];
	const steps: StepsCopy = {
		title: m.landing_how_title(),
		lead: m.landing_how_lead(),
		steps: [
			{ title: m.landing_step1_title(), body: m.landing_step1_body() },
			{ title: m.landing_step2_title(), body: m.landing_step2_body() },
			{ title: m.landing_step3_title(), body: m.landing_step3_body() },
		],
		ask: { file: m.landing_step1_file(), meta: m.landing_step1_meta(), message: m.landing_step1_ask(), reply: m.landing_step1_reply() },
		build: {
			head: m.landing_step2_head(),
			badge: m.landing_step2_badge(),
			tasks: [m.landing_step2_task1(), m.landing_step2_task2(), m.landing_step2_task3(), m.landing_step2_task4()],
			working: m.landing_step2_working(),
		},
		live: {
			url: m.landing_step3_url(),
			badge: m.landing_step3_badge(),
			app: m.landing_step3_app(),
			note: m.landing_step3_note(),
			items: [
				[m.landing_step3_item1(), m.landing_step3_price1()],
				[m.landing_step3_item2(), m.landing_step3_price2()],
			],
			share: m.landing_step3_share(),
			export: m.landing_step3_export(),
		},
	};
	const crew: CrewCard[] = [
		{ agent: 'nomi', name: 'Nomi', role: m.landing_crew_nomi_role(), body: m.landing_crew_nomi_body(), example: m.landing_crew_nomi_example() },
		{ agent: 'sora', name: 'Sora', role: m.landing_crew_sora_role(), body: m.landing_crew_sora_body(), example: m.landing_crew_sora_example() },
		{ agent: 'koda', name: 'Koda', role: m.landing_crew_koda_role(), body: m.landing_crew_koda_body(), example: m.landing_crew_koda_example() },
		{ agent: 'iris', name: 'Iris', role: m.landing_crew_iris_role(), body: m.landing_crew_iris_body(), example: m.landing_crew_iris_example() },
		{ agent: 'dana', name: 'Dana', role: m.landing_crew_dana_role(), body: m.landing_crew_dana_body(), example: m.landing_crew_dana_example() },
		{ agent: 'kala', name: 'Kala', role: m.landing_crew_kala_role(), body: m.landing_crew_kala_body(), example: m.landing_crew_kala_example() },
		{ agent: 'maya', name: 'Maya', role: m.landing_crew_maya_role(), body: m.landing_crew_maya_body(), example: m.landing_crew_maya_example() },
		{ agent: 'tara', name: 'Tara', role: m.landing_crew_tara_role(), body: m.landing_crew_tara_body(), example: m.landing_crew_tara_example() },
		{ agent: 'rena', name: 'Rena', role: m.landing_crew_rena_role(), body: m.landing_crew_rena_body(), example: m.landing_crew_rena_example() },
		{ agent: 'raga', name: 'Raga', role: m.landing_crew_raga_role(), body: m.landing_crew_raga_body(), example: m.landing_crew_raga_example() },
		{ agent: 'supervisor', name: 'Supervisor', role: m.landing_crew_supervisor_role(), body: m.landing_crew_supervisor_body(), example: m.landing_crew_supervisor_example() },
		{ agent: 'personality', name: 'Personality', role: m.landing_crew_personality_role(), body: m.landing_crew_personality_body(), example: m.landing_crew_personality_example() },
	];
	const own: ExportCopy = {
		title: m.landing_own_title(),
		lead: m.landing_own_lead(),
		checks: [m.landing_own_code(), m.landing_own_models()],
		files: {
			chats: [m.landing_own_chats(), m.landing_own_chats_meta()],
			memory: [m.landing_own_memory(), m.landing_own_memory_meta()],
			project: [m.landing_own_project(), m.landing_own_project_meta()],
		},
		panel: { title: m.landing_own_panel(), ready: m.landing_own_ready(), zip: m.landing_own_zip(), size: m.landing_own_size(), download: m.landing_own_download() },
	};
</script>

<svelte:head>
	<title>{m.landing_title()}</title>
	<meta name="description" content={m.landing_description()} />
</svelte:head>

<div class="page" data-landing data-theme="light">
	<LandingNav
		home={m.landing_home()}
		homeHref="/welcome"
		label={m.landing_nav_label()}
		links={[
			{ href: `#${HOW}`, label: m.landing_nav_how() },
			{ href: `#${CREW}`, label: m.landing_nav_crew() },
			{ href: `#${FAQ}`, label: m.landing_nav_faq() },
		]}
		signIn={{ href: '/sign-in', label: m.landing_sign_in() }}
		tryFree={{ href: '/register', label: m.landing_try_free() }}
	/>
	<main>
		<LandingHero
			title={m.landing_hero_title()}
			lines={[
				[m.landing_hero_1a(), m.landing_hero_1b()],
				[m.landing_hero_2a(), m.landing_hero_2b()],
				[m.landing_hero_3a(), m.landing_hero_3b()],
			]}
			lead={m.landing_hero_lead()}
			label={m.landing_composer_label()}
			bind:value={draft}
			action="?/start"
			attach={m.landing_attach()}
			voice={m.landing_voice()}
			send={m.landing_send()}
			tryLabel={m.landing_try()}
			{chips}
			{toasts}
		/>
		<LandingSteps id={HOW} copy={steps} />
		<LandingCrew id={CREW} title={m.landing_crew_title()} lead={m.landing_crew_lead()} {crew} />
		<LandingExport copy={own} />
		<LandingFaq
			id={FAQ}
			title={m.landing_faq_title()}
			ask={{ title: m.landing_faq_ask_title(), note: m.landing_faq_ask_note(), prompt: m.landing_faq_ask_prompt(), start: m.landing_faq_ask_start() }}
			action="?/start"
			items={faq}
		/>
		<LandingCta
			title={m.landing_cta_title()}
			lead={m.landing_cta_lead()}
			label={m.landing_composer_label()}
			placeholder={m.landing_cta_placeholder()}
			action="?/start"
			attach={m.landing_attach()}
			voice={m.landing_voice()}
			send={m.landing_send()}
		/>
	</main>
	<LandingFooter
		home={m.landing_home()}
		homeHref="/welcome"
		navLabel={m.landing_footer_nav()}
		tagline={m.landing_footer_tagline()}
		columns={[
			{
				title: m.landing_footer_product(),
				links: [
					{ href: `#${HOW}`, label: m.landing_nav_how() },
					{ href: `#${CREW}`, label: m.landing_footer_crew() },
				],
			},
			{ title: m.landing_footer_help(), links: [{ href: `#${FAQ}`, label: m.landing_nav_faq() }] },
		]}
		legal={m.landing_footer_legal({ year: String(new Date().getFullYear()) })}
		languageLabel={m.landing_footer_language()}
		{languages}
		onlanguage={(next) => setLocale(next as Locale)}
	/>
</div>

<style>
	.page {
		min-height: 100vh;
		overflow-x: clip;
	}

	main {
		display: block;
	}
</style>
