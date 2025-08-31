-- Add migration script here
CREATE TABLE domains (
    id SERIAL PRIMARY KEY,
    name TEXT UNIQUE NOT NULL
);