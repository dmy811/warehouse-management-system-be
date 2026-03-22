INSERT INTO public.roles (name, created_at, updated_at)
VALUES
    ('keeper', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
    ('manager', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
ON CONFLICT (name) DO NOTHING;

SELECT id, name, created_at FROM public.roles ORDER BY id;