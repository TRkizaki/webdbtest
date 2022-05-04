CREATE TABLE variants (
   id INTEGER PRIMARY KEY NOT NULL, 
   name VARCHAR(255) NOT NULL
);

CREATE TABLE products_variants (
   id INTEGER PRIMARY KEY NOT NULL, 
   variant_id INTEGER NOT NULL,
   product_id INTEGER NOT NULL,
   value VARCHAR(255),
   FOREIGN KEY(variant_id) REFERENCES variants(id),
   FOREIGN KEY(product_id) REFERENCES products(id)   
);