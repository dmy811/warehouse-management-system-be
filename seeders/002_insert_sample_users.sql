-- insert user role ADMIN
INSERT INTO public.users (name, email, password)
SELECT 'El Admin', 'admin@warehouse.com', '$argon2id$v=19$m=19456,t=2,p=1$F5Zcjg4LavVVybXNwMPzxA$DXxQ6zVtQwkAGSx9CyVre4UyT1CMKL2Ron/O5nhKZTw' -- admin123
WHERE NOT EXISTS(
    SELECT 1 FROM public.users
    WHERE LOWER(email) = LOWER('admin@warehouse.com')
    AND deleted_at IS NULL
);


-- insert user_role admin
INSERT INTO public.user_roles (user_id, role_id)
SELECT
    u.id,
    r.id
FROM public.users u, public.roles r
WHERE u.email = 'admin@warehouse.com' AND r.name = 'ADMIN'
ON CONFLICT (user_id, role_id) DO NOTHING;

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
WHERE u.email = 'admin@warehouse.com'
ORDER BY u.id;