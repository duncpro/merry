use merry_compiler::codegen::codegen;
use merry_compiler::report::{Issue, print_issue};
use merry_compiler::mtree::{make_mtree, verify_mtree};
use merry_compiler::ltree::{make_ltree, verify_ltree};
use merry_compiler::misc::ansi;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let Some(input_file) = args.get(1) else {
        println!("No input file given.");
        println!("Usage: merry <source file>");
        return Ok(());
    };
    println!();
    println!("{}#{} merry {}v{}{} is compiling {}\"{}\"{}...", ansi::BOLD, ansi::DEFAULT_TEXT_STYLE,
        ansi::FG_GREY, env!("CARGO_PKG_VERSION"), ansi::FG_DEFAULT,
        ansi::FG_GREY, input_file, ansi::FG_DEFAULT);
    let source_text = std::fs::read_to_string(input_file)?;
    let ltree = make_ltree(&source_text);
    let mtree = make_mtree(&ltree);
    let mut issues: Vec<Issue> = Vec::new();
    for issue in verify_ltree(&ltree) { issues.push(issue.into()) }
    for issue in verify_mtree(&mtree) { issues.push(issue.into()) }
    println!("{}##{} compilation finished with {}{}{} issues.", ansi::BOLD, ansi::DEFAULT_TEXT_STYLE,
        ansi::FG_GREY, issues.len(), ansi::FG_DEFAULT);
    println!();
    issues.sort_by_key(|issue| issue.quote.first_line_no);
    for (i, issue) in issues.iter().enumerate() { 
        print!("{}. ", i + 1);
        print_issue(issue, input_file); 
    }
    
    let output_file_path = args.get(2).map(|a| a.as_str()).unwrap_or("out.html");
    let mut output = std::fs::File::create_new(output_file_path)?;
    codegen(&mtree, &mut output)?;
    return Ok(());
}
