CREATE TABLE skills_tracker(
       id uuid NOT NULL,
       PRIMARY KEY (id),
       skill_name TEXT NOT NULL,
       completed BOOL NOT NULL,
       entered_at timestamptz NOT NULL
);
