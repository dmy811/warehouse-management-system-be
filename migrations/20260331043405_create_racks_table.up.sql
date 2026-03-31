-- Add up migration script here
CREATE TABLE IF NOT EXISTS public.racks (
    id           BIGSERIAL PRIMARY KEY,
    warehouse_id BIGINT        NOT NULL REFERENCES public.warehouses(id) ON DELETE CASCADE,
    code         VARCHAR(20)   NOT NULL,
    zone         VARCHAR(10),
    level        INT,
    description  TEXT,
    capacity     BIGINT,
    deleted_at   TIMESTAMPTZ,
    -- TIMESTAMPTZ adalah alias resmi PostgreSQL untuk TIMESTAMP WITH TIME ZONE
    -- TIMESTAMPTZ dan NOW() lebih umum dipakai di komunitas PostgreSQL karena lebih pendek. TIMESTAMP WITH TIME ZONE dan CURRENT_TIMESTAMP lebih verbose tapi lebih portable kalau mau pindah database engine lain seperti MySQL atau SQLite.
    created_at   TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    -- Rack code must be unique within a warehouse
    CONSTRAINT unique_racks_warehouse_code UNIQUE (warehouse_id, code)
);
 
CREATE INDEX idx_racks_warehouse_id ON public.racks (warehouse_id) WHERE deleted_at IS NULL;

 
CREATE TRIGGER trigger_racks_updated_at
BEFORE UPDATE ON public.racks
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();