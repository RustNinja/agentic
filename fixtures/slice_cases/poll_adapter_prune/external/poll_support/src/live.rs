use std::task::Poll;

pub struct PollFrame {
    payload: String,
}

impl PollFrame {
    pub fn render(&self) -> String {
        format!("poll:{}", self.payload)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-frame:{}", self.payload)
    }
}

pub struct Poller {
    payload: Option<String>,
}

impl Poller {
    pub fn new(raw: &str) -> Self {
        Self {
            payload: Some(raw.trim().to_string()),
        }
    }

    pub fn poll_next(&mut self) -> Poll<Option<PollFrame>> {
        Poll::Ready(self.payload.take().map(|payload| PollFrame { payload }))
    }

    pub fn dead_method(&mut self) -> Poll<Option<PollFrame>> {
        Poll::Ready(Some(PollFrame {
            payload: "dead-frame".to_string(),
        }))
    }
}

pub fn selected_poll(raw: &str) -> String {
    let mut poller = Poller::new(raw);
    match poller.poll_next() {
        Poll::Ready(Some(frame)) => frame.render(),
        Poll::Ready(None) => "empty".to_string(),
        Poll::Pending => "pending".to_string(),
    }
}

pub fn dead_live_poll(raw: &str) -> String {
    let mut poller = Poller::new(raw);
    match poller.dead_method() {
        Poll::Ready(Some(frame)) => frame.dead_method(),
        _ => "dead-pending".to_string(),
    }
}
