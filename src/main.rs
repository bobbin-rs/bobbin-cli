#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
extern crate clap;
extern crate termcolor;

pub mod printer;

use clap::{Arg, App, SubCommand};
//use std::io::Write;


use errors::*;
mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! {
        links {            
        }
        foreign_links {
            Io(::std::io::Error);
        }
    }
}

fn main() {
    if let Err(ref e) = run() {
        use ::std::io::Write;
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "error: {}", e).expect(errmsg);

        for e in e.iter().skip(1) {
            writeln!(stderr, "caused by: {}", e).expect(errmsg);
        }

        // The backtrace is not always generated. Try to run this example
        // with `RUST_BACKTRACE=1`.
        if let Some(backtrace) = e.backtrace() {
            writeln!(stderr, "backtrace: {:?}", backtrace).expect(errmsg);
        }

        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = App::new("bobbin")
        .version("0.1")
        .arg(Arg::with_name("verbose").long("verbose").short("v"))
        .arg(Arg::with_name("device").long("device").short("d").takes_value(true))
        .subcommand(SubCommand::with_name("list")
            .arg(Arg::with_name("all").long("all"))
        )
        .subcommand(SubCommand::with_name("load")            
            .arg(Arg::with_name("target").long("target").takes_value(true))
            .arg(Arg::with_name("bin").long("bin").takes_value(true))
            .arg(Arg::with_name("example").long("example").takes_value(true))
            .arg(Arg::with_name("release").long("release"))
            .arg(Arg::with_name("features").long("features"))
            .arg(Arg::with_name("console").long("console").min_values(0).max_values(1))
        )
        .subcommand(SubCommand::with_name("run")            
            .arg(Arg::with_name("target").long("target").takes_value(true))
            .arg(Arg::with_name("bin").long("bin").takes_value(true))
            .arg(Arg::with_name("example").long("example").takes_value(true))
            .arg(Arg::with_name("release").long("release"))
            .arg(Arg::with_name("features").long("features").takes_value(true))
            .arg(Arg::with_name("console").long("console").min_values(0).max_values(1))
        )
        .subcommand(SubCommand::with_name("halt"))
        .subcommand(SubCommand::with_name("resume")
            .arg(Arg::with_name("console").long("console").min_values(0).max_values(1))
        )
        .subcommand(SubCommand::with_name("reset")
            .arg(Arg::with_name("run").long("run"))
            .arg(Arg::with_name("halt").long("halt"))
            .arg(Arg::with_name("init").long("init"))
            .arg(Arg::with_name("console").long("console").min_values(0).max_values(1))
        )
        .subcommand(SubCommand::with_name("console")
            .arg(Arg::with_name("console").long("console").min_values(0).max_values(1))
        )
        .get_matches();

    
    // if let Some(cmd_args) = args.subcommand_matches("list") {        
    //     try!(cmd_list(&args, cmd_args));
    // } else if let Some(cmd_args) = args.subcommand_matches("load") {
    //     try!(cmd_device(&args, cmd_args));
    // } else if let Some(cmd_args) = args.subcommand_matches("run") {
    //     try!(cmd_device(&args, cmd_args));
    // } else if let Some(cmd_args) = args.subcommand_matches("halt") {
    //     try!(cmd_device(&args, cmd_args));
    // } else if let Some(cmd_args) = args.subcommand_matches("resume") {
    //     try!(cmd_device(&args, cmd_args));
    // } else if let Some(cmd_args) = args.subcommand_matches("reset") {
    //     try!(cmd_device(&args, cmd_args));
    // } else if let Some(cmd_args) = args.subcommand_matches("console") {
    //     try!(cmd_console(&args, cmd_args));
    // } else if let Some(cmd_args) = args.subcommand_matches("debug") {
    //     try!(cmd_debug(&args, cmd_args));
    // } else {
    //     println!("{}", args.usage());
    // }
    let _ = args;

    let mut p = printer::printer().with_verbose(args.is_present("verbose"));

    p.verbose("Debug", "Verbose Message")?;
    p.info("Testing", "Hello, World!")?;
    p.error("Error","Code Red!")?;
    Ok(())
}
