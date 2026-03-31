-- Add up migration script here
CREATE TYPE shipments_status AS ENUM ('DRAFT', 'SHIPPED');

CREATE TABLE IF NOT EXISTS public.shipments (
    id                  BIGSERIAL PRIMARY KEY,
    warehouse_id        BIGINT                  NOT NULL REFERENCES public.warehouses(id) ON DELETE CASCADE,
    customer_id         BIGINT                  NOT NULL REFERENCES public.customers(id) ON DELETE CASCADE,
    reference_number    VARCHAR(100)            NOT NULL,
    status              shipments_status        NOT NULL DEFAULT 'DRAFT',
    notes               TEXT,
    created_at          TIMESTAMPTZ             NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ             NOT NULL DEFAULT NOW()
);


CREATE TRIGGER trigger_shipments_items_updated_at
BEFORE UPDATE ON public.shipments
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();