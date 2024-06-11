use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::{NOTHING, UTF8_FULL};
use comfy_table::{Cell, ContentArrangement, Table};
use crossterm::style::Stylize;
use inquire::ui::{Attributes, RenderConfig, StyleSheet};
use inquire::{error::InquireResult, max_length, min_length, Confirm, Select, Text};
use regex::Regex;

use crate::shell::Shell;

static MAX_HEADER_LEN: usize = 72;

fn log_error(msg: impl std::fmt::Display) -> () {
    let msg = format!("{}: {msg}", "ERROR".red());
    println!("{msg}")
}

#[derive(Clone, Debug)]
pub struct CommitBuilder {
    pub message: String,
    pub body: Option<String>,
    pub footer: Option<String>,
}

impl CommitBuilder {
    pub fn new() -> Self {
        inquire::set_global_render_config(RenderConfig {
            prompt: StyleSheet::new().with_attr(Attributes::BOLD),
            ..Default::default()
        });

        Self {
            message: String::new(),
            body: None,
            footer: None,
        }
    }

    pub fn prompt_type(mut self) -> InquireResult<Self> {
        let ct = Select::new(
            "Select the type of change you're committing:",
            get_commit_types(),
        )
        .with_page_size(10)
        .with_formatter(&|o| format!("{}: {}", o.value._type, o.value.description))
        .prompt()
        .inspect_err(|e| {
            log_error(e);
        })?;

        let _type = ct._type;
        let new_commit_message = format!("{}:", _type);
        self.message = new_commit_message;

        return Ok(self);
    }

    pub fn prompt_jira(mut self, skip: bool) -> InquireResult<Self> {
        if skip {
            return Ok(self);
        }

        let regex = Regex::new(r"(?m)^(?<jiraIssue>[a-zA-Z0-9]+-\d+)").unwrap();
        let res = Shell::new("git")
            .arg("branch")
            .arg("--show-current")
            .exec()
            .map_err(|e| {
                log_error(e);
                Vec::<String>::new()
            })
            .unwrap();

        let empty = "".to_string();
        let current_branch = res.get(0).unwrap_or(&empty);
        let default = match regex.captures(current_branch) {
            Some(v) => v.name("jiraIssue").map_or("", |m| m.as_str()),
            None => "",
        };

        let _jira = Text::new("Enter JIRA issue (DAZ-12345):")
            .with_validator(min_length!(1, "You must enter a JIRA issue"))
            .with_default(default)
            .prompt()?;

        self.message = format!("{} [{}]", self.message, _jira);

        Ok(self)
    }

    pub fn prompt_header(mut self) -> InquireResult<Self> {
        let curr_commit_str = &self.message;
        let prompt: Vec<String> = vec![
            "Write a short, imperative tense description of the change:"
                .bold()
                .to_string(),
            "[------------------------------------------------------------------------]"
                .to_string(),
            curr_commit_str.clone(),
        ];

        let remaining_len = MAX_HEADER_LEN - curr_commit_str.chars().count();

        let _header = Text::new(prompt.join("\n").as_str())
            .with_render_config(RenderConfig::default_colored())
            .with_validator(min_length!(1, "You must have a commit message"))
            .with_validator(max_length!(
                remaining_len,
                format!("Your commit message should be less than {MAX_HEADER_LEN} characters")
            ))
            .prompt()
            .inspect_err(|e| {
                log_error(e);
            })?;

        self.message += format!(" {_header}").as_str();

        return Ok(self);
    }

    pub fn prompt_body(mut self) -> InquireResult<Self> {
        let _body = Text::new("Provide a longer description of the change: (press ESC to skip)\n")
            .prompt_skippable()
            .inspect_err(|e| {
                log_error(e);
            })?;

        self.body = _body;

        return Ok(self);
    }

    pub fn prompt_breaking_change(mut self) -> InquireResult<Self> {
        let _breaking_change = Confirm::new("Are there any breaking changes?")
            .with_default(false)
            .prompt()
            .inspect_err(|e| {
                log_error(e);
            })?;

        if _breaking_change {
            let _description = Text::new("Describe the breaking changes:\n")
                .with_validator(min_length!(1, "You must describe the breaking changes"))
                .prompt()
                .inspect_err(|e| {
                    log_error(e);
                })?;

            self.footer = Some(format!("BREAKING CHANGE: {_description}"));
        }

        return Ok(self);
    }

