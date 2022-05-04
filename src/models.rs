use super::schema::{products, variants};

use diesel::mysql::MysqlConnection;
use diesel::result::Error;
use diesel::{BelongingToDsl, GroupedBy, QueryDsl, RunQueryDsl};
use diesel_demo::models::{
    NewCompleteProduct, NewProduct, NewVariant, NewVariantValue, Product, ProductVariant, Variant,
};

use diesel::Connection;
use diesel_demo::{establish_connection, establish_connection_test};

use diesel::query_dsl::QueryDsl;
use serde::{Deserialize, Serialize};

use diesel_demo::models::*;

#[derive(Insertable, Debug)]
//Therefore, we need it to be Insertable, We also need to give it the name of our table.
#[table_name = "products"]
//This struct will be our model for inserting data in our database.
pub struct NewProduct {
    pub name: String,
    pub cost: f64,
    pub active: bool,
}

fn create_product(new_product: NewProduct, conn: &MysqlConnection) -> Result<usize, Error> {
    use diesel_demo::schema::products::dsl::*;
    //In this case, the target is products, which comes from the DSL module from the products schema.
    diesel::insert_into(products) //The insert_into function needs a target.
        .values(new_product) // We can use the values method to insert the data.
        .execute(conn)
}

//We create a function called create_product that requires an argument of the NewCompleteProduct type.
//This function will contain the data we need to create a product and its variants.
//In the code, we make a loop over the new variants and filter them by name to check if the variant was already created.
// We do this to avoid duplications and create it if necessary.
fn create_product(new_product: NewCompleteProduct, conn: &MysqlConnection) -> Result<i32> {
    use diesel_demo::schema::products::dsl::products;
    use diesel_demo::schema::products_variants::dsl::*;
    use diesel_demo::schema::variants::dsl::*;

    conn.transaction(|| {
        diesel::insert_into(products)
            .values(new_product.product)
            .execute(conn)?;

        let last_product_id: i32 = diesel::select(last_insert_rowid).first(conn)?;

        for new_variant in new_product.variants {
            let variants_result = variants
                .filter(name.eq(&new_variant.variant.name))
                .limit(1)
                .load::<Variant>(conn)?;

            let last_variant_id: i32 = match variants_result.first() {
                Some(variant) => variant.id,
                None => {
                    diesel::insert_into(variants)
                        .values(name.eq(&new_variant.variant.name))
                        .execute(conn)?;

                    diesel::select(last_insert_rowid).first(conn)?
                }
            };

            for new_value in new_variant.values {
                diesel::insert_into(products_variants)
                    .values((
                        product_id.eq(last_product_id),
                        variant_id.eq(last_variant_id),
                        value.eq(new_value),
                    ))
                    .execute(conn)?;
            }
        }
        Ok(last_product_id)
    })
}

//we add the models we need to create new variants.
#[derive(Identifiable, Queryable, Debug, Serialize, Deserialize)]
#[table_name = "variants"]
pub struct Variant {
    pub id: i32,
    pub name: String,
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "variants"]
pub struct NewVariant {
    pub name: String,
}

use super::schema::products_variants;

#[derive(Insertable, Debug)]
#[table_name = "products_variants"]
pub struct NewProductVariant {
    pub product_id: i32,
    pub variant_id: i32,
    pub value: Option<String>,
}
//We might need additional models that have a different purpose and that aren’t connected to a table for our business logic.
#[derive(Clone)]
pub struct NewVariantValue {
    pub variant: NewVariant,
    pub values: Vec<Option<String>>,
}

pub struct NewCompleteProduct {
    pub product: NewProduct,
    pub variants: Vec<NewVariantValue>,
}

//we’re going to write the code needed to create variants related to our products.
#[macro_use]
extern crate diesel;

use anyhow::Result;
use diesel::mysql::MysqlConnection;
use diesel::query_dsl::QueryDsl;
use diesel::Connection;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
use diesel_demo::models::*;

//allows us to use an SQL function in our code.
//we need a function called last_insert_rowid to get the last id inserted.
no_arg_sql_function!(last_insert_rowid, diesel::sql_types::Integer);

//The first model, Product, will be used to get data from the database.
//It needs to be serialized, so we use the Serialize and Deserialize directives.
//We also need to debug it, so we use Debug for our testing purposes.
#[derive(Queryable, Debug, Serialize, Deserialize)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub cost: f64,
    pub active: bool,
}
//write the code to get a list of products.
//will return 10 products. We can modify it to return any number we want.
fn list_products(
    conn: &SqliteConnection,
) -> Result<Vec<(Product, Vec<(ProductVariant, Variant)>)>, Error> {
    use diesel_demo::schema::products::dsl::products;
    use diesel_demo::schema::variants::dsl::variants;

    let products_result = products.limit(10).load::<Product>(conn)?;
    let variants_result = ProductVariant::belonging_to(&products_result)
        .inner_join(variants)
        .load::<(ProductVariant, Variant)>(conn)?
        .grouped_by(&products_result);
    let data = products_result
        .into_iter()
        .zip(variants_result)
        .collect::<Vec<_>>();

    Ok(data)
}

