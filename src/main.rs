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

fn main() -> eyre::Result<()> {
    let opts = Opts::from_args();
    println!("Input file: {:?}", opts.input);

    let input = fs::read_to_string(opts.input).unwrap();
    println!("Raw input contents: {:?}", input);
    let lexer_res = TokenLexer::new(input.as_str()).collect::<Vec<Token>>();
    println!("Lexer tokens: {:?}", lexer_res);


    let parsed_statements = Parser::new(lexer_res).parse();
    println!("Parsed expression: {:?}", parsed_statements);

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

    let first_stmt = parsed_statements.first().unwrap();
    println!("{:?}", first_stmt);
    let translated = LLVMTranslator::translate(
        &context, 
        &builder, 
        &fpm, 
        &module, 
        &first_stmt,
    ).unwrap();
    println!("Translated LLVM IR: {}", translated);

    Ok(())
}






