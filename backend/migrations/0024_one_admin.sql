CREATE UNIQUE INDEX IF NOT EXISTS one_admin_idx ON users (role) WHERE role = 'admin';
