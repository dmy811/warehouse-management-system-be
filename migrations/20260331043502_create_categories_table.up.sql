-- Add up migration script here
CREATE TABLE IF NOT EXISTS public.categories (
    id          BIGSERIAL PRIMARY KEY,
    name        VARCHAR(100)  NOT NULL,
    deleted_at  TIMESTAMPTZ,
    created_at  TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ   NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_categories_deleted_at ON public.categories (deleted_at)
WHERE deleted_at IS NULL;

CREATE TRIGGER trigger_categories_updated_at
BEFORE UPDATE ON public.categories
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();