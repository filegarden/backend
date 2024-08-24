CREATE TABLE users (
    created_at timestamptz NOT NULL DEFAULT now(),
    id bytea PRIMARY KEY,
    email varchar(254) UNIQUE,
    name text NOT NULL,
    birthdate date NOT NULL,
    password_hash text NOT NULL,
    totp_secret bytea,
);

CREATE TABLE unverified_emails (
    created_at timestamptz NOT NULL DEFAULT now(),
    user_id bytea PRIMARY KEY REFERENCES users (id) ON DELETE CASCADE,
    email varchar(254) NOT NULL,
    token_hash bytea NOT NULL,
);

CREATE TABLE password_resets (
    created_at timestamptz NOT NULL DEFAULT now(),
    user_id bytea PRIMARY KEY REFERENCES users (id) ON DELETE CASCADE,
    token_hash bytea NOT NULL,
);

CREATE TABLE sessions (
    created_at timestamptz NOT NULL DEFAULT now(),
    accessed_at timestamptz NOT NULL DEFAULT now(),
    token_hash bytea PRIMARY KEY,
    user_id bytea NOT NULL REFERENCES users (id) ON DELETE CASCADE,
);

CREATE INDEX sessions_by_accessed_at ON sessions (accessed_at);
CREATE INDEX sessions_by_user_id ON sessions (user_id);

CREATE TYPE encoding AS ENUM ('br');

CREATE TABLE files (
    created_at timestamptz NOT NULL DEFAULT now(),
    modified_at timestamptz NOT NULL DEFAULT now(),
    id bytea PRIMARY KEY,
    name text NOT NULL,
    owner_id bytea NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    parent_id_path bytea[] NOT NULL,
    parent_name_path text[] NOT NULL,
    shared boolean NOT NULL DEFAULT FALSE,
    parts integer NOT NULL DEFAULT 1,
    size bigint NOT NULL,
    encoded_size bigint NOT NULL,
    encoding encoding,
    type text NOT NULL,

    UNIQUE (owner_id, parent_name_path, name),
);

CREATE INDEX files_by_id_path ON files (owner_id, parent_id_path, id);

CREATE TABLE folders (
    created_at timestamptz NOT NULL DEFAULT now(),
    id bytea PRIMARY KEY,
    name text NOT NULL,
    owner_id bytea NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    parent_id_path bytea[] NOT NULL UNIQUE,
    parent_name_path text[] NOT NULL UNIQUE,
    share_key bytea UNIQUE,

    UNIQUE (owner_id, parent_name_path, name),
);

CREATE INDEX folders_by_id_path ON folders (owner_id, parent_id_path, id);
