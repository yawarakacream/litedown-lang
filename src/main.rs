pub mod evaluator;
pub mod litedown_element;
pub mod parser;
pub mod utility;

use std::{
    env,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use crate::{
    evaluator::default::document::document::Document,
    parser::environment::parse_litedown,
    utility::{html::print_html_to_pdf, tree_string_builder::ToTreeString},
};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 2 {
        let source_path = PathBuf::from(&args[1]);
        let source_path = fs::canonicalize(source_path).unwrap();
        println!("Parsing {:?}", source_path);
        // let output_path = "./demo/demo.html";

        let source_code = fs::read_to_string(&source_path).unwrap();
        let source_code = source_code.as_str();

        // let ret = print_nom(&source_code, parse_environment(0));
        // if let Some(env) = ret {
        //     println!("{}", env.stringify_as_tree().unwrap());
        // }

        match parse_litedown(Some(source_path.clone()), source_code) {
            Ok(ast) => {
                // ast
                println!("{}", ast.to_tree_string());

                // html
                let evaluator = Document::new_evaluator();
                let html = match evaluator.eval(source_path.clone(), ast) {
                    Ok(html) => html.merge(),
                    Err(error) => {
                        println!("An error occurred\n{:?}", error);
                        return;
                    }
                };
                // println!("{}", html);

                let source_file_extension = source_path.extension().unwrap();
                if !source_file_extension.eq_ignore_ascii_case("ld") {
                    panic!("Illegal extension: {:?}", source_file_extension);
                }

                let source_file_name = source_path.file_name().unwrap().to_str().unwrap();
                let source_file_name_without_ext = &source_file_name
                    [0..(&source_file_name.len() - (&source_file_extension.len() + 1))];

                // save html
                let output_html_path =
                    source_path.with_file_name(format!("{}.html", source_file_name_without_ext));

                println!("Saving to {:?}", output_html_path);

                let mut output_html = File::create(&output_html_path).unwrap();
                writeln!(output_html, "{}", html).unwrap();
                output_html.flush().unwrap();

                // save pdf
                let output_pdf_path =
                    source_path.with_file_name(format!("{}.pdf", source_file_name_without_ext));

                println!("Saving to {:?}", output_pdf_path);

                let output_pdf = print_html_to_pdf(output_html_path.to_str().unwrap()).unwrap();
                fs::write(output_pdf_path, &output_pdf).unwrap();
            }
            Err(err) => {
                eprintln!("\x1B[41mError!\x1B[0m");
                eprintln!("{}", nom::error::convert_error(source_code, err.clone()));
            }
        }
    } else {
        println!("Too many arguments");
    }
}
