//use chrono::Utc;
use futures_util::{stream, Stream, StreamExt};
use klickhouse::{ConnectionManager, Client, ClientOptions, Progress, Row, Uuid};
//use sqlx::pool;
use crate::utils; 
use std::time::Instant;
use tokio;
use env_logger;
use log;
use serde::Serialize;


// TODO:: 增加一个COLUMNA会增加许多开销么？
// TODO:: 直接插入parquet?
// TODO::调查truncate ddl --done

//pub struct MyUserData {
//    id: Uuid,
//    user_data: String,
//    created_at: DateTime,
//}
#[derive(Debug, Default, Row, Serialize)]
pub struct Course {
    pub cid: i32,
    pub c_name: String,
    pub tid: i32,
}

#[tokio::main]
pub async fn klickhouse_sampl(num_records:i32, pool_connection:bool) {

    env_logger::Builder::new()
        .parse_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    let ck_client_str = utils::read_config_json("db_config.json", "rui_klickhouse").unwrap();
    println!("connection params: {:?}", ck_client_str);
    let new_conn = ClientOptions{
        username: ck_client_str[1].clone(),
        password: ck_client_str[2].clone(),
        default_database: ck_client_str[3].clone(),
        tcp_nodelay: true,
    };
    let tcp_url  = ck_client_str[0].clone();
    // https://github.com/Protryon/klickhouse/issues/44
    //println!("url: {}", tcp_url);

    let manager = ConnectionManager::new(tcp_url, new_conn)
                                        .await
                                        .unwrap();
    let pool = bb8::Pool::builder()
        .max_size(15)
        .build(manager)
        .await
            .unwrap(); 
    let client = pool.get().await.unwrap();
    
        
    //let client = Client::connect(tcp_url, new_conn)
    //        .await
    //        .unwrap();
    println!("connection done...");

    
    // Retrieve and display query progress events 
    /*
    let mut progress = client.subscribe_progress();
    let progress_task = tokio::task::spawn(async move {
        let mut current_query = Uuid::nil();
        let mut progress_total = Progress::default();
        while let Ok((query, progress)) = progress.recv().await {
            if query != current_query {
                progress_total = Progress::default();
                current_query = query;
            }
            progress_total += progress;
            println!(
                "Progress on query {}: {}/{} {:.2}%",
                query,
                progress_total.read_rows,
                progress_total.new_total_rows_to_read,
                100.0 * progress_total.read_rows as f64
                    / progress_total.new_total_rows_to_read as f64
            );
        }
    });
    
 */
    // Prepare table
    let table_name = "test.course";
    let drop_ddl = format!("DROP TABLE IF EXISTS {}", table_name);
    let create_ddl = format!("CREATE TABLE IF NOT EXISTS {} (
        cid Int32 NOT NULL 
        , c_name String
        , tid Int32 DEFAULT 4
        ) 
        ENGINE = MergeTree()
        PRIMARY KEY (cid)
        ", table_name);
    let truncate_ddl = format!("TRUNCATE TABLE IF EXISTS {}", table_name);
    client
        .execute(drop_ddl)
        .await
        .unwrap();
    client
        .execute(create_ddl)
        .await
        .unwrap();
    client
        .execute(truncate_ddl)
        .await
        .unwrap();
    println!("truncate done...");

    // Insert rows Block  
    // insert batch by batch
    /// Wrapper over [`Client::insert_native`] to send a single block.
    /// Make sure any query you send native data with has a `format native` suffix.
    
    let start = Instant::now();
    let mut rows = Vec::new();
    for idx in 1..num_records {
        rows.push(Course {
            cid: idx
            , c_name: utils::generate_random_utf8_string(10)
            , tid: 77
        })
    };
    client
        .insert_native_block("INSERT INTO test.course FORMAT native", rows)
        .await
        .unwrap();
    let duration = start.elapsed();
    println!("insert block done, using time: {:?}", duration);

    // Insert rows by row
    let start_2 = Instant::now();
    for jdx in 0..num_records { 
        let new_value = utils::generate_random_utf8_string(10);
        let insert_query = format!("INSERT INTO test.course (*)
        values ({jdx},'{new_value}', 88)");
        client
            .execute(insert_query)
            .await
            .unwrap();
    }
    let duration = start_2.elapsed();
    println!("insert rows done, using time: {:?}", duration);

    // OUTPUT ROWS using parquet
    //let output_pq_query = format!("SELECT * 
    //        FROM test.course
    //        INTO OUTFILE 'C:\\Users\\dongx\\test_xu\\data\\laaa.parquet' FORMAT Parquet");
//
    //client
    //    .execute(output_pq_query)
    //    .await
    //    .unwrap();
    //println!("output parquet done...");

    // Read back rows
    let check_query = format!("SELECT * FROM {} where cid >= {}", table_name, num_records-5);
    let mut all_rows = client
        .query::<Course>(check_query)
        .await
        .unwrap();
    while let Some(row) = all_rows.next().await {
        let row = row.unwrap();
        println!("row received '{}': {:?}", row.cid, row);
    }

    // Drop the client so that the progress task finishes.
    
    drop(client);
    println!("client dropped...");
}