ALTER TABLE categories
ADD CONSTRAINT categories_incident_excludes_hardware
CHECK (
    ticket_type <> 'incident'
    OR lower(
        btrim(regexp_replace(name, '[[:space:]]+', ' ', 'g'))
    ) NOT IN ('hardware', 'hardware issue', 'hardware request')
);

INSERT INTO categories (ticket_type, name, description)
VALUES (
    'incident',
    'Others',
    'IT incidents outside the listed categories.'
)
ON CONFLICT (ticket_type, lower(name)) DO NOTHING;
