extern crate getopts;

use getopts::Options;
use hrm::interpreter::SimpleInterpreter;
use hrm::lexer::Lexer;
use std::env;
use std::fs::File;
use std::io::Read;
use std::process;

fn print_usage(program: &str, opts: Options) {
    let brief = format!(
        "Human Resource Machine's interpreter written in Rust.
[Steam：Human Resource Machine](https://store.steampowered.com/app/375820/Human_Resource_Machine/)

Usage: {} source.rhm [options]
    FILE: program read from script file",
        program
    );
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("i", "input", "set input file name", "NAME");
    opts.optopt("o", "output", "set output file name", "NAME");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let _output = matches.opt_str("o");
    let input = matches.opt_str("i");
    let script = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(&program, opts);
        return process::exit(64);
    };

    let mut p = String::new();
    let mut f = File::open(script).expect("file not found");
    f.read_to_string(&mut p)
        .expect("something went wrong reading the file");

    let program = Lexer::lex(&p);
    let mut interpreter = SimpleInterpreter::new();
    if let Some(input_path) = input {
        let mut input_file = File::open(input_path).expect("File was not opened");
        let mut buf = String::new();
        let _ = input_file.read_to_string(&mut buf);
        interpreter.set_input_stream(buf);
    }

    let exit_status = interpreter.eval(&program);
    if let Err(e) = exit_status {
        eprintln!("Error: {:?}", e);
    }
}
