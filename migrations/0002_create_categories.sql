CREATE TABLE categories (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        ticket_type TEXT NOT NULL
        CHECK (ticket_type IN ('incident', 'service_request')),
        name TEXT NOT NULL
                        CHECK (name = btrim(name) AND char_length(name) BETWEEN 1 AND 80),
        description TEXT CHECK (char_length(description) <= 1000),
        is_active BOOLEAN NOT NULL DEFAULT true,
        created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
        
        UNIQUE (id, ticket_type)
);

CREATE UNIQUE INDEX categories_ticket_type_name_idx
ON categories (ticket_type, lower(name));




