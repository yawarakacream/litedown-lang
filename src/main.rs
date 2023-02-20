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

use parser::{
    environment::parse_environment, environment_header::parse_environment_header,
    passage_line::parse_passage_line,
};
use utility::nom::print_nom;

use crate::{evaluator::litedown::LitedownEvaluator, parser::environment::parse_litedown};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        print_nom(
            "@name[string = `aiueo`, number = 1.1]@",
            parse_environment_header(0),
        );

        print_nom("left @func{body} right", parse_passage_line);

        print_nom(
            "\
    @env1@
        aaa
        bbb @func[t = 1]{pqr}
    
        ccc
    
        @env2@
            xxx
            yyy
    
            zzz
    
        ddd
     ",
            parse_environment(0),
        );
    } else if args.len() == 2 {
        let source_path = PathBuf::from(&args[1]);
        let source_path = fs::canonicalize(source_path).unwrap();
        println!("Parsing {:?}", source_path);
        // let output_path = "./demo/demo.html";

        let source_code = fs::read_to_string(&source_path).unwrap();

        // let ret = print_nom(&source_code, parse_environment(0));
        // if let Some(env) = ret {
        //     println!("{}", env.stringify_as_tree().unwrap());
        // }

        match parse_litedown(&source_code) {
            Ok(ast) => {
                // ast
                println!("{}", ast.root.stringify_as_tree().unwrap());

                // html
                let evaluator = LitedownEvaluator::new();
                let html = evaluator.eval(&ast).unwrap();
                println!("{}", html);

                let source_file_extension = source_path.extension().unwrap();
                if !source_file_extension.eq_ignore_ascii_case("ld") {
                    panic!("Illegal extension: {:?}", source_file_extension);
                }

                let source_file_name = source_path.file_name().unwrap().to_str().unwrap();
                let source_file_name_without_ext = &source_file_name
                    [0..(&source_file_name.len() - (&source_file_extension.len() + 1))];

                // save html
                let output_html =
                    source_path.with_file_name(format!("{}.html", source_file_name_without_ext));

                println!("Saving to {:?}", output_html);

                let mut output_html = File::create(output_html).unwrap();
                writeln!(output_html, "{}", html).unwrap();
                output_html.flush().unwrap();
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }
    } else {
        println!("Too many arguments");
    }
}
