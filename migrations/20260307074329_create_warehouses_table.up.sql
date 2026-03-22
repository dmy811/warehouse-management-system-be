-- Add up migration script here
CREATE TABLE IF NOT EXISTS public.warehouses (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(150) NOT NULL,
    address TEXT NOT NULL,
    photo TEXT,
    phone VARCHAR(20),
    deleted_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE public.warehouses
ADD CONSTRAINT unique_warehouses_name UNIQUE (name);

CREATE INDEX IF NOT EXISTS idx_warehouses_deleted_at
ON public.warehouses (deleted_at);

CREATE INDEX IF NOT EXISTS idx_warehouses_name
ON public.warehouses (name);