
use crate::utils; 
use clickhouse::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use anyhow::{Result, Error};
//use serde::Deserialize;
use clickhouse::Row;
use std::time::Duration;
use clickhouse::sql::Identifier;
use rustls;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio;
use std::time::Instant;
use std::fmt;
//use crate::async_func::Course;
use clickhouse::insert::Insert;
use std::thread;

//pub struct Insert<T> {
//    state: InsertState,
//    buffer: BytesMut,
//    #[cfg(feature = "lz4")]
//    compression: Compression,
//    send_timeout: Option<Duration>,
//    end_timeout: Option<Duration>,
//    // Use boxed `Sleep` to reuse a timer entry, it improves performance.
//    // Also, `tokio::time::timeout()` significantly increases a future's size.
//    sleep: Pin<Box<Sleep>>,
//    _marker: PhantomData<fn() -> T>, // TODO: test contravariance.
//}


#[derive(Row, Deserialize, Serialize, Debug)]
pub struct Course {
    pub cid: i32,
    pub c_name: String,
    pub tid: i32,
}

struct MyInsert(clickhouse::insert::Insert<Course>);

impl fmt::Debug for MyInsert {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Insert")
    }
}

// link:https://cloud.tencent.com/developer/article/1814306

#[tokio::main]
pub async fn clickhouse_sampl(num_records:i32) -> Result<()> {
    rustls::crypto::ring::default_provider().install_default().expect("Failed to install rustls crypto provider");
    //println!("xixi");
    let ck_client_str = utils::read_config_json("db_config.json", "rui_clickhouse").unwrap();
    println!("connection params: {:?}", ck_client_str);


// client error: InvalidParams(http::Error(InvalidUri(InvalidAuthority)))
// ck NEED http style url as input such as, http://110.40.186.179:8123
// 数据对齐！
// 不存在的 TBL不可以用cursor.next()来读取数据!
    let ck_client = Client::default()
                .with_url(&ck_client_str[0])
                .with_user(&ck_client_str[1])
                .with_password(&ck_client_str[2]);
                        // https://clickhouse.com/docs/en/operations/settings/settings#async-insert
        //.with_option("async_insert", "1")
        // https://clickhouse.com/docs/en/operations/settings/settings#wait-for-async-insert
        //.with_option("wait_for_async_insert", "0");;

    // create & drop & truncate tbl DDL;
    let table_name = "test.course";
    let create_tbl_ddl = format!("CREATE TABLE IF NOT EXISTS {} (
        cid Int32 NOT NULL 
        , c_name String
        , tid Int32 DEFAULT 4
        ) 
        ENGINE = MergeTree()
        PRIMARY KEY (cid)
        ", table_name);
    let truncate_ddl = format!("TRUNCATE TABLE IF EXISTS {table_name}");
    ck_client.query(&create_tbl_ddl).execute().await.unwrap();
    ck_client.query(&truncate_ddl).execute().await.unwrap();
    println!("truncate done...");


    let start = Instant::now();
    let mut inserter = ck_client.inserter(&table_name)
                                    .unwrap()
                                    .with_timeouts(Some(Duration::from_secs(5))
                                    , Some(Duration::from_secs(20)));


    // INSERT records thru rs-Struct async but single thread
    for idx in 1..num_records {
        inserter.write(&Course { cid: idx, c_name: utils::generate_random_utf8_string(10), tid: 3 }); //.unwrap();
        let wtr_res = inserter.commit().await.unwrap();
        //println!("{:?}", wtr_res);
    }
    inserter.end().await.unwrap();
    //println!("{:?}", wtr_res);
    let duration = start.elapsed();
    println!("insert done, using time: {:?}", duration);

    //let output_pq_query = format!("SELECT * 
    //FROM test.course
    //INTO OUTFILE 'C:\\Users\\dongx\\test_xu\\data\\laaa.parquet' FORMAT Parquet");
//
    //ck_client.query(&output_pq_query).execute().await.unwrap();
    //println!("output parquet done...");
    
/* 
    // INSERT records thru async
    //.with_max_rows(750_000)
    //.with_period(Some(Duration::from_secs(15)));
    let mut handles = Vec::new();
    let m_inserter = Arc::new(Mutex::new(inserter));
    
    //let _ = Arc::try_unwrap(m_inserter);
    for idx in 1..num_records {
        let c_inserter = Arc::clone(&m_inserter); //.clone();

        let handle = tokio::spawn(async move {
            let mut de_inserter = c_inserter.lock().await; //.unwrap(); 
            // https://docs.rs/clickhouse/latest/src/clickhouse/inserter.rs.html#200-225
            de_inserter.write(&Course { cid: idx, c_name: utils::generate_random_utf8_string(10), tid: 3 });  //.unwrap();
            //let insert_res = de_inserter.commit().await.unwrap();
            //println!("insert_res: {:?}", insert_res);
        });
        
        handles.push(handle);
    }
    //Arc::try_unwrap(m_inserter).end().unwrap();
    //inserter.end().await.unwrap();
    
    match Arc::into_inner(m_inserter) {
        Some(inner_t) => {
            inner_t
            .into_inner()
            //.unwrap()
            .commit()
            .await
            .unwrap();
        }
        None => {
            eprintln!("error: None inner_T");
        }
    };
    
        //.commit()
        //.await
        //.unwrap();

    for handle in handles {
        handle.await.unwrap();
    }
*/

    //let inserter = Arc::try_unwrap(m_inserter).unwrap(); //.expect("Arc::try_unwrap failed").into_inner();
    //inserter;
    // End the inserter
    //let mut de_inserter = m_inserter.lock().await;
    // The Insert::end must be called to finalize the INSERT.
    // Otherwise, the whole INSERT will be aborted.
    //Arc::try_unwrap(m_inserter).unwrap().into_inner().unwrap().end().await.unwrap();


    // check the inserted data
    let mut cursor = ck_client
                .query("SELECT * from test.course limit 5")
                //.with_option("wait_end_of_query", "1")
                .fetch::<Course>()
                //.await
                .unwrap();

    while let Some(row) = cursor.next().await? { 
        println!("Stored data: {:?}", row);
    }

    

    println!("clickhouse done"); 
    Ok(())

}

