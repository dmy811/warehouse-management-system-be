-- Add up migration script here
CREATE TABLE IF NOT EXISTS public.goods_receipt_items (
    id          BIGSERIAL PRIMARY KEY,
    receipt_id  BIGINT        NOT NULL REFERENCES public.goods_receipts(id) ON DELETE CASCADE,
    product_id  BIGINT        NOT NULL REFERENCES public.products(id) ON DELETE CASCADE,
    quantity    BIGINT        NOT NULL CHECK (quantity >= 0),
    created_at  TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ   NOT NULL DEFAULT NOW()
);

CREATE TRIGGER trigger_goods_receipt_items_updated_at
BEFORE UPDATE ON public.goods_receipt_items
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();