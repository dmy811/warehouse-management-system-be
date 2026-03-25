-- Add up migration script here
CREATE TABLE IF NOT EXISTS public.roles (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(150) NOT NULL
);

ALTER TABLE public.roles
ADD CONSTRAINT unique_roles_name UNIQUE (name);

CREATE INDEX IF NOT EXISTS idx_roles_name ON public.roles (name);