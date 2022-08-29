-- structure

CREATE DATABASE rustmerce;

CREATE TABLE products (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    price FLOAT NOT NULL
    category_id INT,
    CONSTRAINT fk_category FOREIGN KEY (category_id) REFERENCES categories(id)
);

CREATE TABLE assets (
    id SERIAL PRIMARY KEY,
    filename TEXT NOT NULL,
    product_id INT NOT NULL,
    CONSTRAINT fk_product FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE
);

CREATE TABLE categories (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    parent_id INT,
    CONSTRAINT fk_parent FOREIGN KEY (parent_id) REFERENCES category(id) ON DELETE CASCADE
);

-- util procedures

-- get_category_tree returns id of the category and ids of all deeply nested sub-categories
CREATE FUNCTION get_category_tree(category_id int) RETURNS TABLE(id int) 
AS $$
    WITH RECURSIVE parent_category AS (
        SELECT id FROM categories WHERE id = $1
        UNION ALL
        SELECT c.id  FROM categories AS c, parent_category AS pc WHERE c.parent_id = pc.id
    ) SELECT * FROM parent_category;
$$ LANGUAGE SQL;