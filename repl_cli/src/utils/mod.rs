use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use std::path::{Path, PathBuf};

use anyhow::{Result, Error};
use arrow::array::AsArray;
use rand::{distributions::Alphanumeric, seq::SliceRandom, Rng};

//use polars::prelude::*;
use parquet::{
    column::writer::ColumnWriter
    , data_type::ByteArray
    , file::{
        properties::WriterProperties,
        writer::SerializedFileWriter,
    }
    , schema::parser::parse_message_type
};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use polars::{prelude::*, sql::SQLContext};
use serde_json;
use serde::Serialize;



pub fn generate_random_utf8_string(length: usize) -> String {
    let s: String = rand::thread_rng()
    .sample_iter(&Alphanumeric)
    .take(length)
    .map(char::from)
    .collect();
    s
//println!("{}", s);
}

pub fn generate_random_sex() -> String {
    let mut rng = rand::thread_rng();
    let arr = ["F", "M"];
    let samp = arr.choose(&mut rng).unwrap();
    samp.to_string()
}

pub fn read_config_json(config_path:&str, connection_name:&str) -> Result<Vec<String>, Error> {
    
    let file = File::open(config_path).unwrap();
    let reader = BufReader::new(file);
    let all_configs: Result<serde_json::Value, serde_json::Error> = serde_json::from_reader(reader);
    println!("config json fetched");

    let db_client_str = match all_configs {
        Ok(jsn_value) => {
            jsn_value
            .as_array()
            .unwrap()
            .iter()
            .filter(|x| x.as_object().unwrap().get("connection_name").unwrap().as_str().unwrap() == connection_name)
            .map(|x| {
                let config = x.get("connection_details").unwrap().as_object().unwrap();
                let url = config.get("url").unwrap().as_str().unwrap().to_string();
                let user = config.get("user").unwrap().as_str().unwrap().to_string();
                let password = config.get("password").unwrap().as_str().unwrap().to_string();
                let database = config.get("database").unwrap().as_str().unwrap().to_string();
                //println!("url: {}, user: {}, password: {}, database: {}", url, user, password, database);
                let arr_out = vec![url, user, password, database];
                arr_out.into_iter().collect::<Vec<String>>()
            })
            .collect() //::<Vec<String>>()
        }
        Err(e) => {
            println!("error: {:#?}", e);
            Vec::new()
        }                        //    //.with_database();
    };
    //println!("{:#?}, {}", &db_client_str[0], &db_client_str[0][0]);
    println!("db params fetched");
    Ok(db_client_str[0].clone())

}


//use polars::prelude::*;
//use std::fs::File;
//
//pub fn write_parquet() -> Result<(), PolarsError> {
//    let data = vec![1, 2, 3, 4, 5]; // Vector of integers.
//    let s = Series::new("data", data); // Create series from the vector.
//    let mut df = DataFrame::new(vec![s])?; // Create a DataFrame from series.
//    let file = File::create("example.parquet").expect("could not create file");
//    ParquetWriter::new(file).finish(&mut df); // Write DataFrame to Parquet file.
//    Ok(())
//}


#[derive(Debug, Serialize)]
pub struct Course {
    pub cid: i32,
    pub c_name: String,
    pub tid: i32,
}

// TODO:: online rdr:https://drawingdata.io/tools/parquet-viewer
pub fn write_parquet(outfile_name:&str) -> Result<(), Error> {
    // create wtr needs file, schema, props
    let message_type = "
        message schema {
            REQUIRED INT32 cid;
            REQUIRED BINARY c_name (UTF8);
            REQUIRED INT32 tid;
        }
    ";
    let directory = Path::new("C:\\Users\\dongx\\test_xu\\data");
    let file_path = directory.join(outfile_name);
    let file = File::create(file_path)?;
    let schema = Arc::new(parse_message_type(message_type)?);
    let props = Arc::new(WriterProperties::builder().build());
    let mut writer = SerializedFileWriter::new(file, schema.clone(), props)?;

    // Create a row group writer
    for idx in 1..100 {
        let course = Course {
            cid: idx,
            c_name: generate_random_utf8_string(20),
            tid: 3,
        };
        let mut row_group_writer = writer.next_row_group()?;

        while let Some(mut col_writer) = row_group_writer.next_column().unwrap() {
            match col_writer.untyped() {
                ColumnWriter::Int32ColumnWriter(ref mut typed_writer) => {
                    if typed_writer.get_descriptor().name() == "cid" {
                        typed_writer.write_batch(&[course.cid], None, None).unwrap();
                    } else if typed_writer.get_descriptor().name() == "tid" {
                        typed_writer.write_batch(&[course.tid], None, None).unwrap();
                    }
                }
                ColumnWriter::ByteArrayColumnWriter(ref mut typed_writer) => {
                    if typed_writer.get_descriptor().name() == "c_name" {
                        let byte_array = ByteArray::from(course.c_name.as_str());
                        typed_writer.write_batch(&[byte_array], None, None).unwrap();
                    }
                }
                _ => unimplemented!(),
            }
            col_writer.close().unwrap();
        }
        row_group_writer.close().unwrap();
    }
    //row_group_writer.close().unwrap();
    writer.close().unwrap();

    Ok(())
}


pub fn read_with_polars(infile_name: &str) -> Result<()> {
    let directory = Path::new("C:\\Users\\dongx\\test_xu\\data");
    let file_path = directory.join(infile_name);
    let df = LazyFrame::scan_parquet(file_path, Default::default())?;
    let mut ctx = SQLContext::new();
    ctx.register("course", df);
    let df = ctx
        .execute("SELECT cid::text, c_name::text, tid::text FROM course limit 10")?
        .collect()?;

    println!("{:?}", df);
    Ok(())
}

//TODO:: undone!
pub fn read_with_parquet(infile_name: &str) -> Result<()> {
    
    let directory = Path::new("C:\\Users\\dongx\\test_xu\\data");
    let file_path = directory.join(infile_name);
    let file = File::open(file_path)?;
    let reader = ParquetRecordBatchReaderBuilder::try_new(file)?
        .with_batch_size(8192)
        .with_limit(3)
        .build()?;

    for record_batch in reader {
        let record_batch = record_batch?;

        let c_names = record_batch.column(1).as_binary::<i32>();

        for c_name in c_names {
            let c_name = c_name.unwrap();
            println!("{:?}", String::from_utf8_lossy(c_name));
        }
    }

    Ok(())
}
