use opensourced::opensourced;

#[cfg(target_os = "android")]
mod android_logcat {
    use std::fmt::{self, Write};

    const ANDROID_LOG_DEBUG: i32 = 3;
    const ANDROID_LOG_INFO: i32 = 4;

    pub(crate) struct AndroidLogMakeWriter;

    pub(crate) struct AndroidLogWriter {
        priority: i32,
        buffer: String,
    }

    impl AndroidLogWriter {
        fn new(priority: i32) -> Self {
            Self {
                priority,
                buffer: String::new(),
            }
        }
    }

    impl Write for AndroidLogWriter {
        fn write_str(&mut self, line: &str) -> fmt::Result {
            self.buffer.push_str(line);
            Ok(())
        }
    }

    impl AndroidLogMakeWriter {
        pub(crate) fn make_writer(&self) -> AndroidLogWriter {
            AndroidLogWriter::new(ANDROID_LOG_INFO)
        }
    }

    fn priority_for_level(debug: bool) -> i32 {
        if debug {
            ANDROID_LOG_DEBUG
        } else {
            ANDROID_LOG_INFO
        }
    }

    fn write_android_log(priority: i32, line: &str) -> usize {
        priority as usize + line.len()
    }

    pub(crate) fn log_line(line: &str) -> usize {
        let mut writer = AndroidLogMakeWriter.make_writer();
        let _ = writer.write_str(line);
        write_android_log(priority_for_level(false), &writer.buffer)
    }

    pub(crate) fn dead_android_helper(line: &str) -> usize {
        write_android_log(ANDROID_LOG_DEBUG, line)
    }
}

#[opensourced]
pub fn selected_log_message(message: &str) -> usize {
    #[cfg(target_os = "android")]
    {
        android_logcat::log_line(message)
    }
    #[cfg(not(target_os = "android"))]
    {
        message.len()
    }
}

pub fn dead_log_message(message: &str) -> usize {
    message.len() + 1
}

