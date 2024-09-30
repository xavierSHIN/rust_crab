use sqlx::mysql::MySqlPoolOptions;
//use sqlx::MySql;
use std::env;
use sqlx::FromRow;
use std::sync::{Arc, Mutex};
use rand::{distributions::Alphanumeric,Rng};
use std::time::Instant;


//use console_subscriber;

// TODO:: pool到底是什么？
// todo:: 加入多线程之间的通信和修改？Arc & Mutex
// TODO:: 1)随机产生1w条数据，然后进行插入操作,比较异步和同步的速度
// TODO:: 2)_读取插入的1w条数据，然后进行update操作,比较异步和同步的速度
// TODO:: 3)_timer 如何实现？instant? runtime?


fn generate_random_utf8_string(length: usize) -> String {
    let s: String = rand::thread_rng()
    .sample_iter(&Alphanumeric)
    .take(length)
    .map(char::from)
    .collect();
    s
//println!("{}", s);
}


fn generate_random_i32() -> i32 {
    let mut rng = rand::thread_rng();
    let range = 0..1000000;
    rng.gen_range(range) as i32
}


#[derive(Debug, FromRow, Clone)]
pub struct Course {
    pub cid: i32,
    pub c_name: String,
    pub tid: i32,
}

#[derive(Debug, FromRow, Clone)]
struct Student {
    sid: i32,
    s_name: String,
    s_age: String,
    s_sex: i8
}

// set DATABASE_URL=mysql://user:password@host/database
#[tokio::main]
pub async fn sqlx_asymain(num_connector:u32, add_rows:u32, arc_mode:bool) -> Result<(), sqlx::Error> {
    // Set the database URL
    //console_subscriber::init();
    let ck_str = "clickhouse://default:@110.40.186.179:8123/test";
    let mysql_str = "mysql://root:714233@localhost/sg_data";
    env::set_var("DATABASE_URL", mysql_str);

    // Load the database URL from an environment variable
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    //
    let start = Instant::now();
    // Create a connection pool
    let pool = MySqlPoolOptions::new()
        .max_connections(num_connector)
        .connect(&database_url)
        .await?;
    //let pool_b = pool.clone();
    // Use the pool to interact with the database
    let (len,): (i64,) = sqlx::query_as("SELECT COUNT(*) from sg_data.course")
        .fetch_one(&pool)
        .await?;//.get(0_i8);
    println!("len: {:?}", len);
    let clean_res = sqlx::query("DELETE from sg_data.course where cid>=?")
        .bind(5)
        .execute(&pool)
        .await.unwrap(); 
    println!("clean_res: {:?}", clean_res);

    // async starts with settings
    let mut handles = Vec::new();
    let normal_pool = pool.clone();
    let arc_pool =  Arc::new(pool);

    for row_idx in 5..5+add_rows {
        //let mut courses_c = Arc::clone(&arc_courses);
        if arc_mode == true {
            let pool_c = Arc::clone(&arc_pool);
            // The problem is that the pool variable is being moved 
            // into the async block inside the loop
            // , which causes it to be unavailable in subsequent iterations.
            let handle  = tokio::spawn(async move {
                let new_course = Course {
                    cid:  row_idx as i32,
                    c_name: generate_random_utf8_string(5),
                    tid: 1,};
                let res = sqlx::query("INSERT INTO sg_data.course (cid, c_name, tid) VALUES (?, ?, ?)")
                .bind(new_course.cid)
                .bind(new_course.c_name.clone())
                .bind(new_course.tid)
                .execute(&*pool_c)
                .await.unwrap();
            //println!("[{row_idx}]: insert row {:?} fetched \n", res);
            });
            handles.push(handle);
        }
        else {
            let pool_c = normal_pool.clone();
            // The problem is that the pool variable is being moved 
            // into the async block inside the loop
            // , which causes it to be unavailable in subsequent iterations.
            let handle  = tokio::spawn(async move {
                let new_course = Course {
                    cid:  row_idx as i32,
                    c_name: generate_random_utf8_string(5),
                    tid: 1,};
                let res = sqlx::query("INSERT INTO sg_data.course (cid, c_name, tid) VALUES (?, ?, ?)")
                .bind(new_course.cid)
                .bind(new_course.c_name.clone())
                .bind(new_course.tid)
                .execute(&pool_c)
                .await.unwrap();
            //println!("[{row_idx}]: insert row {:?} fetched \n", res);
            });
            handles.push(handle);
        };
        
    }
   // println!("handle {:#?}", &handles);
    for handle in handles {
        handle.await.unwrap();
    }
    //let pool = MySqlPoolOptions::new()
    //    .max_connections(5)
    //    .connect(&database_url)
    //    .await?;
    //let courses: Vec<Course> = sqlx::query_as("SELECT * from sg_data.course")
    //            .fetch_all(&pool)
    //            .await.unwrap();
    //println!("courses: {:?}", courses);
    let duration = start.elapsed();
    let insert_str = if let true = arc_mode {"use arc"} else {"no use arc"}; 
    println!("mode:{insert_str}- [{num_connector}] connectors  works, Time elapsed in sqlx_asymain() is: {:?}", duration);
    Ok(())
}