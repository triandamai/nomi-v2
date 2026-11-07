import type { SubmitFunction } from '@sveltejs/kit';

/**
 * Whether a form is out with the server, for a button's `loading`. Use as
 * `use:enhance={sending.submit}`; it stays on through a redirect until the next page has loaded.
 * `reset: false` keeps what was typed when the server sends the form back with an error.
 */
export class FormPending {
	on = $state(false);
	#reset: boolean;

	constructor({ reset = true }: { reset?: boolean } = {}) {
		this.#reset = reset;
	}

	submit: SubmitFunction = () => {
		this.on = true;
		return async ({ update }) => {
			try {
				await update({ reset: this.#reset });
			} finally {
				this.on = false;
			}
		};
	};
}
