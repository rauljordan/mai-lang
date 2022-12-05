use std::path::PathBuf;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::process::{Command,Stdio};
use execute::Execute;

use structopt::StructOpt;

use inkwell::context::Context;
use inkwell::passes::PassManager;

mod token;
mod lexer;
mod parser;
mod llvm_translator;

use llvm_translator::Translator;
use parser::Parser;
use lexer::TokenLexer;
use token::Token;

#[derive(StructOpt,Debug)]
#[structopt(name = "mai")]
struct Opts {
    #[structopt(short,long,default_value="main.mai")]
    input: PathBuf,
}

fn main() -> eyre::Result<()> {
    let opts = Opts::from_args();
    println!("Input file path: {:?}", opts.input);

    let input = fs::read_to_string(opts.input).unwrap();
    println!("Raw input contents:");
    println!("{:?}", input);
    println!("");

    let lexer_res = TokenLexer::new(input.as_str()).collect::<Vec<Token>>();
    println!("Lexed tokens:");
    println!("{:?}", lexer_res);
    println!("");


    let parsed_statements = Parser::new(lexer_res).parse();
    println!("Parsed expression:");
    println!("{:?}", parsed_statements);
    println!("");

    let context = Context::create();
    let module = context.create_module("tmp");
    let builder = context.create_builder();

    // Pass manager for functions.
    let fpm = PassManager::create(&module);

    fpm.add_instruction_combining_pass();
    fpm.add_reassociate_pass();
    fpm.add_gvn_pass();
    fpm.add_cfg_simplification_pass();
    fpm.add_basic_alias_analysis_pass();
    fpm.add_promote_memory_to_register_pass();
    fpm.add_instruction_combining_pass();
    fpm.add_reassociate_pass();

    fpm.initialize();

    // TODO: Translate all statements into LLVM IR.
    let first_stmt = parsed_statements.first().unwrap();
    let translated = Translator::translate(
        &context, 
        &builder, 
        &fpm, 
        &module, 
        &first_stmt,
    ).unwrap();
    let result = translated
        .to_string()
        .replace("\"", "")
        .replace("\\n", "\n");

    // Write an IR file to the temporary dir.
    let mut file = File::create("/tmp/main.ll")?;
    file.write_all(result.into_bytes().as_slice())?;

    // Execute LLC to translate into an object file targeted at the 
    // wasm32-unknown-unknown triple.
    // TODO: Use llvm-sys to programmatically perform the following actions rather than
    // hardcoding llvm 15 toolchain commands.
    let mut command = Command::new("llc-15");
    command.arg("-march=wasm32");
    command.arg("-filetype=obj");
    command.arg("/tmp/main.ll");
    command.arg("-o=/tmp/main.o");

    let Some(0) = command.execute().unwrap() else {
        panic!("Could not compile bitcode");
    };

    // Execute wasm-ld to translate the bitcode into web assembly.
    let mut command = Command::new("wasm-ld-15");
    command.arg("/tmp/main.o");
    command.arg("-o");
    command.arg("/tmp/main.wasm");
    command.arg("--no-entry");
    // TODO: Do not export all, as it is dangerous.
    command.arg("--export-all");

    let Some(0) = command.execute().unwrap() else {
        panic!("Could not compile wasm binary");
    };

    let mut command = Command::new("wasm2wat");
    command.arg("/tmp/main.wasm");

    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let output = command.execute_output()?;
    let Some(0) = output.status.code() else {
        panic!("Could not show wat for compiled wasm");
    };

    let wat_output = String::from_utf8(output.stdout)?;
    println!("Compiled wasm to wat:");
    println!("{}", wat_output);

    let mut file = File::create("/tmp/main.wat")?;
    file.write_all(wat_output.clone().into_bytes().as_slice())?;

    // Running the web assembly module with wasmer;
    let mut store = wasmer::Store::default();
    let module = wasmer::Module::new(&store, &wat_output)?;
    // The module doesn't import anything, so we create an empty import object.
    let import_object = wasmer::imports! {};
    let instance = wasmer::Instance::new(&mut store, &module, &import_object)?;

    let safe_sub = instance.exports.get_function("safe_sub")?;
    let result = safe_sub.call(&mut store, &[wasmer::Value::F64(20.0), wasmer::Value::F64(13.0)])?;
    println!("{:?}", result[0]);
    let result = safe_sub.call(&mut store, &[wasmer::Value::F64(20.0), wasmer::Value::F64(21.0)])?;
    println!("{:?}", result[0]);
    Ok(())
}

