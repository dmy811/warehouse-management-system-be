-- Add up migration script here
CREATE TABLE IF NOT EXISTS public.products (
    id          BIGSERIAL PRIMARY KEY,
    name        VARCHAR(100)  NOT NULL,
    sku         VARCHAR(50)   NOT NULL,
    unit        VARCHAR(20)   NOT NULL,
    description TEXT,
    photo       TEXT,
    min_stock   INT           NOT NULL DEFAULT 0,
    category_id BIGINT        NOT NULL REFERENCES public.categories(id),
    deleted_at  TIMESTAMPTZ,
    created_at  TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ   NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS unique_products_sku_active
ON public.products (sku) 
WHERE deleted_at IS NULL;

CREATE INDEX idx_products_deleted_at ON public.products (deleted_at) WHERE deleted_at IS NULL;

CREATE TRIGGER products_updated_at
BEFORE UPDATE ON public.products
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();