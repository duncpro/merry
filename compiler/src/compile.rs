use std::ffi::OsStr;
use std::path::PathBuf;

use crate::ctree::make_ctree;
use crate::report::{Issue, print_issue};
use crate::mtree::{make_mtree, verify_mtree};
use crate::ltree::{make_ltree, verify_ltree};
use crate::misc::ansi;
use crate::{codegen_html, assert_matches};

pub fn compile_dir(src_dir_path: PathBuf, dest_dir_path: PathBuf, head: &Option<PathBuf>) -> std::io::Result<()> {
    std::fs::create_dir_all(&dest_dir_path)?;

    let contents = std::fs::read_dir(&src_dir_path)?;

    for entry_result in contents {
        assert_matches!(entry_result, Ok(entry));
        let mut nested_dest_dir_path = dest_dir_path.clone();
        nested_dest_dir_path.push(entry.file_name());
        if entry.file_type()?.is_dir() {
            compile_dir(entry.path(), nested_dest_dir_path, head)?;
            continue;
        }
        if entry.file_type()?.is_file() {
            if let Some(extension) = entry.path().extension() {
                if extension == "md2" {
                    let mut dest_file_path = nested_dest_dir_path.clone();
                    dest_file_path.set_extension(OsStr::new("html"));
                    compile_file(entry.path(), dest_file_path, head)?;
                }
            }
        }
    }
    return Ok(())
}

pub fn compile_file(input_file: std::path::PathBuf, output_file: std::path::PathBuf, head: &Option<PathBuf>)
-> std::io::Result<()> 
{
    println!("{}#{} merryc {}v{}{} is compiling {}\"{}\"{}...", ansi::BOLD, ansi::STOP_BOLD,
        ansi::FG_GREY, env!("CARGO_PKG_VERSION"), ansi::FG_DEFAULT,
        ansi::FG_GREY, input_file.display(), ansi::FG_DEFAULT);
    
    let source_text = std::fs::read_to_string(&input_file)?;
    let ltree = make_ltree(&source_text);
    let mtree = make_mtree(&ltree);
    
    let mut issues: Vec<Issue> = Vec::new();
    for issue in verify_ltree(&ltree) { issues.push(issue.into()) }
    for issue in verify_mtree(&mtree) { issues.push(issue.into()) }

    let mut cwd = input_file.clone();
    assert!(cwd.pop());
    let ctree = make_ctree(mtree, &mut issues, cwd);
    
    let mut output = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&output_file)?;
    codegen_html::codegen(&ctree, &mut output, &mut issues, head)?; 
    println!("{}##{} compilation finished with {}{}{} issues.", ansi::BOLD, ansi::STOP_BOLD,
        ansi::FG_GREY, issues.len(), ansi::FG_DEFAULT);
    println!();
    
    issues.sort_by_key(|issue| issue.quote.first_line_no);
    for (i, issue) in issues.iter().enumerate() { 
        print!("{}. ", i + 1);
        assert_matches!(input_file.as_path().to_str(), Some(in_file_path_str));
        print_issue(issue, in_file_path_str); 
    }
    
    return Ok(());
}