    pub fn prompt_confirm(self) -> InquireResult<Self> {
        println_preview(&self);

        Confirm::new("Are you sure that you want to commit?")
            .with_default(true)
            .prompt()
            .inspect_err(|e| {
                log_error(e);
            })?;

        return Ok(self);
    }
}

impl std::fmt::Display for CommitBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = &self.message;
        let body = self.body.to_owned().unwrap_or("".to_string());
        let footer = self.footer.to_owned().unwrap_or("".to_string());

        let trimmed = vec![message.clone(), body.clone(), footer.clone()]
            .into_iter()
            .filter(|v| !v.is_empty())
            .map(|v| v.clone())
            .collect::<Vec<String>>()
            .join("\n\n")
            .trim()
            .to_string();

        write!(f, "{trimmed}")
    }
}

fn println_preview(c: &CommitBuilder) -> () {
    println!("{}", "Commit Preview:\n".to_string());

    let commit_preview = format!("\n{c}\n");

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(MAX_HEADER_LEN.try_into().unwrap())
        .add_row(vec![
            Cell::new(commit_preview.as_str()).fg(comfy_table::Color::Green)
        ]);
    table
        .column_mut(0)
        .expect("There should be a column")
        .set_constraint(comfy_table::ColumnConstraint::LowerBoundary(
            comfy_table::Width::Fixed(50),
        ))
        .set_padding((3, 3));

    println!("{table}\n");
}

pub fn pretty_print(c: &CommitBuilder) -> String {
    let mut table = Table::new();
    table
        .load_preset(NOTHING)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(MAX_HEADER_LEN.try_into().unwrap())
        .add_row(vec![Cell::new(c.to_string().as_str())]);

    table
        .column_mut(0)
        .expect("There should be a column")
        .set_padding((0, 0));

    format!("{table}")
}
struct CommitType {
    _type: &'static str,
    description: &'static str,
}

impl CommitType {
    pub fn new(_type: &'static str, description: &'static str) -> Self {
        Self { _type, description }
    }
}

impl std::fmt::Display for CommitType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let type_str = format!("{}:", self._type);
        let description = self.description;
        write!(f, "{type_str:<10} {description}")
    }
}

fn get_commit_types() -> Vec<CommitType> {
    return vec![
        CommitType::new("feat", "A new feature"),
        CommitType::new("fix", "A bug fix"),
        CommitType::new("docs", "Documentation only changes"),
        CommitType::new("style", "Changes that do not affect the meaning of the code (white-space, formatting, missing semi-colons, etc)"),
        CommitType::new("refactor", "A code change that neither fixes a bug nor adds a feature"),
        CommitType::new("revert", "Reverts a previous commit"),
        CommitType::new("perf", "A code change that improves performance"),
        CommitType::new("test", "Adding missing or correcting existing tests"),
        CommitType::new("chore", "Changes to the build process or auxiliary tools and libraries such as documentation generation"),
    ];
}

#[cfg(test)]
mod tests {
    use super::CommitBuilder;

    #[test]
    fn should_format_commit_str() {
        let mut c = CommitBuilder {
            message: "feat: something new".to_string(),
            body: Some("Something new.".to_string()),
            footer: Some("BREAKING CHANGE: Something new.".to_string()),
        };

        assert_eq!(
            c.to_string(),
            r#"feat: something new

Something new.

BREAKING CHANGE: Something new."#
        );

        c = CommitBuilder {
            message: "feat: something new".to_string(),
            body: Some("Something new.".to_string()),
            footer: None,
        };

        assert_eq!(
            c.to_string(),
            r#"feat: something new

Something new."#
        );

        c = CommitBuilder {
            message: "feat: something new".to_string(),
            body: None,
            footer: None,
        };

        assert_eq!(c.to_string(), r#"feat: something new"#)
    }
}
