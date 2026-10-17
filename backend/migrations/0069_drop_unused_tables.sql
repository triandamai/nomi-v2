-- Tables nothing writes to: link codes (account linking went through sign-in instead) and
-- session participants (chats have one owner; projects have their own members).
DROP TABLE IF EXISTS link_codes;
DROP TABLE IF EXISTS session_participants;
