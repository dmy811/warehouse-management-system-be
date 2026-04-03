-- Add up migration script here
CREATE TABLE IF NOT EXISTS public.refresh_tokens (
    id              BIGSERIAL PRIMARY KEY,
    token_id        VARCHAR(255)    NOT NULL UNIQUE,
    user_id         BIGINT          NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    token_hash      VARCHAR(255)    NOT NULL,
    expires_at      TIMESTAMPTZ      NOT NULL,
    revoked_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    last_used_at    TIMESTAMPTZ,
    ip_address      INET,
    user_agent      TEXT,
    
    
    INDEX idx_token_id (token_id),
    INDEX idx_user_id (user_id),
    INDEX idx_expires_at (expires_at),
    INDEX idx_revoked_at (revoked_at)
);