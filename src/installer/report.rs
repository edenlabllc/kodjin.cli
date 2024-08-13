#[derive(Default)]
pub struct InstallReport {
    pub created: usize,
    pub errors: usize,
    pub already_existed: usize,
}
