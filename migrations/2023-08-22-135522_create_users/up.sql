-- Create the users table
CREATE TABLE users (
                       id SERIAL PRIMARY KEY,
                       email VARCHAR(100) NOT NULL,
                       password VARCHAR(100) NOT NULL,
                       fullname VARCHAR(100) NOT NULL,
                       role_id INT REFERENCES roles(id) NOT NULL
);