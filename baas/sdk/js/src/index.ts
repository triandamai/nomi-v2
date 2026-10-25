export { createClient, NomiClient, type ClientOptions } from './client.js';
export { NomiError } from './errors.js';
export {
  AuthClient,
  AuthAdmin,
  type AuthEvent,
  type AuthListener,
  type AuthOptions,
  type AdminUserAttributes,
  type Identity,
  type OAuthOptions,
  type Provider,
  type Session,
  type SessionStore,
  type User
} from './auth.js';
export { QueryBuilder, type CountOptions, type FilterOperator, type OrderOptions } from './query.js';
export { StorageClient, BucketClient, type FileObject, type UploadOptions } from './storage.js';
export { RealtimeClient, type Change, type ChangeEvent, type ChangeListener, type RealtimeOptions } from './realtime.js';
export { FunctionsClient, type InvokeOptions } from './functions.js';
export type { AnyDatabase, DatabaseDef, Result, RpcDef, TableDef } from './types.js';
export { VERSION } from './version.js';
