use std::fs::{metadata, File};
use std::io::{Read, Write};
use std::process::exit;

use clap::{crate_authors, crate_version, App, Arg};
use syn;
use walkdir::{DirEntry, WalkDir};

use ruml::{file_parser, render_plantuml, Entity};

fn main() {
    let matches = App::new("ruml")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Parse rust code and visualize")
        .arg(
            Arg::with_name("output_type")
                .short("t")
                .long("type")
                .value_name("OUTPUT_TYPE")
                .help("Output type. Default value is PlantUml")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file or directory to use")
                .required(false)
                .index(1),
        )
        .arg(
            Arg::with_name("FILENAME")
                .help("Sets the output file name")
                .required(false)
                .index(2),
        )
        .get_matches();

    let source = matches
        .value_of("INPUT")
        .unwrap_or(".");

    let file_name = matches.value_of("FILENAME").unwrap_or("plantuml.puml");

    match metadata(source) {
        Err(e) => {
            println!("Unable to find source.{}{}", source, e);
            exit(1)
        }
        Ok(md) => {
            if md.is_dir() {
                let mut entities: Vec<Entity> = Vec::new();
                for entry in WalkDir::new(source) {
                    match entry {
                        Err(e) => {
                            println!("{}", e);
                            exit(1)
                        }
                        Ok(entry) => {
                            if is_rust_module(&entry) {
                                let path =
                                    entry.path().to_str().expect("fail to access rust module");
                                match parse_syntax(path) {
                                    Ok(syntax) => {
                                        let mut ent = file_parser(syntax);
                                        entities.append(&mut ent);
                                    }
                                    Err(e) => {
                                        println!("Failed to parse syntax for {}: {}", path, e);
                                    }
                                }
                            }
                        }
                    }
                }

                if File::open(file_name).is_ok() {
                    std::fs::remove_file(file_name).expect("Unable to remove file");
                }

                let file = File::create(file_name);
                let mut file = file.expect("Unable to create file");

                // Write some text to the file
                file.write_all(render_plantuml(entities).as_bytes())
                    .expect("Unable to write to file");

                // Flush the file to ensure all data is written
                let _ = file.flush();
                exit(0)
            }
        }
    }

    fn parse_syntax(path: &str) -> Result<syn::File, Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;
        let mut src = String::new();
        file.read_to_string(&mut src)?;
        let syntax = syn::parse_file(&src)?;
        Ok(syntax)
    }

    fn is_rust_module(entry: &DirEntry) -> bool {
        let path: String = entry
            .file_name()
            .to_str()
            .unwrap_or("")
            .chars()
            .rev()
            .take(3)
            .collect();
        &path == "sr."
    }
}
