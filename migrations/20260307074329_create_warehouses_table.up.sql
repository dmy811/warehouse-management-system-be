-- Add up migration script here
CREATE TABLE IF NOT EXISTS public.warehouses (
    id          BIGSERIAL PRIMARY KEY,
    name        VARCHAR(150)             NOT NULL,
    address     TEXT                     NOT NULL,
    photo       TEXT,
    phone       VARCHAR(15),
    deleted_at  TIMESTAMP WITH TIME ZONE,
    created_at  TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Prevents duplicate names like "Gudang A" and "gudang a"
CREATE UNIQUE INDEX IF NOT EXISTS unique_warehouses_name_active
ON public.warehouses (LOWER(name)) 
WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_warehouses_deleted_at
ON public.warehouses (deleted_at)
WHERE deleted_at IS NULL;

CREATE TRIGGER trigger_warehouse_updated_at
BEFORE UPDATE ON public.warehouses
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();