-- Add up migration script here
CREATE TABLE IF NOT EXISTS public.transfer_items (
    id          BIGSERIAL PRIMARY KEY,
    transfer_id BIGINT        NOT NULL REFERENCES public.transfers(id) ON DELETE CASCADE,
    product_id  BIGINT        NOT NULL REFERENCES public.products(id) ON DELETE CASCADE,
    quantity    BIGINT        NOT NULL,
    created_at  TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ   NOT NULL DEFAULT NOW()
);

CREATE TRIGGER trigger_transfer_items_updated_at
BEFORE UPDATE ON public.transfer_items
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();