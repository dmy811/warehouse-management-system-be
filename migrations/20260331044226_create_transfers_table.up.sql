-- Add up migration script here
CREATE TYPE transfers_status AS ENUM ('DRAFT', 'COMPLETED');

CREATE TABLE IF NOT EXISTS public.transfers (
    id                   BIGSERIAL PRIMARY KEY,
    from_warehouse_id    BIGINT             NOT NULL REFERENCES public.warehouses(id) ON DELETE CASCADE,
    to_warehouse_id      BIGINT             NOT NULL REFERENCES public.warehouses(id) ON DELETE CASCADE,
    status               transfers_status   NOT NULL DEFAULT 'DRAFT',
    notes                TEXT,
    created_by           BIGINT             NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    created_at           TIMESTAMPTZ        NOT NULL DEFAULT NOW(),
    updated_at           TIMESTAMPTZ        NOT NULL DEFAULT NOW()
);

CREATE TRIGGER trigger_transfers_updated_at
BEFORE UPDATE ON public.transfers
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();