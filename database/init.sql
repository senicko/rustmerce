CREATE DATABASE rustmerce;

CREATE TABLE products (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    price FLOAT NOT NULL
);

CREATE TABLE assets (
    id SERIAL PRIMARY KEY,
    url TEXT NOT NULL,
    product_id INT NOT NULL,
    CONSTRAINT fk_product FOREIGN KEY (product_id) REFERENCES products(id)
);