create table boards (
    id integer primary key,
    name text not null unique,
    slug text not null unique,
    description text not null,
    active boolean not null default 1
);

create table threads (
    id integer primary key,
    board_id integer not null references boards(id),
    created_at integer not null default (unixepoch()),
    bumped_at integer not null default (unixepoch()),
    active boolean not null default 1,
    image_id integer references images(id),
    title text not null,
    body text not null
);

create table posts (
    id integer primary key,
    thread_id integer not null references threads(id),
    created_at integer not null default (unixepoch()),
    bumped_at integer not null default (unixepoch()),
    active boolean not null default 1,
    image_id integer references images(id),
    screen_name text,
    body text not null,
    in_reply_to integer references posts(id)
);

create table images (
    id integer primary key,
    uri text not null,
    sha256 text not null unique,
    mime_type text not null,
    original_name text
);

create index idx_threads_board_bumped on threads(board_id, bumped_at desc);
create index idx_posts_thread on posts(thread_id);
