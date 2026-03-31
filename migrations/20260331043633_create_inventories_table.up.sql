-- Add up migration script here
CREATE TABLE IF NOT EXISTS public.inventories (
    id              BIGSERIAL PRIMARY KEY,
    warehouse_id    BIGINT        NOT NULL REFERENCES public.warehouses(id) ON DELETE CASCADE,
    product_id      BIGINT        NOT NULL REFERENCES public.products(id) ON DELETE CASCADE,
    quantity        BIGINT        NOT NULL CHECK (quantity >= 0),
    created_at      TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ   NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_inventories_warehouse_product UNIQUE (warehouse_id, product_id)
);

CREATE INDEX idx_inventories_product_id ON public.inventories(product_id);

CREATE TRIGGER trigger_inventories_updated_at
BEFORE UPDATE ON public.inventories
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();