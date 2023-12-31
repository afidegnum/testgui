use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::fmt;
use std::ptr::null;
use std::sync::mpsc::Sender;
use std::{thread, time};
use tokio::runtime::Runtime;
use tokio_postgres::tls::NoTlsStream;
use tokio_postgres::{Client, Connection, Error as TokioError, NoTls, Row, Socket};

// Define a custom error enum
#[derive(Debug)]
pub enum MetadataError {
    JsonResponseNotFound,
    RowNotFound,
    // PostgresError(PostgresError),
    TokioError(TokioError),
}

impl fmt::Display for MetadataError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MetadataError::JsonResponseNotFound => write!(f, "No JSON response found"),
            MetadataError::RowNotFound => write!(f, "No row found in the result"),
            // MetadataError::PostgresError(ref err) => err.fmt(f),
            MetadataError::TokioError(ref err) => err.fmt(f),
        }
    }
}

impl From<tokio_postgres::Error> for MetadataError {
    fn from(error: tokio_postgres::Error) -> MetadataError {
        MetadataError::TokioError(error)
    }
}

impl Error for MetadataError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            MetadataError::JsonResponseNotFound | MetadataError::RowNotFound => None,
            // MetadataError::PostgresError(ref err) => Some(err),
            MetadataError::TokioError(ref err) => Some(err),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Table {
    pub table: serde_json::Value,
}

impl From<Row> for Table {
    fn from(row: Row) -> Self {
        Self { table: row.get(0) }
    }
}

pub async fn get_metadata(
    schema: String,
    ctx: egui::Context,
    sender: Sender<crate::app::TaskMessage>,
) -> Result<(), MetadataError> {
    let qry = r#"
        select row_to_json(Tb) as tbls from(
        SELECT T."table_name", (select json_agg(col) from (
        SELECT "column_name", udt_name, is_nullable, data_type, column_default FROM
        information_schema.columns WHERE table_name = T."table_name" AND column_default NOT LIKE 'nextval%%'
        ) col ) as cols,

        (select json_agg(colx) from (
        SELECT "column_name", udt_name, is_nullable, data_type, column_default FROM
        information_schema.columns WHERE table_name = T."table_name"
        ) colx ) as identifiable_columns,

        (select json_agg(colr) from (
        SELECT
            tc.table_name,
            kcu.column_name,
            ccu.table_name AS foreign_table_name,
            ccu.column_name AS foreign_column_name
        FROM
            information_schema.table_constraints AS tc
            JOIN information_schema.key_column_usage AS kcu
              ON tc.constraint_name = kcu.constraint_name
              AND tc.table_schema = kcu.table_schema
            JOIN information_schema.constraint_column_usage AS ccu
              ON ccu.constraint_name = tc.constraint_name
              AND ccu.table_schema = tc.table_schema
        WHERE tc.constraint_type = 'FOREIGN KEY' AND tc.table_name=T."table_name"
        ) colr ) as relates
        FROM information_schema.tables as T
        WHERE table_schema=$1) Tb;
    "#;
    let (client, connection) = tokio_postgres::connect(
        "postgresql://postgres:chou1979@localhost/authenticate",
        NoTls,
    )
    .await
    .unwrap();
    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    let conn = tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let stmt = client.prepare(qry).await?;
    let rows = client.query(&stmt, &[&schema]).await?;

    let finalized: Vec<Table> = rows.into_iter().map(Table::from).collect();
    conn.abort();
    // println!("{:#?}", finalized);

    let gen_function = move |diagram: &mut crate::app::Diagram| {
        diagram.tables = finalized
        //from here you have access to the diagram!
        //now you can put in the code to tell it what to do with the metadata
    };
    // sender.send(Box::new(gen_function)).unwrap();
    sender
        .send(crate::app::TaskMessage::Generic(Box::new(gen_function)))
        .unwrap();
    ctx.request_repaint();

    Ok(())
    // Execute the query and collect the results
    // let rows = client.query(query, &[&schema]).await;
}
