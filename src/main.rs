use std::path::PathBuf;
use std::fs;

use inkwell::values::AnyValue;
use structopt::StructOpt;

use inkwell::context::Context;
use inkwell::passes::PassManager;

mod token;
mod lexer;
mod parser;
mod ir_translator;

use ir_translator::LLVMTranslator;
use parser::Parser;
use lexer::TokenLexer;
use token::Token;

#[derive(StructOpt,Debug)]
#[structopt(name = "mai")]
struct Opts {
    #[structopt(short,long,default_value="main.mai")]
    input: PathBuf,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let opts = Opts::from_args();
    println!("Input file: {:?}", opts.input);

    let input = fs::read_to_string(opts.input).unwrap();
    println!("Raw input contents: {:?}", input);
    let lexer_res = TokenLexer::new(input.as_str()).collect::<Vec<Token>>();
    println!("Lexer tokens: {:?}", lexer_res);


    let mut parser = Parser::new(lexer_res);
    let parser_res = parser.expression();
    println!("Parsed expression: {:?}", parser_res);

    let context = Context::create();
    let module = context.create_module("tmp");
    let builder = context.create_builder();

    // Create FPM
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

    let item = LLVMTranslator::translate(&context, &builder, &fpm, &module, &parser_res).unwrap();
    println!("Translated LLVM IR: {:?}", item.print_to_string());

    Ok(())
}






