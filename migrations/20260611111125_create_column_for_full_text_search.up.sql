-- Add up migration script here
-- Users table
ALTER TABLE public.users
ADD COLUMN search_vector tsvector
    GENERATED ALWAYS AS (
        to_tsvector('simple', COALESCE(name, '') || ' ' || COALESCE(email, ''))
    ) STORED;

CREATE INDEX idx_users_search_vector
ON public.users USING GIN(search_vector);

-- Warehouse table
ALTER TABLE public.warehouses
ADD COLUMN search_vector tsvector
    GENERATED ALWAYS AS (
        to_tsvector('simple', COALESCE(name, '') || ' ' || COALESCE(email, ''))
    ) STORED;

CREATE INDEX idx_warehouses_search_vector
ON public.warehouses USING GIN(search_vector);