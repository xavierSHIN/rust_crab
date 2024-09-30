use anyhow::Result;
use clap::Parser;


mod csv_func;
mod aead_func;
mod thread_func;
mod async_func;
mod klickhouse_func;
mod clickhouse_func;
mod concurrent;

pub use csv_func::*;
pub use aead_func::*;
pub use thread_func::*;
pub use async_func::*;
pub use klickhouse_func::*;
pub use clickhouse_func::*;
pub use concurrent::*;


/*
------------------------------------------------------------------------------------------
主体app:Rcli被赋予了args:name, deprt , 和subcommend: Csv, Json
 , 和options: --age , --location
 , subcmd 内有Csv(options -i， -O), Json(options -i， -O)

Usage: test_xu.exe [OPTIONS] --age <AGE> <NAME> <DEPRT> <COMMAND>

Commands:
  csv   here is an explanation for Csv args
  json  here is an explanation for Json args
  help  Print this message or the help of the given subcommand(s)

Arguments:
  <NAME>   Specify your name 会被解析成 [NAME]
  <DEPRT>  Specify your department name 会被解析成 [deprt_name]

Options:
  -a, --age <AGE>            Specify your age 会被解析成 [age]
  -l, --location <LOCATION>  Specify your location 会被解析成 [location] [default: wuhan]
  -h, --help                 Print help
  -V, --version              Print version
  ------------------------------------------------------------------------------------------
*/

// cmdline: test_xu.exe <NAME> <DEPRT> --age <AGE> --location <LOCATION> <COMMAND>
// cmdline: cargo run dongx  ff_dept -a 35 -l tianjin csv -i .\data\stock_600519.csv -o .\data\stock_600519.json
#[derive(Parser, Debug)]
#[clap(author, version, about)]
#[command(name="rcli", version="0.1.1", about, long_about = None)]
// TODO:: 7) 找到cobra版本的clap???
// todo:: focus on result!
// 主体app:Rcli被赋予了args:name, deprt
// , 和subcommend: Csv, Json
// , 和options: --age , --location
// , subcmd 内有Csv(options -i， -O), Json(options -i， -O) 
struct Rcli {
    /// Specify your name
    /// 会被解析成 [NAME]
    #[arg(short, long, default_value="dongx")]
    name: String,
    /// Specify your department name
    /// 会被解析成 [deprt_name]
    #[arg(short, long, default_value="ff_dept")]
    deprt: String,
    /// Specify your age
    /// 会被解析成 [age]
    #[arg(short, long, default_value_t=34)]
    age: i8,
    /// Specify your location
    /// 会被解析成 [location]
    #[arg(short, long, default_value="wuhan")]
    location: String,
    /// 用struct 定义subcmd, 每一个field是一个subcmd,他的类型是enum,即该field可以接受的value
    /// 在 clap 中可以使用 #[command(subcommand)] 搭配 #[derive(Subcommand)] 实现子命令功能
    #[command(subcommand)]
    take_action: Subcmd,
}


#[derive(Debug, Parser)]
enum Subcmd {
    /// here is an explanation for Csv args
    #[command(name = "csv", about = "Show CSV, or convert CSV to other formats")]
    Csv(Csvargs), 
    /// here is an explanation for Json args
    #[command(name = "json", about = "Show JSON, or convert JSON to other formats")]
    Json(Jsonargs),
    #[command(subcommand, about = "Text，使用chacha20poly1305算法来加密&解密")]
    Text(TextSubCommand),
    //Image(ImageSubCommand)
    #[command(name = "thread", about = "展示多线程工作，在线程间是如何传递信息的")]
    Tread(Threadargs)
}

#[derive(Debug, Parser, Clone, Copy)]
pub enum Outputformat {
    Json,
    //Csv,
    Yaml,
    Toml
}

#[derive(Debug, Parser)]
struct Csvargs {
    /// 输入csv所在的文件路径
    // 会被解析成 -i 因为用了 #[arg(short, long)]
    #[arg(short, long, default_value=r".\data\input.csv", value_parser= csv_func::verify_csv)]
    input: String,

    /// 想要输出的文件路径
    // value_parse func 返回的对象会成为新的 arg.input 
    // 所以需要保证 input 依然是String
    // todo:: 4) 需要一个fn verify_filename来验证文件名
    #[arg(short, long, default_value=r".\data\output.json")]
    output: String,

    /// 分隔符
    // default_value_t 是简单引用
    #[arg(short, long, default_value_t=',')]
    delim: char,

    /// 输出格式, 可以从json, yaml中选择
    // 此处不使用string类型而是要用enum：Outputformat
    // 在命令行我们能直接用Outputformat::Json来指定么？no
    // 需要一个输入 format_str 到 parse_format fn中，然后输出: Outputformat::Json
    // 输出的Outputformat::Json要参与到 convert_csv中
    #[arg(short, long, default_value="json" ,value_parser=csv_func::parse_format)]
    format: Outputformat

}

