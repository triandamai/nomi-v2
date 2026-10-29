import { buildUrl, request, send, type HttpContext } from './http.js';
import type { Result } from './types.js';

/** Signed URLs last an hour unless asked otherwise. */
export const DEFAULT_SIGNED_URL_SECONDS = 3600;
/** Uploads can be big; give them longer than a normal request. */
const UPLOAD_TIMEOUT_MS = 300_000;

/** A file, or a folder (its name ends in `/`, with no size or date), in a listing. */
export interface FileObject {
  name: string;
  size: number;
  content_type: string | null;
  updated_at: string | null;
}

export interface UploadOptions {
  contentType?: string;
  /** Replace a file that's already at this path. Off by default, so an upload never overwrites by accident. */
  upsert?: boolean;
  cacheControl?: string;
}

function encodePath(path: string): string {
  return path
    .replace(/^\/+/, '')
    .split('/')
    .map(encodeURIComponent)
    .join('/');
}

/** Files in one bucket. Reached as `nomi.storage.from('avatars')`. */
export class BucketClient {
  constructor(
    private readonly http: HttpContext,
    private readonly bucket: string
  ) {}

  private objectPath(kind: 'object' | 'sign' | 'public', path: string): string {
    return `/storage/${kind}/${encodeURIComponent(this.bucket)}/${encodePath(path)}`;
  }

  upload(path: string, body: Blob | ArrayBuffer | ArrayBufferView | ReadableStream | string, options: UploadOptions = {}): Promise<Result<{ path: string }>> {
    const headers: Record<string, string> = { 'x-nomi-upsert': String(options.upsert ?? false) };
    const type = options.contentType ?? (typeof Blob !== 'undefined' && body instanceof Blob ? body.type : '');
    if (type) headers['content-type'] = type;
    if (options.cacheControl) headers['cache-control'] = options.cacheControl;
    return request(this.http, this.objectPath('object', path), {
      method: 'PUT',
      body,
      headers,
      timeoutMs: UPLOAD_TIMEOUT_MS
    });
  }

  async download(path: string): Promise<Result<Blob>> {
    const { response, error } = await send(this.http, this.objectPath('object', path), { timeoutMs: UPLOAD_TIMEOUT_MS });
    if (error || !response) return { data: null, error: error!, status: response?.status ?? 0 };
    return { data: await response.blob(), error: null, status: response.status };
  }

  /** Lists files under `prefix` (a folder path, or empty for the bucket's top level). */
  list(prefix = '', options: { limit?: number; offset?: number } = {}): Promise<Result<FileObject[]>> {
    return request(this.http, `/storage/list/${encodeURIComponent(this.bucket)}`, {
      query: { prefix, limit: options.limit?.toString(), offset: options.offset?.toString() }
    });
  }

  remove(paths: string[]): Promise<Result<null>> {
    return request(this.http, `/storage/object/${encodeURIComponent(this.bucket)}`, {
      method: 'DELETE',
      body: { paths }
    });
  }

  /** A URL anyone can open for `expiresIn` seconds, even in a private bucket. */
  async createSignedUrl(path: string, expiresIn = DEFAULT_SIGNED_URL_SECONDS): Promise<Result<{ signedUrl: string }>> {
    const result = await request<{ signed_url: string }>(this.http, this.objectPath('sign', path), {
      method: 'POST',
      body: { expires_in: expiresIn }
    });
    if (result.error) return result;
    return { data: { signedUrl: new URL(result.data.signed_url, this.http.baseUrl).toString() }, error: null, status: result.status };
  }

  /** The permanent URL of a file in a public bucket. Makes no request. */
  getPublicUrl(path: string): { publicUrl: string } {
    return { publicUrl: buildUrl(this.http.baseUrl, this.objectPath('public', path)) };
  }
}

export class StorageClient {
  constructor(private readonly http: HttpContext) {}

  from(bucket: string): BucketClient {
    return new BucketClient(this.http, bucket);
  }
}
