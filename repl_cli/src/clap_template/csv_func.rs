use std::path::Path;
use csv::Reader;  //, StringRecord, StringRecordsIter};
use anyhow::{Result, Error, anyhow};
use serde::{Serialize, Deserialize};
use std::fs;
//use std::fs::File;
use crate::clap_template::Outputformat;
//use toml::{Deserializer, Serializer};
//use serde_json::{Value};
// 
// TODO:: Ssynchronize the 3 fn into 1?
// TODO:: output path should check for the format match
#[derive(Serialize, Deserialize, Debug)]
// load macro serde::Serialize etc
//#[serde(rename_all = "PascalCase")]
struct Stock {
    index: isize,
    date: String,
    open: f64,
    close: f64,
    high: f64,
    low: f64,
    volume: f64,
    code: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
struct Player {
    name: String,
    position: String,
    #[serde(rename = "DOB")]
    dob: String,
    nationality: String,
    #[serde(rename = "Kit Number")]
    kit: u8,
}


pub fn verify_filename(filepath: &str) -> Result<String, Error> {
    if Path::new(filepath).exists() {
        Ok(filepath.to_string())
    } else {
        Err(anyhow!("error path"))
    }
}

// Removed the implementation of the unstable Try trait

pub fn verify_csv (filepath: &str) -> Result<String, Error> {
    // rust 每一步都要对可能产生err的操作先做验证，会把错误提前返回到 Result<T, E>
    let mut reader = Reader::from_path(&filepath)?;
    // anyhow 库调用之后才可以使用 '？'
    // '？'表示信任result<T>内的值，如果result为error，则返回error
    for result in reader.records().take(3) {
        // 其中 reader.records() 是一个迭代器，每次迭代都会返回一个Result<StringRecord, Error>
        // 而不是返回 StringRecord,
        let record = match result {
            Ok(record) => record,
            Err(e) => return Err(e.into()), // 这里用into()把anyhow::Error转换为std::io::Error
        };
        println!("{:?}", record);
    };

    Ok(filepath.to_string())
        
}

#[derive(Serialize)]
struct TomlArc {
    dummy_key: Stock,

}


pub fn parse_to_json(filepath: &str
    , output_path: &str
    , capacity: usize, 
    ) -> Result<(), Error> {
    
    let mut rdr = Reader::from_path(&filepath)?;
    let mut wtr = Vec::with_capacity(capacity);
    let headers = rdr.headers()?.clone();
    for result in rdr.records() {
        let record = result?;
        let json_value = headers
            .iter()
            .zip(record.iter())
            .collect::<serde_json::Value>();
        wtr.push(json_value);
    }
            
    let cut_wtr = &wtr[..capacity];
    let content = serde_json::to_string_pretty(cut_wtr)?;
    fs::write(output_path, content)?;

    Ok(())
}

pub fn parse_to_yaml(filepath: &str
    , output_path: &str
    , capacity: usize, 
    ) -> Result<(), Error> {
    let mut rdr = Reader::from_path(&filepath)?;
    let mut wtr = Vec::with_capacity(capacity);
    let headers = rdr.headers()?.clone();
    for result in rdr.records() {
        let record = result?;
        let json_value = headers
            .iter()
            .zip(record.iter())
            .collect::<serde_json::Value>();
        wtr.push(json_value);
    }
            
    let cut_wtr = &wtr[..capacity];
    let content = serde_yaml::to_string(cut_wtr)?;
    fs::write(output_path, content)?;
    
    Ok(())
}


pub fn parse_to_toml(filepath: &str
    , output_path: &str
    , capacity: usize, 
    ) -> Result<(), Error> {
        
    let mut rdr = Reader::from_path(&filepath)?;
    let mut wtr = Vec::with_capacity(capacity);
    for result in rdr.deserialize() {
        let record:Stock = result?;
        // let record: Player = result?;
        let tomlarc = TomlArc {
            dummy_key: record,
        };
        let toml_string = toml::to_string(&tomlarc).unwrap();
        wtr.push(toml_string);
    }
    let cut_wtr = &wtr[..capacity];
    
    let concatenated_content = cut_wtr.join("\n");
    fs::write(output_path, concatenated_content).unwrap();    
    Ok(())

}


pub fn convert_csv(filepath: &str
    , output_path: &str
    , capacity: usize
    , format: Outputformat
    ) -> Result<()> {
    match format {
        Outputformat::Json => {
            if let Err(e) = parse_to_json(filepath
                , output_path
                , capacity
            ) {
                eprint!("parse func error: {:?}", e)
            };
        },
        Outputformat::Yaml => {
            if let Err(e) = parse_to_yaml(filepath
                , output_path
                , capacity
            ) {
                eprint!("parse func error: {:?}", e)
            };
        },
        Outputformat::Toml => {
            if let Err(e) = parse_to_toml(filepath
                , output_path
                , capacity
            ) {
                eprint!("parse func error: {:?}", e)
            };
        },
    };
    // let content = match format {
    //    Outputformat::Json => serde_json::to_string_pretty(cut_wtr)?,
    //    Outputformat::Yaml => serde_yaml::to_string(cut_wtr)?,
    //    Outputformat::Toml => toml::to_string(cut_wtr).unwrap(),
    println!("--------------------------------------------done");
    Ok(())
}

pub fn parse_format(format:&str) -> Result<Outputformat, Error> {
    // 这里的实现方式太low了，需要重构--...--
    // format.parse() 实现了从&str到其他数据类型的转换
    // 前提是该数据类型Outputformat有实现FromStr trait
    // 但是实现的过程是一样的
    match format.to_lowercase().as_str() {
        "json" => Ok(Outputformat::Json),
        //"csv" => Ok(Outputformat::Csv),
        "yaml" => Ok(Outputformat::Yaml),
        "toml" => Ok(Outputformat::Toml),
        _ => Err(anyhow!("error format"))
    }
}
// cargo run dongx  ff_dept -a 35 -l tianjin csv -i .\data\stock_600519.csv -o .\data\stock_6006.yaml -f 'yaml'