use clap::{arg, command, Parser};

use crate::{
    commit::{pretty_print, CommitBuilder},
    error::{FriseError, FriseResult},
    git::{do_commit, is_clean},
};

#[derive(Parser)]
#[command(name = "frise")]
#[command(version = "0.1")]
#[command(about, long_about = None)]
pub struct Cli {
    /// Turn debugging information on.
    #[arg(long, value_enum, default_value = "false", global = true, hide = true)]
    pub debug: bool,

    /// Build commit message, but do not commit the staged changes.
    #[arg(long, default_value = "false")]
    dry_run: bool,

    /// Disabled jira check
    #[arg(long, default_value = "false")]
    skip_jira: bool,
}

pub fn run() -> FriseResult<()> {
    let cli = Cli::parse();

    let min_level = match cli.debug {
        true => log::LevelFilter::Debug,
        false => log::LevelFilter::Error,
    };

    env_logger::Builder::new().filter_level(min_level).init();

    let clean = is_clean()?;

    if clean {
        return Err(FriseError::Custom(
            "No files added to staging! Did you forget to run git add?".to_string(),
        ));
    }
    let msg = CommitBuilder::new()
        .prompt_type()?
        .prompt_jira(cli.skip_jira)?
        .prompt_header()?
        .prompt_body()?
        .prompt_breaking_change()?
        .prompt_confirm()?;

    let pretty = pretty_print(&msg);

    if cli.dry_run {
        println!("{pretty}");
    } else {
        do_commit(&pretty)?;
    }

    return Ok(());
}
