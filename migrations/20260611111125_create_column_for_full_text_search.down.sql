-- Add down migration script here
ALTER TABLE public.users DROP COLUMN IF EXISTS search_vector;
ALTER TABLE public.warehouses DROP COLUMN IF EXISTS search_vector;

DROP INDEX IF EXISTS idx_users_search_vector;
DROP INDEX IF EXISTS idx_warehouses_search_vector;