CREATE TABLE IF NOT EXISTS companies
(
    id            uuid NOT NULL PRIMARY KEY,
    name          text NOT NULL,
    email         text NOT NULL UNIQUE,
    password_hash text NOT NULL
);

CREATE TYPE target AS
(
    age_from   integer,
    age_to     integer,
    country    text,
    categories text[]
);

CREATE TYPE promo_mode AS ENUM ('COMMON', 'UNIQUE');

CREATE TABLE IF NOT EXISTS promos
(
    id            uuid        NOT NULL PRIMARY KEY,
    company_id    uuid        NOT NULL REFERENCES companies (id) ON DELETE CASCADE,
    description   text        NOT NULL,
    image_url     text,
    target        target      NOT NULL,
    max_count     integer     NOT NULL,
    active_from   timestamptz NOT NULL,
    active_until  timestamptz NOT NULL,
    mode          promo_mode  NOT NULL,
    promo_common  text,
    promo_unique  text[]      NOT NULL,
    like_count    integer     NOT NULL,
    used_count    integer     NOT NULL,
    comment_count integer     NOT NULL,
    active        boolean     NOT NULL
);

CREATE TYPE user_target_settings AS
(
    age     integer,
    country text
);

CREATE TABLE IF NOT EXISTS users
(
    id            uuid                 NOT NULL PRIMARY KEY,
    name          text                 NOT NULL,
    surname       text                 NOT NULL,
    email         text                 NOT NULL,
    avatar_url    text,
    other         user_target_settings NOT NULL,
    password_hash text                 NOT NULL

);

CREATE TABLE IF NOT EXISTS likes
(
    user_id  uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    promo_id uuid NOT NULL REFERENCES promos (id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, promo_id)
);

CREATE TABLE IF NOT EXISTS activations
(
    user_id  uuid        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    promo_id uuid        NOT NULL REFERENCES promos (id) ON DELETE CASCADE,
    promo    text        NOT NULL,
    date     timestamptz NOT NULL,
    PRIMARY KEY (user_id, promo_id, promo, date)
);

CREATE TABLE IF NOT EXISTS comments
(
    id        uuid        NOT NULL PRIMARY KEY,
    author_id uuid        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    promo_id  uuid        NOT NULL REFERENCES promos (id) ON DELETE CASCADE,
    text      text        NOT NULL,
    date      timestamptz NOT NULL
);