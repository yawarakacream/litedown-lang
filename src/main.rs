use std::{
    env,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use anyhow::{bail, Context, Result};
use litedown_lang::{
    html_evaluator::litedown::{evaluate_litedown_to_html, Ld2HtmlInput},
    // evaluator::default::{document::document::Document, slide::slide::Slide},
    parser::litedown::parse_litedown,
    utility::{html::print_html_to_pdf, tree_string_builder::ToTreeString},
};

struct Argument<'a> {
    path: &'a str,
    pdf: bool,
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // test
    if args.len() == 1 {
        // parse_litedown_new("@test0@")?;
        // println!("");

        // parse_litedown_new("@test1[1]@")?;
        // println!("");

        // parse_litedown_new("@test2[-2.4uda]@")?;
        // println!("");

        // parse_litedown_new("@test2[p = 1.5u]@")?;
        // println!("");

        // parse_litedown_new(r#"@test2[pfwefw = "aaa"]@"#)?;
        // println!("");

        // parse_litedown_new(r#"@test2["aaa"]@"#)?;
        // println!("");

        // parse_litedown_new(r#"@test2[hiugret, j6895, p = 54,]@"#)?;
        // println!("");

        parse_litedown(
            r#"@test2[hiugret, j6895, p = 54]:
  aaa
  bbb"#,
        )?;
        println!("");

        parse_litedown(r#"@f[p]{あいうえお@g@{かきくけこ}さしすせそ}"#)?;
        println!("");

        parse_litedown(
            r#"@test2[hiugret, j6895, p = 54]@
    aaa
    bbb
    ccc
    
    ddd
    @rgn@
        123
        456

    789
    
    @tr

    eee
    
    @yu[aaa]@
        yopkj
        
        @a@
            h
        htrui
    k
    "#,
        )?;
        println!("");

        parse_litedown(
            r#"@fff@
    111
    
    @ggg@
        222
        
        @hhh@
            333
        
            @iii@
                ouou

                @jjj


        aeg


        
        p
        
    444"#,
        )?;
        println!("");

        return Ok(());
    }

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

    let source_path = PathBuf::from(args.path);
    let source_path =
        fs::canonicalize(source_path).context("Could not canonicalize source path")?;

    // check extension
    let source_file_extension = source_path.extension().unwrap();
    if !source_file_extension.eq_ignore_ascii_case("ld") {
        bail!(
            "Invalid source path: Unknown extension {:?}",
            source_file_extension
        );
    }

    let source_code = &fs::read_to_string(&source_path).context("Could not read source file")?;

    // ast
    println!("Parsing {:?}", source_path);
    let ast = parse_litedown(source_code).context("Could not parse ld")?;
    println!("{}", ast.to_tree_string());

    // html
    let html = evaluate_litedown_to_html(Ld2HtmlInput {
        ast,
        source_path: Some(source_path.clone()),
    })
    .context("Could not evaluate ast to html")?
    .to_string()
    .merge();

    let source_file_name = source_path.file_name().unwrap().to_str().unwrap();
    let source_file_name_without_ext =
        &source_file_name[0..(&source_file_name.len() - (&source_file_extension.len() + 1))];

    // save html
    let output_html_path =
        source_path.with_file_name(format!("{}.html", source_file_name_without_ext));

    println!("Saving to {:?}", output_html_path);

    let mut output_html = File::create(&output_html_path).unwrap();
    writeln!(output_html, "{}", html).unwrap();
    output_html.flush().unwrap();

    // pdf
    if args.pdf {
        let output_pdf_path =
            source_path.with_file_name(format!("{}.pdf", source_file_name_without_ext));

        println!("Saving to {:?}", output_pdf_path);

        let output_pdf = print_html_to_pdf(output_html_path.to_str().unwrap()).unwrap();
        fs::write(output_pdf_path, &output_pdf).unwrap();
    }

    Ok(())
}
