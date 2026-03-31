-- Add up migration script here
CREATE TABLE IF NOT EXISTS public.roles (
    id          BIGSERIAL PRIMARY KEY,
    name        VARCHAR(50)              NOT NULL,
    created_at  TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE public.roles
ADD CONSTRAINT unique_roles_name UNIQUE (name);

-- UNIQUE otomatis bikin index
-- CREATE INDEX IF NOT EXISTS idx_roles_name ON public.roles (name);