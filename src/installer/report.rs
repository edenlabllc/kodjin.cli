use crate::installer;
use console::style;

#[derive(Default, Debug)]
pub struct InstallReport {
    pub created: usize,
    pub updated: usize,
    pub removed: usize,
    pub errors: usize,
    pub already_existed: usize,
}

impl InstallReport {
    pub fn to_string(&self, action: installer::Action) -> String {
        let Self {
            created,
            removed,
            updated,
            errors,
            already_existed,
        } = self;
        let total = created + removed + errors + already_existed;
        match action {
            installer::Action::Install => {
                format!(
                    "{} resources processed, {} created, {} updated, {} errors, and {} already existed",
                    style(total).bold(),
                    style(created).bold(),
                    style(updated).bold(),
                    style(errors).bold(),
                    style(already_existed).bold(),
                )
            }
            installer::Action::Uninstall => {
                format!(
                    "{} resources processed, {} removed, {} errors",
                    style(total).bold(),
                    style(removed).bold(),
                    style(errors).bold(),
                )
            }
        }
    }
}
