-- Profile pictures are now served by Nomi at /avatars/<user>/<file> (from the bucket, or disk)
-- rather than linked straight at the bucket, whose public URL often didn't exist (private
-- buckets, S3-compatible services without S3_PUBLIC_URL_BASE). Pictures already uploaded keep
-- their file; only the link changes.
UPDATE user_profiles
SET avatar_url = '/' || substring(avatar_url from '(avatars/[0-9a-fA-F-]{36}/[0-9a-fA-F-]{36}\.(?:png|jpg|webp|gif))$'),
    updated_at = now()
WHERE avatar_url ~ 'avatars/[0-9a-fA-F-]{36}/[0-9a-fA-F-]{36}\.(png|jpg|webp|gif)$'
  AND avatar_url NOT LIKE '/avatars/%';
