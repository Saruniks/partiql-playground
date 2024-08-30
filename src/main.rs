mod schema;

use regex::Regex;
use schema::*;

use aws_sdk_dynamodb::{
    error::SdkError,
    operation::create_table::CreateTableError,
    types::{
        builders::{AttributeDefinitionBuilder, KeySchemaElementBuilder},
        AttributeDefinition, AttributeValue, KeySchemaElement, KeyType, ProvisionedThroughput,
        ScalarAttributeType,
    },
};

use diesel::insert_into;
use diesel::prelude::*;
use diesel::sql_types::Text;

#[tokio::main]
async fn main() {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_dynamodb::Client::new(&config);

    create_table_if_doesnt_exist(&client).await;
    // write_to_table(&client).await;
    // write_to_table_partiql(&client).await;

    // Convert Diesel query to a string and execute in DynamoDB
    write_to_table_with_diesel_query(&client).await;
}

async fn create_table_if_doesnt_exist(client: &aws_sdk_dynamodb::Client) {
    match client
        .create_table()
        .table_name("test_table")
        .provisioned_throughput(
            // Set Free Tier-compliant Read and Write Capacity Units
            ProvisionedThroughput::builder()
                .read_capacity_units(1)
                .write_capacity_units(1)
                .build()
                .unwrap(),
        )
        .key_schema(
            KeySchemaElementBuilder::default()
                .attribute_name("test_schema_key".to_string())
                .key_type(KeyType::Hash)
                // .attribute_type(ScalarAttributeType::S)
                .build()
                .unwrap(),
        )
        .attribute_definitions(
            AttributeDefinitionBuilder::default()
                .attribute_name("test_schema_key".to_string())
                .attribute_type(ScalarAttributeType::S)
                .build()
                .unwrap(),
        )
        .send()
        .await
    {
        Ok(_) => println!("Table created"),
        Err(err) => match err {
            SdkError::ServiceError(err) => match err.into_err() {
                CreateTableError::ResourceInUseException(err) => {
                    println!("{err}")
                }
                err => panic!("{err}"),
            },
            _ => panic!("{err}"),
        },
    };
}

async fn write_to_table(client: &aws_sdk_dynamodb::Client) {
    client
        .put_item()
        .table_name("test_table")
        .item(
            "test_schema_key",
            AttributeValue::S("test_item_value_4".to_string()),
        )
        .send()
        .await
        .unwrap();
}

async fn write_to_table_partiql(client: &aws_sdk_dynamodb::Client) {
    client
        .execute_statement()
        .statement("INSERT INTO test_table VALUE {'test_schema_key': ?}")
        .parameters(AttributeValue::S("test_item_value_partiql".to_string()))
        .send()
        .await
        .unwrap();
}

async fn write_to_table_with_diesel_query(client: &aws_sdk_dynamodb::Client) {
    // Construct a Diesel insert query
    let diesel_insert_query = insert_into(test_table::table)
        .values(test_table::test_schema_key.eq("test_item_value_partiql_diesel_rs_4"));

    // Convert the Diesel query to an SQL string with debug_query
    let query_string = diesel::debug_query::<diesel::pg::Pg, _>(&diesel_insert_query).to_string();

    println!("Before: {}", query_string);

    // Parse and adapt the SQL string to DynamoDB format
    let (dynamodb_query, parameters) = parse_and_adapt_query(&query_string);

    println!("After {}", dynamodb_query);

    // Execute the DynamoDB PartiQL query
    match client
        .execute_statement()
        .statement(&dynamodb_query) // Pass the modified query string
        .set_parameters(Some(parameters)) // Pass the extracted parameters
        .send()
        .await
    {
        Ok(_) => println!("Item inserted successfully."),
        Err(e) => println!("Error executing PartiQL query: {:?}", e),
    }
}

// Function to parse and adapt SQL from Diesel to DynamoDB PartiQL
fn parse_and_adapt_query(diesel_query: &str) -> (String, Vec<AttributeValue>) {
    // Use regex to extract the `-- binds: [...]` parameters
    let re_binds = Regex::new(r"-- binds: \[(.*?)\]").unwrap();

    // Find and extract parameter list from the `-- binds` comment
    let params_str = re_binds
        .captures(diesel_query)
        .and_then(|caps| caps.get(1).map(|m| m.as_str()))
        .expect("No parameters found in Diesel debug output!");

    // Parse parameters into individual values
    let params: Vec<AttributeValue> = params_str
        .split(',')
        .map(|s| {
            let trimmed = s.trim().trim_matches('"').to_string();
            AttributeValue::S(trimmed)
        })
        .collect();

    // Adapt the query string to a format that DynamoDB understands
    // let mut query_string = diesel_query
    //     .split("--")
    //     .next()
    //     .unwrap_or("")
    //     .trim()
    //     .to_string();

    // // Replace PostgreSQL placeholders ($1, $2, etc.) with DynamoDB's placeholder (?)
    // let re_placeholders = Regex::new(r"\$\d+").unwrap();

    // Correct the `VALUES` to `VALUE` and use JSON-like syntax for DynamoDB PartiQL
    let query_string = "INSERT INTO test_table VALUE {'test_schema_key': ?}".to_string(); // Oh no

    (query_string, params)
}
