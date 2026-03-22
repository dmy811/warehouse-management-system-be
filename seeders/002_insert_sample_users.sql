-- insert user role keeper
INSERT INTO public.users (name, email, password, phone, created_at, updated_at)
VALUES
    (
        'John Keeper',
        'keeper@warehouse.com',
        '$argon2id$v=19$m=65536,t=3,p=1$cmFuZG9tc2FsdA$SOKL051B9Tb+OVguQvildaUKL9Gu1FdS3wqCat/DWwE', -- keeper123
        '+6283774468582',
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP
    )
ON CONFLICT (email) DO NOTHING;

-- insert user role keeper
INSERT INTO public.users (name, email, password, phone, created_at, updated_at)
VALUES
    (
        'Jane Manager',
        'manager@warehouse.com',
        '$argon2id$v=19$m=65536,t=3,p=1$cmFuZG9tc2FsdA$fNsgl7wIZgDz7juwyhVa6c5fI00ufT5TocI62aFmcX8', -- manager123
        '+6277336645518',
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP
    )
ON CONFLICT (email) DO NOTHING;

-- insert user_role keeper
INSERT INTO public.user_roles (user_id, role_id, created_at, updated_at)
SELECT
    u.id,
    r.id,
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
FROM public.users u, public.roles r
WHERE u.email = 'keeper@warehouse.com' AND r.name = 'keeper'
ON CONFLICT (user_id) DO NOTHING;

-- insert user_role manager
INSERT INTO public.user_roles (user_id, role_id, created_at, updated_at)
SELECT
    u.id,
    r.id,
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
FROM public.users u, public.roles r
WHERE u.email = 'manager@warehouse.com' AND r.name = 'manager'
ON CONFLICT (user_id) DO NOTHING;

-- verify the inserted users and their roles
SELECT 
    u.id,
    u.name,
    u.email,
    u.phone,
    r.name as role_name,
    ur.created_at as role_assigned_at
FROM public.users u
LEFT JOIN public.user_roles ur ON u.id = ur.user_id
LEFT JOIN public.roles r ON ur.role_id = r.id
WHERE u.email IN ('keeper@warehouse.com', 'manager@warehouse.com')
ORDER BY u.id;