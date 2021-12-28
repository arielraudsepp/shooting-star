CREATE TABLE diary_entries(
       id SERIAL,
       PRIMARY KEY (id),
       entry_date DATE NOT NULL,
       created_at timestamptz NOT NULL
);
