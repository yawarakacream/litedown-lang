pub mod environment;
pub mod nom_utility;
pub mod parser;

use std::{env, fs};

use nom_utility::print_nom;
use parser::{
    environment::parse_environment, environment_header::parse_environment_header,
    passage_line::parse_passage_line,
};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        print_nom(
            "@name[string = `aiueo`, number = 1.1]@",
            parse_environment_header(0),
        );

        print_nom(
            "\
    @name[
        string = `あいうえお`,
        number = 1.1
    ]@
        aaa
        bbb",
            parse_environment(0),
        );

        print_nom("@inlineev@ chichichi", parse_environment(0));

        print_nom("@eaaaaaav@", parse_environment(0));

        print_nom(
            "@ev@
     ",
            parse_environment(0),
        );

        print_nom(
            "@ev@
    iorteu",
            parse_environment(0),
        );

        print_nom(
            "@ev@
     line 1
     line 2
     a
    
     
     b
     ",
            parse_environment(0),
        );

        print_nom(
            "\
    @env1@
        aaa
        bbb
    
        ccc
    
        @env2@
            xxx
            yyy
    
            zzz
    
        ddd
     ",
            parse_environment(0),
        );

        print_nom("left @func{body} right", parse_passage_line);
    } else if args.len() == 2 {
        let source_path = &args[1];
        println!("Parsing {}", source_path);
        // let output_path = "./demo/demo.html";

        let source_code = fs::read_to_string(&source_path).unwrap();
        print_nom(&source_code, parse_environment(0))
    } else {
        println!("Too many arguments");
    }
}