fn main() {
    println!("The products are: {:#?}", list_products());
}
//We create three products and assert that our function returns what we’re expecting.
//We’re serializing the results because it’s easier for our comparing purposes.

fn show_product(
    id: i32,
    conn: &SqliteConnection,
) -> Result<(Product, Vec<(ProductVariant, Variant)>), Error> {
    use diesel_demo::schema::products::dsl::products;
    use diesel_demo::schema::variants::dsl::variants;
    //We use first as a translation for LIMIT 1 SQL clause to find the product with the required id.
    let product_result = products.find(id).get_result::<Product>(conn)?;
    //We use get_result instead of first because the return type has the trait BelongingToDsl implemented.
    let variants_result = ProductVariant::belonging_to(&product_result)
        .inner_join(variants)
        .load::<(ProductVariant, Variant)>(conn)?;

    Ok((product_result, variants_result))
}

fn search_products(
    search: String,
    conn: &MysqlConnection,
) -> Result<Vec<(Product, Vec<(ProductVariant, Variant)>)>, Error> {
    use diesel_demo::schema::products::dsl::*;
    use diesel_demo::schema::variants::dsl::variants;

    let pattern = format!("%{}%", search);
    let products_result = products
        .filter(name.like(pattern))
        //The only difference is filter, which translates to an SQL Where clause.
        .load::<Product>(conn)?;
    let variants_result = ProductVariant::belonging_to(&products_result)
        .inner_join(variants)
        .load::<(ProductVariant, Variant)>(conn)?
        .grouped_by(&products_result);
    let data = products_result
        .into_iter()
        .zip(variants_result)
        .collect::<Vec<_>>();

    Ok(data)
}

#[test]
fn create_product_test() {
    let connection = establish_connection_test();
    connection.test_transaction::<_, Error, _>(|| {
        create_product(
            NewCompleteProduct {
                product: NewProduct {
                    name: "boots".to_string(),
                    cost: 13.23,
                    active: true,
                },
                variants: vec![NewVariantValue {
                    variant: NewVariant {
                        name: "size".to_string(),
                    },
                    values: vec![
                        Some(12.to_string()),
                        Some(14.to_string()),
                        Some(16.to_string()),
                        Some(18.to_string()),
                    ],
                }],
            },
            &connection,
        )
        .unwrap();

        assert_eq!(
            serde_json::to_string(&list_products(&connection).unwrap()).unwrap(),
            serde_json::to_string(&vec![(
                Product {
                    id: 1,
                    name: "boots".to_string(),
                    cost: 13.23,
                    active: true
                },
                vec![
                    (Some(12.to_string()), "size".to_string()),
                    (Some(14.to_string()), "size".to_string()),
                    (Some(16.to_string()), "size".to_string()),
                    (Some(18.to_string()), "size".to_string())
                ]
            ),])
            .unwrap()
        );

        Ok(())
    });
}

#[test]
fn test_list_products() {
    //establish_connection_test to connect to our test database.
    let connection = establish_connection_test();
    connection.test_transaction::<_, Error, _>(|| {
        //test_transaction, which is very useful because it doesn’t commit to the database.
        //This allows us to have a clean database every time we run our tests.
        let variants = vec![NewVariantValue {
            variant: NewVariant {
                name: "size".to_string(),
            },
            values: vec![
                Some(12.to_string()),
                Some(14.to_string()),
                Some(16.to_string()),
                Some(18.to_string()),
            ],
        }];

        create_product(
            NewCompleteProduct {
                product: NewProduct {
                    name: "boots".to_string(),
                    cost: 13.23,
                    active: true,
                },
                variants: variants.clone(),
            },
            &connection,
        )
        .unwrap();
        create_product(
            NewCompleteProduct {
                product: NewProduct {
                    name: "high heels".to_string(),
                    cost: 20.99,
                    active: true,
                },
                variants: variants.clone(),
            },
            &connection,
        )
        .unwrap();
        create_product(
            NewCompleteProduct {
                product: NewProduct {
                    name: "running shoes".to_string(),
                    cost: 10.99,
                    active: true,
                },
                variants: variants.clone(),
            },
            &connection,
        )
        .unwrap();

        let variants_result = |start_id, for_product_id| {
            vec![
                (
                    ProductVariant {
                        id: start_id + 1,
                        variant_id: 1,
                        product_id: for_product_id,
                        value: Some("12".to_string()),
                    },
                    Variant {
                        id: 1,
                        name: "size".to_string(),
                    },
                ),
                (
                    ProductVariant {
                        id: start_id + 2,
                        variant_id: 1,
                        product_id: for_product_id,
                        value: Some("14".to_string()),
                    },
                    Variant {
                        id: 1,
                        name: "size".to_string(),
                    },
                ),
                (
                    ProductVariant {
                        id: start_id + 3,
                        variant_id: 1,
                        product_id: for_product_id,
                        value: Some("16".to_string()),
                    },
                    Variant {
                        id: 1,
                        name: "size".to_string(),
                    },
                ),
                (
                    ProductVariant {
                        id: start_id + 4,
                        variant_id: 1,
                        product_id: for_product_id,
                        value: Some("18".to_string()),
                    },
                    Variant {
                        id: 1,
                        name: "size".to_string(),
                    },
                ),
            ]
        };

        assert_eq!(
            serde_json::to_string(&list_products(&connection).unwrap()).unwrap(),
            serde_json::to_string(&vec![
                (
                    Product {
                        id: 1,
                        name: "boots".to_string(),
                        cost: 13.23,
                        active: true
                    },
                    variants_result(0, 1)
                ),
                (
                    Product {
                        id: 2,
                        name: "high heels".to_string(),
                        cost: 20.99,
                        active: true
                    },
                    variants_result(4, 2)
                ),
                (
                    Product {
                        id: 3,
                        name: "running shoes".to_string(),
                        cost: 10.99,
                        active: true
                    },
                    variants_result(8, 3)
                )
            ])
            .unwrap()
        );

        Ok(())
    });
}

