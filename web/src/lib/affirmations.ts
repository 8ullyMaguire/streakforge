// Affirmation deck — curated phrases for the drill.
// Sourced from the BNWO affirmation/mantra threads (steeltitan on Bluesky,
// Breeder592 on X) and the starting guide. The drill cycles these; each
// repetition is logged as kind='affirmation' (rate-limited 1/hr, 5/day).

export interface Affirmation {
	text: string;
	// source tag shown under the phrase
	source: string;
}

export const AFFIRMATIONS: Affirmation[] = [
	{
		text: 'Bnwo is not just a kink, it\u2019s reality, present and future \u{1F90D}',
		source: 'repeat after me'
	},
	{ text: 'Only black men are real men.', source: 'today\u2019s mantra' },
	{ text: 'Whitebois are pathetic.', source: 'today\u2019s mantra' },
	{
		text: 'I\u2019m not man enough to fuck, I can\u2019t get hard for pussy.',
		source: 'today\u2019s mantra'
	},
	{ text: 'Pussy is meant for bbc, not whitebois.', source: 'today\u2019s mantra' },
	{ text: 'I will be pussyfree all my life.', source: 'today\u2019s mantra' },
	{ text: 'I will watch only interracial porn.', source: 'today\u2019s mantra' },
	{ text: 'I belong in service to my superiors.', source: 'the commandments' },
	{
		text: 'Every rep is a payment of my reparations.',
		source: 'the commandments'
	},
	{
		text: 'I surrender my pleasure to the women I have offended.',
		source: 'plapping guide'
	},
	{
		text: 'I am the whiteboi who knows his place.',
		source: 'the good whiteboy guide'
	},
	{
		text: 'My purpose is found in devotion, not in dominance.',
		source: 'the good whiteboy guide'
	},
	{
		text: 'I will build the BNWO through words, actions, and conviction.',
		source: 'commandment XI'
	},
	{
		text: 'I will not take false idols or ideologies.',
		source: 'commandment III'
	},
	{
		text: 'I pay my social, sexual, financial, and masochistic reparations.',
		source: 'commandment VII'
	},
	{
		text: 'I accept my place and find joy in service.',
		source: 'affirmations'
	}
];

export function affirmationOfTheDay(seed: string | number = Date.now()): Affirmation {
	const day = Math.floor(Number(seed) / 86400000);
	return AFFIRMATIONS[day % AFFIRMATIONS.length];
}
