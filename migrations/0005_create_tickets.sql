CREATE TABLE category_agents (
    category_id uuid NOT NULL REFERENCES categories,
    user_id uuid NOT NULL REFERENCES users,
    PRIMARY KEY (category_id, user_id)
);
CREATE SEQUENCE incident_number START WITH 1001;
CREATE SEQUENCE request_number START WITH 1001;
CREATE TABLE tickets (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    requester_id uuid NOT NULL REFERENCES users,
    category_id uuid NOT NULL,
    ticket_type text NOT NULL,
    ticket_number text NOT NULL UNIQUE,
    priority text NOT NULL CHECK (priority IN ('p1','p2','p3','p4')),
    subject text NOT NULL CHECK (char_length(btrim(subject)) BETWEEN 1 AND 160),
    description text NOT NULL CHECK (char_length(btrim(description)) BETWEEN 1 AND 1000),
    custom_concern text,
    status text NOT NULL DEFAULT 'open'
        CHECK (status IN ('open','in_progress','awaiting_employee_confirmation','resolved')),
    version bigint NOT NULL DEFAULT 1 CHECK (version > 0),
    assignee_id uuid REFERENCES users,
    reviewed_by uuid REFERENCES users,
    reviewed_at timestamptz,
    resolution text,
    root_cause text,
    created_at timestamptz NOT NULL DEFAULT now(),
    FOREIGN KEY (category_id, ticket_type) REFERENCES categories (id, ticket_type)
);
CREATE INDEX tickets_requester_created_idx ON tickets(requester_id, created_at DESC, id);
CREATE TABLE ticket_events (
    ticket_id uuid NOT NULL REFERENCES tickets,
    actor_id uuid NOT NULL REFERENCES users,
    event_type text NOT NULL,
    resulting_version bigint NOT NULL CHECK (resulting_version > 0),
    new_assignee_id uuid REFERENCES users,
    new_priority text,
    old_status text,
    new_status text,
    resolution text,
    root_cause text,
    explanation text,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (ticket_id, resulting_version)
);
