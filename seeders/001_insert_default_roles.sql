INSERT INTO public.roles (name)
VALUES
    ('ADMIN'),
    ('MANAGER'),
    ('STAFF'),
    ('VIEWER')
ON CONFLICT (name) DO NOTHING;

SELECT id, name, created_at FROM public.roles ORDER BY id;