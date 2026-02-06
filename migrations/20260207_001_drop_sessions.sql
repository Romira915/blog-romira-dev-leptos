-- Migrate session store from PostgreSQL to Valkey (Redis-compatible)
DROP INDEX IF EXISTS idx_tower_sessions_expiry;
DROP TABLE IF EXISTS tower_sessions;
