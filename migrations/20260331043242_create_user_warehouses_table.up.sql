-- Add up migration script here
CREATE TABLE IF NOT EXISTS public.user_warehouses (
    id              BIGSERIAL PRIMARY KEY,
    user_id         BIGINT                   NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    warehouse_id    BIGINT                   NOT NULL REFERENCES public.warehouses(id) ON DELETE CASCADE,
    created_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT unique_user_warehouses UNIQUE (user_id, warehouse_id)
);
-- to query what warehouses owned by user id y
CREATE INDEX IF NOT EXISTS idx_user_warehouses_user_id ON public.user_warehouses(user_id);
-- to query whos got the access to this warahouse_id x
CREATE INDEX IF NOT EXISTS idx_user_warehouses_warehouse_id ON public.user_warehouses(warehouse_id);

CREATE TRIGGER trigger_user_warehouse_updated_at
BEFORE UPDATE ON public.user_warehouses
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();