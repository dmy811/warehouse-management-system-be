-- Add up migration script here
CREATE TABLE IF NOT EXISTS public.users (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(150) NOT NULL,
    email VARCHAR(150) NOT NULL,
    password TEXT NOT NULL,
    photo TEXT,
    phone VARCHAR(20),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE public.users
ADD CONSTRAINT unique_users_email UNIQUE (email);

CREATE INDEX IF NOT EXISTS idx_users_email ON public.users (email);

CREATE INDEX IF NOT EXISTS idx_users_created_at ON public.users (created_at);