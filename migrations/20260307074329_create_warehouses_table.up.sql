-- Add up migration script here
CREATE TABLE IF NOT EXISTS public.warehouses (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(150) NOT NULL,
    address TEXT NOT NULL,
    photo TEXT,
    phone VARCHAR(20),
    deleted_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Prevents duplicate names like "Gudang A" and "gudang a"
CREATE UNIQUE INDEX idx_warehouse_name_unique
ON public.warehouses (LOWER(name))
WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_warehouses_deleted_at
ON public.warehouses (deleted_at)
WHERE deleted_at IS NULL;

CREATE TRIGGER trigger_warehouse_updated_at
BEFORE UPDATE ON public.warehouses
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

-- CREATE TABLE IF NOT EXISTS racks (
--     id           SERIAL PRIMARY KEY,
--     warehouse_id INT           NOT NULL REFERENCES warehouses(id) ON DELETE CASCADE,
--     code         VARCHAR(20)   NOT NULL,
--     zone         VARCHAR(10),
--     level        INT,
--     description  TEXT,
--     capacity     INT,
--     deleted_at   TIMESTAMPTZ,
--     created_at   TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
--     updated_at   TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
--     -- Rack code must be unique within a warehouse
--     UNIQUE (warehouse_id, code)
-- );
 
-- CREATE INDEX idx_racks_warehouse_id ON racks (warehouse_id) WHERE deleted_at IS NULL;
 
-- -- Auto-update updated_at triggers
-- CREATE TRIGGER warehouses_updated_at
--     BEFORE UPDATE ON warehouses
--     FOR EACH ROW EXECUTE FUNCTION update_updated_at();
 
-- CREATE TRIGGER racks_updated_at
--     BEFORE UPDATE ON racks
--     FOR EACH ROW EXECUTE FUNCTION update_updated_at();