-- Add down migration script here
DROP TABLE IF EXISTS public.transfers;
DROP TYPE IF EXISTS transfers_status;