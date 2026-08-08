use std::path::Path;

use opencase::{cmd_init, cmd_record, cmd_report, cmd_review, cmd_run, cmd_scriptify, cmd_skill_install, validate_dir};

fn usage() -> ! {
    eprintln!(
        "opencase — declarative test-case management\n\n\
         Usage:\n  opencase [--cases <dir>] <command> [args]\n\n\
         Commands:\n  init                            scaffold a cases/ directory with an example case\n  \
         skill install                  install the embedded agent skills (--agent pi|claude|codex|project)\n  \
         validate                        check all cases against schema and state machine\n  \
         review [id]                     list draft cases; review <id> --approve | --edit\n  \
         run <id>                        print the execution prompt (gate: reviewed)\n  \
         record <id> --result <pass|fail> [--category <cat>] [--commit <sha>] [--note <text>]\n  \
         report                          markdown summary of status, coverage, failures\n  \
         scriptify <id> [--covered-by <path>] [--rebaseline]   print conversion context, flip case to scripted; --rebaseline refreshes the drift baseline"
    );
    std::process::exit(2);
}

fn die(e: String) -> ! {
    eprintln!("opencase: {e}");
    std::process::exit(1);
}

fn main() {
    let mut args = std::env::args().skip(1);
    let mut cases = "cases".to_string();
    let mut cmd: Option<String> = None;
    let mut rest: Vec<String> = Vec::new();
    while let Some(a) = args.next() {
        match a.as_str() {
            "--cases" => cases = args.next().unwrap_or_else(|| usage()),
            "validate" | "review" | "run" | "record" | "report" | "scriptify" | "init" | "skill" => {
                cmd = Some(a);
                rest = args.collect();
                break;
            }
            other => {
                eprintln!("opencase: unknown argument '{other}'");
                usage();
            }
        }
    }
    let cmd = cmd.unwrap_or_else(|| usage());

    match cmd.as_str() {
        "validate" => {
            let (count, problems) = validate_dir(Path::new(&cases));
            for p in &problems {
                println!("{p}");
            }
            println!("{count} case(s), {} problem(s)", problems.len());
            if !problems.is_empty() {
                std::process::exit(1);
            }
        }
        "review" => {
            let mut id = None;
            let mut approve = false;
            let mut edit = false;
            for a in &rest {
                match a.as_str() {
                    "--approve" => approve = true,
                    "--edit" => edit = true,
                    other if other.starts_with('-') => {
                        eprintln!("opencase: unknown flag '{other}'");
                        usage();
                    }
                    other => id = Some(other.to_string()),
                }
            }
            match cmd_review(Path::new(&cases), id.as_deref(), approve, edit) {
                Ok(msg) => println!("{msg}"),
                Err(e) => die(e),
            }
        }
        "run" => {
            let id = rest.first().cloned().unwrap_or_else(|| usage());
            match cmd_run(Path::new(&cases), &id) {
                Ok(msg) => println!("{msg}"),
                Err(e) => die(e),
            }
        }
        "record" => {
            let mut id = None;
            let mut result = None;
            let mut category = None;
            let mut commit = None;
            let mut note = None;
            let mut i = 0;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--result" | "--category" | "--commit" | "--note" => {
                        let flag = rest[i].clone();
                        i += 1;
                        let val = rest.get(i).cloned().unwrap_or_else(|| usage());
                        match flag.as_str() {
                            "--result" => result = Some(val),
                            "--category" => category = Some(val),
                            "--commit" => commit = Some(val),
                            _ => note = Some(val),
                        }
                    }
                    other if other.starts_with('-') => {
                        eprintln!("opencase: unknown flag '{other}'");
                        usage();
                    }
                    other => id = Some(other.to_string()),
                }
                i += 1;
            }
            let id = id.unwrap_or_else(|| usage());
            let result = result.unwrap_or_else(|| usage());
            match cmd_record(
                Path::new(&cases),
                &id,
                &result,
                category.as_deref(),
                commit.as_deref(),
                note.as_deref(),
            ) {
                Ok(msg) => println!("{msg}"),
                Err(e) => die(e),
            }
        }
        "report" => match cmd_report(Path::new(&cases)) {
            Ok(msg) => print!("{msg}"),
            Err(e) => die(e),
        },
        "scriptify" => {
            let mut id = None;
            let mut covered_by = None;
            let mut rebaseline = false;
            let mut i = 0;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--covered-by" => {
                        i += 1;
                        covered_by = Some(rest.get(i).cloned().unwrap_or_else(|| usage()));
                    }
                    "--rebaseline" => rebaseline = true,
                    other if other.starts_with('-') => {
                        eprintln!("opencase: unknown flag '{other}'");
                        usage();
                    }
                    other => id = Some(other.to_string()),
                }
                i += 1;
            }
            let id = id.unwrap_or_else(|| usage());
            match cmd_scriptify(Path::new(&cases), &id, covered_by.as_deref(), rebaseline) {
                Ok(msg) => println!("{msg}"),
                Err(e) => die(e),
            }
        }
        "init" => match cmd_init(Path::new(&cases)) {
            Ok(msg) => println!("{msg}"),
            Err(e) => die(e),
        },
        "skill" => {
            let mut sub = None;
            let mut agent = None;
            let mut force = false;
            let mut dir = None;
            let mut i = 0;
            while i < rest.len() {
                match rest[i].as_str() {
                    "install" => sub = Some("install".to_string()),
                    "--agent" => {
                        i += 1;
                        agent = Some(rest.get(i).cloned().unwrap_or_else(|| usage()));
                    }
                    "--dir" => {
                        i += 1;
                        dir = Some(rest.get(i).cloned().unwrap_or_else(|| usage()));
                    }
                    "--force" => force = true,
                    other if other.starts_with('-') => {
                        eprintln!("opencase: unknown flag '{other}'");
                        usage();
                    }
                    other => sub = Some(other.to_string()),
                }
                i += 1;
            }
            if sub.as_deref() != Some("install") {
                eprintln!("opencase: skill subcommands: install (--agent pi|claude|codex|project, --force, --dir <path>)");
                std::process::exit(2);
            }
            match cmd_skill_install(
                agent.as_deref().unwrap_or("pi"),
                force,
                dir.as_deref(),
            ) {
                Ok(msg) => println!("{msg}"),
                Err(e) => die(e),
            }
        }
        _ => usage(),
    }
}
