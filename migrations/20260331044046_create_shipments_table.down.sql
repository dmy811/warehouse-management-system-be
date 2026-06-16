-- Add down migration script here
DROP TABLE IF EXISTS public.shipments;
DROP TYPE IF EXISTS shipments_status;