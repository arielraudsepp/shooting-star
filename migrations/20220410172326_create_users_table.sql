CREATE TABLE users(
       id SERIAL,
       PRIMARY KEY (id),
       username TEXT NOT NULL UNIQUE,
       password_hash TEXT NOT NULL
);
