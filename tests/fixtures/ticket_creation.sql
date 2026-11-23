-- Disposable test data defaults. Tables are created by the real migrations.
ALTER TABLE users ALTER COLUMN name SET DEFAULT 'Test user',
                  ALTER COLUMN email SET DEFAULT (gen_random_uuid()::text || '@example.test'),
                  ALTER COLUMN role SET DEFAULT 'employee';
