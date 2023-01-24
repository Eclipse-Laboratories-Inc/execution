use {
    clap::{value_t, ArgMatches},
    std::{
        path::{PathBuf},
        process::exit,
    },
};

pub fn parse_ledger_path(matches: &ArgMatches<'_>, name: &str) -> PathBuf {
    PathBuf::from(value_t!(matches, name, String).unwrap_or_else(|_err| {
        eprintln!(
            "Error: Missing --ledger <DIR> argument.\n\n{}",
            matches.usage()
        );
        exit(1);
    }))
}
