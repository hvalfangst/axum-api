-- Delete data from the 'roles' table
DELETE FROM roles
WHERE role_name IN ('READER', 'WRITER', 'EDITOR', 'ADMIN');