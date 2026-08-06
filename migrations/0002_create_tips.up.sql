CREATE TABLE tips (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    creator_username TEXT NOT NULL REFERENCES creators(username),
    amount TEXT NOT NULL,
    transaction_hash TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