#[test]
fn show_product_test() {
    let connection = establish_connection_test();
    connection.test_transaction::<_, Error, _>(|| {
        let product_id = create_product(
            NewCompleteProduct {
                product: NewProduct {
                    name: "boots".to_string(),
                    cost: 13.23,
                    active: true,
                },
                variants: vec![NewVariantValue {
                    variant: NewVariant {
                        name: "size".to_string(),
                    },
                    values: vec![
                        Some(12.to_string()),
                        Some(14.to_string()),
                        Some(16.to_string()),
                        Some(18.to_string()),
                    ],
                }],
            },
            &connection,
        )
        .unwrap();

        assert_eq!(
            serde_json::to_string(&show_product(product_id, &connection).unwrap()).unwrap(),
            serde_json::to_string(&(
                Product {
                    id: 1,
                    name: "boots".to_string(),
                    cost: 13.23,
                    active: true
                },
                vec![
                    (
                        ProductVariant {
                            id: 1,
                            variant_id: 1,
                            product_id: 1,
                            value: Some("12".to_string(),),
                        },
                        Variant {
                            id: 1,
                            name: "size".to_string(),
                        }
                    ),
                    (
                        ProductVariant {
                            id: 2,
                            variant_id: 1,
                            product_id: 1,
                            value: Some("14".to_string(),),
                        },
                        Variant {
                            id: 1,
                            name: "size".to_string(),
                        }
                    ),
                    (
                        ProductVariant {
                            id: 3,
                            variant_id: 1,
                            product_id: 1,
                            value: Some("16".to_string(),),
                        },
                        Variant {
                            id: 1,
                            name: "size".to_string(),
                        }
                    ),
                    (
                        ProductVariant {
                            id: 4,
                            variant_id: 1,
                            product_id: 1,
                            value: Some("18".to_string(),),
                        },
                        Variant {
                            id: 1,
                            name: "size".to_string(),
                        }
                    )
                ]
            ))
            .unwrap()
        );

        Ok(())
    });
}

#[test]
fn search_products_test() {
    let connection = establish_connection_test();
    connection.test_transaction::<_, Error, _>(|| {
        let variants = vec![NewVariantValue {
            variant: NewVariant {
                name: "size".to_string(),
            },
            values: vec![Some(12.to_string())],
        }];

        create_product(
            NewCompleteProduct {
                product: NewProduct {
                    name: "boots".to_string(),
                    cost: 13.23,
                    active: true,
                },
                variants: variants.clone(),
            },
            &connection,
        )
        .unwrap();
        create_product(
            NewCompleteProduct {
                product: NewProduct {
                    name: "high heels".to_string(),
                    cost: 20.99,
                    active: true,
                },
                variants: variants.clone(),
            },
            &connection,
        )
        .unwrap();
        create_product(
            NewCompleteProduct {
                product: NewProduct {
                    name: "running shoes".to_string(),
                    cost: 10.99,
                    active: true,
                },
                variants: variants.clone(),
            },
            &connection,
        )
        .unwrap();

        assert_eq!(
            serde_json::to_string(&search_products("shoes".to_string(), &connection).unwrap())
                .unwrap(),
            serde_json::to_string(&vec![(
                Product {
                    id: 3,
                    name: "running shoes".to_string(),
                    cost: 10.99,
                    active: true
                },
                vec![(
                    ProductVariant {
                        id: 3,
                        variant_id: 1,
                        product_id: 3,
                        value: Some("12".to_string(),),
                    },
                    Variant {
                        id: 1,
                        name: "size".to_string(),
                    }
                )]
            )])
            .unwrap()
        );

        Ok(())
    });
}
