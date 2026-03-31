-- Add up migration script here
CREATE TABLE IF NOT EXISTS public.inventory_racks (
    id              BIGSERIAL PRIMARY KEY,
    inventory_id    BIGINT        NOT NULL REFERENCES public.inventories(id) ON DELETE CASCADE,
    rack_id         BIGINT        NOT NULL REFERENCES public.racks(id) ON DELETE CASCADE,
    quantity        BIGINT        NOT NULL CHECK (quantity >= 0),
    created_at      TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ   NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_inventory_racks UNIQUE(inventory_id, rack_id)
);

CREATE INDEX idx_inventory_racks_rack_id ON public.inventory_racks(rack_id);

CREATE TRIGGER trigger_inventory_racks_updated_at
BEFORE UPDATE ON public.inventory_racks
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();