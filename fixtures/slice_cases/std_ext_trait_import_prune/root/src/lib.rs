use opensourced::opensourced;

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;

#[opensourced]
pub fn selected_exit_code(status: ExitStatus) -> Option<i32> {
    status.code()
}

#[cfg(unix)]
pub fn dead_exit_status_from_raw(code: u32) -> ExitStatus {
    ExitStatus::from_raw((code as i32) << 8)
}

