-- Extend the disposable creation fixture for review and eligibility tests.
ALTER TABLE users ADD COLUMN role text NOT NULL DEFAULT 'employee',
                  ADD COLUMN account_status text NOT NULL DEFAULT 'active';
CREATE TABLE category_agents (
    category_id uuid REFERENCES categories,
    user_id uuid REFERENCES users,
    PRIMARY KEY (category_id, user_id)
);
ALTER TABLE tickets ADD COLUMN assignee_id uuid REFERENCES users,
                    ADD COLUMN reviewed_by uuid REFERENCES users,
                    ADD COLUMN reviewed_at timestamptz;
ALTER TABLE ticket_events ADD COLUMN new_assignee_id uuid REFERENCES users,
                          ADD COLUMN new_priority text;

ALTER TABLE ticket_events ADD COLUMN old_status text, ADD COLUMN new_status text;

ALTER TABLE tickets ADD COLUMN resolution text;
ALTER TABLE ticket_events ADD COLUMN resolution text;

ALTER TABLE ticket_events ADD COLUMN explanation text;
ALTER TABLE tickets ADD COLUMN root_cause text;
ALTER TABLE ticket_events ADD COLUMN root_cause text;
