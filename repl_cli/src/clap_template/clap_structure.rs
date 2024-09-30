
use clap::{
    Parser //
    //, Args //
    , Subcommand
    , ValueEnum
};

// 学习资料： https://blog.csdn.net/Hedon954/article/details/139578613
/*
 *Tcli 是定义的最简单的cli
 主命令(command)-> args & oprions
 Rcli 则更加复杂一些，因为加入了子命令(subcmd)
 结构 APP（Rcli):
       -- Subcmd(Csv):
        --    Options(-i -o)
 */

 
#[derive(Parser)]
///#[derive(Parser)] 是一个过程宏（procedural macro）
/// ，用于自动为结构体实现 clap::Parser trait。这使得该结构体可以用来解析命令行参数。
/// aka, impl Parser for Tcli { fn parser() ...}
#[command(version, author, about, long_about = None)]

/// todo:: 什么是arguments vs options?
/// arguments 可以直接输入，要严格按照顺序
/// options 需要-o开头，无所谓顺序
/// cargo run dongx -a 35 -l wuchang ff_depart fast

struct Tcli {
    /// Specify your name
    /// 会被解析成 [NAME]
    name: String,
    /// Specify your department name
    /// 会被解析成 [deprt_name]
    deprt: String,

    /// Specify your age optionally
    /// 会被解析成 -a <AGE>, 因为用了 #[arg(short, long)]
    #[arg(short, long)]
    age: i8,

    #[arg(short, long, default_value="wuhan")]
    location: String,

    #[arg(value_enum)]
    mode: Mode,
}

/// cargo run dongx -a 35 -l wuchang ff_depart fast
 
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    /// run in fast mode
    Fast,
    /// run in slow mode
    Slow,
}


pub fn main_x() {
    let tcli = Tcli::parse();
    println!("name: {:?}", tcli.name);
    println!("deprt: {:?}", tcli.deprt);
    println!("age: {:?}", tcli.age);
    println!("location: {:#?}", tcli.location);
    match tcli.mode {
        Mode::Fast => println!("fast mode"),
        Mode::Slow => println!("slow mode"),
    }
}
