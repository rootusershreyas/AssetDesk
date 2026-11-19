DROP INDEX categories_ticket_type_name_idx;

CREATE UNIQUE INDEX categories_name_idx
ON categories (lower(name));
