// Chat responses → Clarify: the questions asked before a plan, and the answers they collect. One
// of many gives `choice`; many of many `choices`; typed in its own words `other`; skipped `null`.

export type ClarifyQuestion = {
	/** Short name on its tab and in the review, e.g. "Getting there". */
	topic: string;
	title: string;
	helper?: string;
	multiple?: boolean;
	options: { title: string; detail?: string; recommended?: boolean }[];
	/** Offers a "Something else…" answer in your own words (one of many only). */
	other?: boolean;
};

export type ClarifyAnswer = { choice: number } | { choices: number[] } | { other: string } | null;

/** Where a question starts: the recommendation picked, nothing ticked, or nothing. */
export function initialAnswer(question: ClarifyQuestion): ClarifyAnswer {
	if (question.multiple) return { choices: [] };
	const recommended = question.options.findIndex((option) => option.recommended);
	return recommended >= 0 ? { choice: recommended } : null;
}

export function isAnswered(answer: ClarifyAnswer): boolean {
	if (!answer) return false;
	if ('choices' in answer) return answer.choices.length > 0;
	if ('other' in answer) return answer.other.trim() !== '';
	return true;
}
