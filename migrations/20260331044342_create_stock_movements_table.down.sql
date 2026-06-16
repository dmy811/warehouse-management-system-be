-- Add down migration script here
DROP TABLE IF EXISTS public.stock_movements;
DROP TYPE IF EXISTS stock_direction;
DROP TYPE IF EXISTS type_reference;