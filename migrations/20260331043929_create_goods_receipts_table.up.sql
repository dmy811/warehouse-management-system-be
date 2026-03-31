-- Add up migration script here
CREATE TYPE goods_receipts_status AS ENUM ('DRAFT', 'RECEIVED');

CREATE TABLE IF NOT EXISTS public.goods_receipts (
    id                  BIGSERIAL PRIMARY KEY,
    warehouse_id        BIGINT                  NOT NULL REFERENCES public.warehouses(id) ON DELETE CASCADE,
    supplier_id         BIGINT                  NOT NULL REFERENCES public.suppliers(id) ON DELETE CASCADE,
    received_by         BIGINT                  NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    reference_number    VARCHAR(100)            NOT NULL,
    status              goods_receipts_status   NOT NULL DEFAULT 'DRAFT',
    received_at         TIMESTAMPTZ,
    notes               TEXT,
    created_at          TIMESTAMPTZ             NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ             NOT NULL DEFAULT NOW()
);

CREATE TRIGGER trigger_goods_receipts_updated_at
BEFORE UPDATE ON public.goods_receipts
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();