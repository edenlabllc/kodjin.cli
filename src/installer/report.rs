use console::style;
use std::fmt;

#[derive(Default, Debug)]
pub struct InstallReport {
    pub created: usize,
    pub errors: usize,
    pub already_existed: usize,
}

impl fmt::Display for InstallReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            created,
            errors,
            already_existed,
        } = self;
        let total = created + errors + already_existed;
        write!(
            f,
            "{} resources processed, {} created, {} errors, and {} already existed",
            style(total).bold(),
            style(created).bold(),
            style(errors).bold(),
            style(already_existed).bold(),
        )
    }
}