#[derive(Debug, Parser)]
pub struct Threadargs {
    /// 输入要使用的线程数
    #[arg( long, default_value="4")]
    num_thread: usize
}

#[derive(Debug, Parser)]
//#[enum_dispatch(CmdExector)]
pub enum TextSubCommand {
    #[command(about = "Encrypt: 对文本加密并输出base64， 使用chacha20算法，需要key(口令)和nonce(动态数字)")]
    Encrypt(TextEncryptOpts),
    #[command(about = "Decrypt: 接收base64解密成文本， 需要和加密过程相同的key(口令)和nonce(动态数字)")]
    Decrypt(TextDecryptOpts),
    //#[command(about = "Generate a random key & nonce pair")]
    //Generate(KeyGenerateOpts)
}

#[derive(Debug, Parser)]
pub struct TextEncryptOpts {
    /// input 可以从命令行读取，也可以从文件路径读取，需要utf8格式
    #[arg(short, long, default_value = "-")]
    pub input: String,
    /// key类似于一个口令，需要在传递者和接收者保持一致
    // , and need insert 'salt' as part of your key 
    #[arg(short, long, default_value="dongx123salt")]
    pub key: String,
    ///nonce代表一个动态数字，类似于两步验证中发送到手机号的验证码
    // need convert u128 to [u8, 12]
    // The largest value that can be represented by this integer type 2^(2128) − 1.
    #[arg( long, default_value_t=123456789)]
    pub nonce: u128,
    /// 密文的输出用base64编码，需要选择格式standard还是urlsafe
    // because the output is base64, so we need chose format from standard or urlsafe
    #[arg(long, default_value="standard")]
    pub base_format: String,
}

#[derive(Debug, Parser)]
pub struct TextDecryptOpts {
    /// input 可以从命令行读取，也可以从文件路径读取，需要base64
    #[arg(short, long,  default_value = "-")]
    pub input: String,
    #[arg(short, long, default_value="dongx123salt")]
    /// key类似于一个口令，需要在传递者和接收者保持一致
    pub key: String,
    /// nonce代表一个动态数字，类似于两步验证中发送到手机号的验证码
    #[arg( long, default_value_t=123456789)]
    pub nonce: u128,
    #[arg( long, default_value="standard")]
    /// 密文的接收要用base64编码，需要选择格式standard还是urlsafe
    pub base_format: String,
}


#[derive(Debug, Parser)]
struct Jsonargs {
    
    #[arg(short, long, default_value="in.json", value_parser= csv_func::verify_filename)]
    input: String,
    /// Specify your output optionally
    /// 会被解析成 -o 因为用了 #[arg(short, long)]
    #[arg(short, long, default_value="out.csv")]
    output: String
}

#[warn(unreachable_patterns)]
pub fn clap_template() -> Result<()> {
    let rcli = Rcli::parse();
    match rcli.take_action {
        Subcmd::Csv(args) => {
    //        //println!("xixi");
            let _ = csv_func::convert_csv(
                &args.input
                , &args.output
                , 5
                , args.format);
            println!("{:#?}", args);
        },
        Subcmd::Json(args) => {
            println!("json out: {:#?}, {:#?}", &args.input, &args.output);
        },
        Subcmd::Tread(args) => {
            let res = thread_func::main_thread(args.num_thread);
            match res {
                Ok(_) => println!("thread done"),
                Err(e) => println!("thread error: {:#?}", e)
            }
        },
        Subcmd::Text(args) => match args {
            TextSubCommand::Encrypt(opts) => {
                println!("text, {:#?}", opts);
                //let mut rdr = aead::get_reader(&opts.input)?;
                let encd = aead_func::process_aead_encode(
                    &opts.input
                    , &opts.key    
                    , opts.nonce             
                    , &opts.base_format);
                match encd {
                    Ok(encd) => println!("encoded: {:#?}", encd),
                    Err(e) => println!("error: {:#?}", e)
                }
                //println!("encoded: {:#?}", encd);
            },
            TextSubCommand::Decrypt(opts) => {
                println!("text, {:#?}", opts);
                let decd = aead_func::process_aead_decode(
                    &opts.input
                    , &opts.key  
                    , opts.nonce                  
                    , &opts.base_format);
                match decd {
                        Ok(decd) => println!("encoded: {:#?}", decd),
                        Err(e) => println!("error: {:#?}", e)
                }
                //println!("decoded: {:#?}", decd);
            },
           // _ => panic!("unknown text subcommand")
        }
    }
    Ok(())
}

// cargo run text encrypt -i  dongxu -k 1 --nonce 12345678901234567890123456789012
// cargo run text decrypt -i ckIstoEhUPj9NQztHvm95_2zZKAMiw --base-format urlsafe
// lsMEOIlscmrAdjLRntNju1sZL6Y=
// lsMEOIlscmrAdjLRntNju1sZL6Y   xixi urlsafe


//EVQ0UbcV9eXpMMfKajUpBfwovTQYDXb3xTxuaOz13xdJhjfXxxkEJtmgsY8GUSovlp5prqTH561WSQ==