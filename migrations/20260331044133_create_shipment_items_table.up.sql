-- Add up migration script here
CREATE TABLE IF NOT EXISTS public.shipment_items (
    id          BIGSERIAL PRIMARY KEY,
    shipment_id BIGINT        NOT NULL REFERENCES public.shipments(id) ON DELETE CASCADE,
    product_id  BIGINT        NOT NULL REFERENCES public.products(id) ON DELETE CASCADE,
    quantity    BIGINT        NOT NULL CHECK (quantity >= 0),
    created_at  TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ   NOT NULL DEFAULT NOW()
);


CREATE TRIGGER trigger_shipment_items_updated_at
BEFORE UPDATE ON public.shipment_items
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();