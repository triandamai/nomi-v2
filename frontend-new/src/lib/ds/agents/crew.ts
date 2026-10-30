// The crew as the design names them. Each owns a colour (crew-<id>, -bg, -ink) and a mascot.
// Which backend agent each one is (chitchat → Nomi, coding → Koda, …) is the app's business, not
// the design system's.

export const CREW = [
	{ id: 'nomi', name: 'Nomi' },
	{ id: 'koda', name: 'Koda' },
	{ id: 'dana', name: 'Dana' },
	{ id: 'kala', name: 'Kala' },
	{ id: 'rena', name: 'Rena' },
	{ id: 'tara', name: 'Tara' },
	{ id: 'maya', name: 'Maya' },
	{ id: 'raga', name: 'Raga' },
	{ id: 'personality', name: 'Personality' },
	{ id: 'supervisor', name: 'Supervisor' },
] as const;

export type CrewId = (typeof CREW)[number]['id'];
