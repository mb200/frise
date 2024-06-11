use crate::{error::FriseResult, shell::Shell};

pub fn is_clean() -> FriseResult<bool> {
    let res = Shell::new("git")
        .arg("diff")
        .arg("--cached")
        .arg("--no-ext-diff")
        .arg("--name-only")
        .exec()?;

    if res.is_empty() {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn do_commit(msg: &String) -> FriseResult<()> {
    Shell::new("git").arg("commit").arg("-m").arg(msg).spawn()?;

    Ok(())
}
