-- Add up migration script here
CREATE TYPE stock_direction AS ENUM ('IN', 'OUT');
CREATE TYPE type_reference AS ENUM ('RECEIPT', 'SHIPMENT', 'TRANSFER');

CREATE TABLE IF NOT EXISTS public.stock_movements (
    id                   BIGSERIAL PRIMARY KEY,
    product_id           BIGINT             NOT NULL REFERENCES public.products(id) ON DELETE CASCADE,
    warehouse_id         BIGINT             NOT NULL REFERENCES public.warehouses(id) ON DELETE CASCADE,
    direction            stock_direction    NOT NULL,
    quantity             BIGINT             NOT NULL,
    reference_type       type_reference     NOT NULL,
    reference_id         BIGINT             NOT NULL,
    created_at           TIMESTAMPTZ        NOT NULL DEFAULT NOW(),
    updated_at           TIMESTAMPTZ        NOT NULL DEFAULT NOW()
);

CREATE TRIGGER trigger_stock_movements_updated_at
BEFORE UPDATE ON public.stock_movements
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();