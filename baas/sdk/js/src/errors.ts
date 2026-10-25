/** What every failed call returns in `error`. `status` is 0 when the request never got an answer. */
export class NomiError extends Error {
  readonly status: number;
  readonly code: string;
  readonly details?: unknown;

  constructor(message: string, status: number, code: string, details?: unknown) {
    super(message);
    this.name = 'NomiError';
    this.status = status;
    this.code = code;
    this.details = details;
  }
}
