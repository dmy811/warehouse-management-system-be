-- Add up migration script here
CREATE TABLE IF NOT EXISTS public.user_roles (
    id          BIGSERIAL PRIMARY KEY,
    user_id     BIGINT                   NOT NULL,
    role_id     BIGINT                   NOT NULL,
    created_at  TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE public.user_roles
ADD CONSTRAINT fk_user_roles_user
FOREIGN KEY (user_id) REFERENCES public.users (id) ON DELETE CASCADE;

ALTER TABLE public.user_roles
ADD CONSTRAINT fk_user_roles_role
FOREIGN KEY (role_id) REFERENCES public.roles (id) ON DELETE CASCADE;

ALTER TABLE public.user_roles
ADD CONSTRAINT unique_user_roles
UNIQUE (user_id, role_id);

CREATE INDEX IF NOT EXISTS idx_user_roles_role_id ON public.user_roles (role_id);
CREATE INDEX IF NOT EXISTS idx_user_roles_created_at ON public.user_roles (created_at);

CREATE TRIGGER trigger_user_roles_updated_at
BEFORE UPDATE ON public.user_roles
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();