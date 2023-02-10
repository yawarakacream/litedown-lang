pub mod environment;
pub mod nom_utility;
pub mod parser;

use nom_utility::print_nom;
use parser::environment_header::parse_environment_header;

fn main() {
    print_nom(
        "@name[string = `あいうえお`, number = 1.1]@",
        parse_environment_header,
    );

    print_nom("@name[1.2em]@", parse_environment_header);
}
