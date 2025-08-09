use crate::installer::{self, processor::InstallResult};
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
    pub fn add_install_result(&mut self, result: InstallResult) {
        match result {
            InstallResult::Created => self.created += 1,
            InstallResult::Updated => self.updated += 1,
            InstallResult::Skipped => self.already_existed += 1,
        }
    }

    pub fn total_count(&self) -> usize {
        let Self {
            created,
            removed,
            updated,
            errors,
            already_existed,
        } = self;
        created + updated + removed + errors + already_existed
    }

    pub fn to_string(&self, action: installer::Action) -> String {
        let total = self.total_count();
        let Self {
            created,
            removed,
            updated,
            errors,
            already_existed,
        } = self;
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
