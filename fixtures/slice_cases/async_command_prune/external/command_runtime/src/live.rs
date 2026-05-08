pub struct CommandJob {
    id: u64,
    label: String,
}

impl CommandJob {
    pub fn new(raw: &str) -> Self {
        Self {
            id: raw.len() as u64,
            label: raw.trim().to_string(),
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn dead_method(&self) -> String {
        format!("dead-command:{}", self.label)
    }
}

pub enum WorkerCommand {
    Start(CommandJob),
}

pub struct WorkerState {
    prefix: String,
}

impl WorkerState {
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
        }
    }

    pub fn render_job(&self, job: &CommandJob) -> String {
        format!("{}:{}:{}", self.prefix, job.id, job.label())
    }

    pub fn dead_method(&self) -> String {
        format!("dead-state:{}", self.prefix)
    }
}

pub async fn handle_command(state: &WorkerState, command: WorkerCommand) -> String {
    match command {
        WorkerCommand::Start(job) => state.render_job(&job),
    }
}

pub async fn selected_command(raw: &str) -> String {
    let state = WorkerState::new("live");
    handle_command(&state, WorkerCommand::Start(CommandJob::new(raw))).await
}

pub async fn dead_live_command(raw: &str) -> String {
    CommandJob::new(raw).dead_method()
}
