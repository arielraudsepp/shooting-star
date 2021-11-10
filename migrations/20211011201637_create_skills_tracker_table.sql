CREATE TABLE skills(
       id uuid NOT NULL,
       PRIMARY KEY (id),
       name TEXT NOT NULL,
       completed BOOL NOT NULL,
       created_at timestamptz NOT NULL
);
