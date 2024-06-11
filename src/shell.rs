use log::debug;
use std::{
    ffi::{OsStr, OsString},
    process::Command,
};

use crate::error::{FriseError, FriseResult};

pub struct Shell {
    program: OsString,
    args: Vec<OsString>,
}

impl Shell {
    pub fn new<S: AsRef<OsStr>>(program: S) -> Shell {
        Shell {
            program: program.as_ref().into(),
            args: vec![],
        }
    }

    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Shell {
        self.args.push(arg.as_ref().into());
        self
    }

    #[allow(dead_code)]
    pub fn option_arg<S: AsRef<OsStr>>(&mut self, arg: Option<S>) -> &mut Shell {
        if arg.is_some() {
            self.args.push(arg.unwrap().as_ref().into());
        }
        self
    }

    pub fn exec(&self) -> FriseResult<Vec<String>> {
        debug!(
            "Executing {:?} with args {:?} in cwd",
            self.program, self.args
        );

        let output = Command::new(&self.program).args(&self.args).output()?;

        if !output.status.success() {
            let err = String::from_utf8(output.stderr)?;
            return Err(FriseError::Custom(err));
        }

        let result = String::from_utf8(output.stdout)?;

        return Ok(result
            .lines()
            .map(|l| l.to_string())
            .collect::<Vec<String>>());
    }

    pub fn spawn(&self) -> FriseResult<()> {
        debug!(
            "Spawning {:?} with args {:?} in cwd",
            self.program, self.args
        );

        Command::new(&self.program)
            .args(&self.args)
            .spawn()?
            .wait()?;

        Ok(())
    }
}
