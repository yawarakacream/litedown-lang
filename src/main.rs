use std::{
    env,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use anyhow::{bail, Context, Result};
use litedown_lang::{
    html_evaluator::litedown::{evaluate_litedown_to_html, Ld2HtmlInput},
    parser::litedown::parse_litedown,
    utility::{html::print_html_to_pdf, tree_string_builder::ToTreeString},
};

struct Argument<'a> {
    path: &'a str,
    pdf: bool,
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let args = {
        let mut path = None;
        let mut pdf = None;
        let mut i = 1;
        while i < args.len() {
            let arg = args[i].as_str();
            if arg.starts_with("-") {
                match &arg[1..] {
                    "p" | "-pdf" => {
                        if pdf.is_none() {
                            pdf = Some(true);
                        } else {
                            bail!("Duplicate argument: {}", arg);
                        }
                    }
                    _ => {
                        bail!("Unknown argument: {}", arg);
                    }
                }
            } else {
                if path.is_none() {
                    path = Some(arg);
                } else {
                    bail!("Invalid argument: {}", arg);
                }
            }
            i += 1
        }
        Argument {
            path: path.context("No path provided")?,
            pdf: pdf.unwrap_or(false),
        }
    };

    let source_path =
        fs::canonicalize(PathBuf::from(args.path)).context("Could not canonicalize source path")?;

    // check extension
    let source_file_extension = source_path.extension().unwrap();
    if !source_file_extension.eq_ignore_ascii_case("ld") {
        bail!(
            "Invalid source path: Unknown extension {:?}",
            source_file_extension
        );
    }

    let source_code = fs::read_to_string(&source_path).context("Could not read source file")?;

    // ast
    println!("Parsing {:?}", source_path);
    let ast = parse_litedown(&source_code).context("Could not parse ld")?;

    let ast_string = ast.to_tree_string();
    if ast_string.lines().count() < 16 {
        println!("{}", ast.to_tree_string());
    } else {
        let output_ast_path = source_path.with_extension("ldast.txt");
        println!("Saving ast to {:?}", output_ast_path);

        let mut output_ast = File::create(&output_ast_path).unwrap();
        writeln!(output_ast, "{}", ast_string).unwrap();
        output_ast.flush().unwrap();
    }

    // html
    let html = evaluate_litedown_to_html(Ld2HtmlInput {
        ast,
        source_path: Some(source_path.clone()),
    })
    .context("Could not evaluate ast to html")?
    .to_string()
    .merge();

    // save html
    let output_html_path = source_path.with_extension("html");
    println!("Saving html to {:?}", output_html_path);

    let mut output_html_file = File::create(&output_html_path).unwrap();
    writeln!(output_html_file, "{}", html).unwrap();
    output_html_file.flush().unwrap();

    // pdf
    if args.pdf {
        let output_pdf_path = source_path.with_extension("pdf");
        println!("Saving pdf to {:?}", output_pdf_path);

        let output_pdf_data = print_html_to_pdf(output_html_path.to_str().unwrap()).unwrap();
        fs::write(output_pdf_path, &output_pdf_data).unwrap();
    }

    Ok(())
}
