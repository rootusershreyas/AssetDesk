-- Test-only schema until A and B confirm the shared migrations.
CREATE TABLE users (id uuid PRIMARY KEY DEFAULT gen_random_uuid());
CREATE SEQUENCE incident_number START WITH 1001;
CREATE SEQUENCE request_number START WITH 1001;
CREATE TABLE tickets (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    requester_id uuid NOT NULL REFERENCES users,
    category_id uuid NOT NULL,
    ticket_type text NOT NULL,
    ticket_number text NOT NULL UNIQUE,
    priority text NOT NULL,
    subject text NOT NULL,
    description text NOT NULL,
    custom_concern text,
    status text NOT NULL DEFAULT 'open',
    version bigint NOT NULL DEFAULT 1,
    FOREIGN KEY (category_id, ticket_type) REFERENCES categories (id, ticket_type)
);
CREATE TABLE ticket_events (
    ticket_id uuid NOT NULL REFERENCES tickets,
    actor_id uuid NOT NULL REFERENCES users,
    event_type text NOT NULL,
    resulting_version bigint NOT NULL,
    UNIQUE (ticket_id, resulting_version)
);
