CREATE DATABASE rustmerce;

CREATE TABLE products (
    id SERIAL,
    name TEXT NOT NULL,
    price FLOAT NOT NULL,
);