//! Own the native compiler and its descendants until completion or failure.
use std::process::{Child, Command, ExitStatus};

pub(super) struct Compiler {
    child: Child,
    terminated: bool,
    #[cfg(windows)]
    job: windows::Job,
}
impl Compiler {
    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>, String> {
        if self.terminated {
            return Err("compiler process already terminated".into());
        }
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            // Keep the exited group leader unreaped until terminate(): its PID
            // cannot be recycled before descendant cleanup targets the group.
            let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
            loop {
                let result = unsafe {
                    libc::waitid(libc::P_PID, self.child.id(), &mut info, libc::WEXITED | libc::WNOHANG | libc::WNOWAIT)
                };
                if result == 0 {
                    break;
                }
                let error = std::io::Error::last_os_error();
                if error.kind() != std::io::ErrorKind::Interrupted {
                    return Err(error.to_string());
                }
            }
            // SAFETY: successful waitid initializes the child status fields.
            unsafe {
                if info.si_pid() == 0 {
                    return Ok(None);
                }
                let status = info.si_status();
                let raw = if info.si_code == libc::CLD_EXITED {
                    status << 8
                } else {
                    status | if info.si_code == libc::CLD_DUMPED { 0x80 } else { 0 }
                };
                Ok(Some(ExitStatus::from_raw(raw)))
            }
        }
        #[cfg(windows)]
        self.child.try_wait().map_err(|e| e.to_string())
    }
    pub fn spawn(command: &mut Command) -> Result<Self, String> {
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
            Ok(Self { child: command.spawn().map_err(|e| e.to_string())?, terminated: false })
        }
        #[cfg(windows)]
        {
            use std::os::windows::{io::AsRawHandle, process::CommandExt};
            let job = windows::Job::new()?;
            // Suspend before assigning the job: cc1/as/ld cannot escape before
            // process-tree ownership has been established. No visible window.
            command.creation_flags(0x00000004 | 0x08000000);
            let mut child = command.spawn().map_err(|e| e.to_string())?;
            if let Err(error) = job.assign_and_resume(child.as_raw_handle(), child.id()) {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error);
            }
            Ok(Self { child, job, terminated: false })
        }
    }
    pub fn terminate(&mut self) {
        if self.terminated {
            return;
        }
        self.terminated = true;
        #[cfg(unix)]
        // SAFETY: spawn established this child's separate process group.
        unsafe {
            libc::kill(-(self.child.id() as i32), libc::SIGKILL);
        }
        #[cfg(windows)]
        self.job.terminate();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
impl Drop for Compiler {
    fn drop(&mut self) {
        // Also clean up descendants when process observation or publication fails.
        self.terminate();
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    use std::{
        fs,
        time::{Duration, Instant, SystemTime, UNIX_EPOCH},
    };
    // Real child processes distinguish descendant cleanup from merely reaping
    // the compiler leader. No shell text contains user-controlled values.
    #[test]
    fn natural_exit_and_explicit_termination_close_the_owned_group_once() {
        for finished in [false, true] {
            let suffix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            let path = std::env::temp_dir().join(format!("sm-compiler-child-{}-{suffix}", std::process::id()));
            let mut command = Command::new("sh");
            command
                .args([
                    "-c",
                    if finished { "sleep 30 & echo $! > \"$1\"; exit 7" } else { "sleep 30 & echo $! > \"$1\"; wait" },
                    "compiler-test",
                ])
                .arg(&path)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null());
            let mut process = Compiler::spawn(&mut command).unwrap();
            let start = Instant::now();
            let descendant: i32 = loop {
                if let Ok(text) = fs::read_to_string(&path)
                    && let Ok(pid) = text.trim().parse()
                {
                    break pid;
                }
                assert!(start.elapsed() < Duration::from_secs(5));
                std::thread::sleep(Duration::from_millis(10));
            };
            if finished {
                loop {
                    if let Some(status) = process.try_wait().unwrap() {
                        assert_eq!(status.code(), Some(7));
                        break;
                    }
                    assert!(start.elapsed() < Duration::from_secs(5));
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
            process.terminate();
            process.terminate();
            assert!(process.try_wait().is_err());
            drop(process);
            let start = Instant::now();
            loop {
                let state = fs::read_to_string(format!("/proc/{descendant}/stat"));
                if state.as_ref().is_err()
                    || state.as_ref().is_ok_and(|s| {
                        s.rsplit_once(')').is_some_and(|(_, fields)| fields.trim_start().starts_with('Z'))
                    })
                {
                    break;
                }
                assert!(start.elapsed() < Duration::from_secs(5), "compiler descendant remained alive");
                std::thread::sleep(Duration::from_millis(10));
            }
            fs::remove_file(path).unwrap();
        }
    }
}

#[cfg(windows)]
mod windows {
    use std::{
        ffi::c_void,
        io,
        mem::{size_of, zeroed},
        ptr,
    };
    type Handle = *mut c_void;
    #[repr(C)]
    struct BasicLimits {
        process_time: i64,
        job_time: i64,
        flags: u32,
        minimum_working_set: usize,
        maximum_working_set: usize,
        active_processes: u32,
        affinity: usize,
        priority: u32,
        scheduling: u32,
    }
    #[repr(C)]
    struct IoCounters {
        read: u64,
        write: u64,
        other: u64,
        read_bytes: u64,
        write_bytes: u64,
        other_bytes: u64,
    }
    #[repr(C)]
    struct ExtendedLimits {
        basic: BasicLimits,
        io: IoCounters,
        process_memory: usize,
        job_memory: usize,
        peak_process_memory: usize,
        peak_job_memory: usize,
    }
    #[repr(C)]
    struct ThreadEntry {
        size: u32,
        usage: u32,
        id: u32,
        owner: u32,
        base_priority: i32,
        delta_priority: i32,
        flags: u32,
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateJobObjectW(attributes: *const c_void, name: *const u16) -> Handle;
        fn SetInformationJobObject(job: Handle, class: i32, info: *const c_void, size: u32) -> i32;
        fn AssignProcessToJobObject(job: Handle, process: Handle) -> i32;
        fn TerminateJobObject(job: Handle, exit_code: u32) -> i32;
        fn CloseHandle(handle: Handle) -> i32;
        fn CreateToolhelp32Snapshot(flags: u32, process: u32) -> Handle;
        fn Thread32First(snapshot: Handle, entry: *mut ThreadEntry) -> i32;
        fn Thread32Next(snapshot: Handle, entry: *mut ThreadEntry) -> i32;
        fn OpenThread(access: u32, inherit: i32, id: u32) -> Handle;
        fn ResumeThread(thread: Handle) -> u32;
    }
    struct OwnedHandle(Handle);
    impl Drop for OwnedHandle {
        fn drop(&mut self) {
            // SAFETY: this object owns a successful Windows handle allocation.
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
    pub(super) struct Job(OwnedHandle);
    impl Job {
        pub fn new() -> Result<Self, String> {
            // SAFETY: null security/name parameters request a private job; limits
            // use the documented C ABI and exact initialized structure size.
            unsafe {
                let handle = CreateJobObjectW(ptr::null(), ptr::null());
                if handle.is_null() {
                    return Err(format!("create compiler job: {}", io::Error::last_os_error()));
                }
                let job = Self(OwnedHandle(handle));
                let mut limits: ExtendedLimits = zeroed();
                limits.basic.flags = 0x00002000; // JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
                if SetInformationJobObject(
                    handle,
                    9,
                    (&limits as *const ExtendedLimits).cast(),
                    size_of::<ExtendedLimits>() as u32,
                ) == 0
                {
                    return Err(format!("configure compiler job: {}", io::Error::last_os_error()));
                }
                Ok(job)
            }
        }
        pub fn assign_and_resume(&self, process: Handle, process_id: u32) -> Result<(), String> {
            // SAFETY: caller owns a still-suspended Child. Assignment precedes any
            // compiler execution. A new suspended process has one primary thread.
            unsafe {
                if AssignProcessToJobObject(self.0.0, process) == 0 {
                    return Err(format!("assign compiler job: {}", io::Error::last_os_error()));
                }
                let snapshot = CreateToolhelp32Snapshot(4, 0); // TH32CS_SNAPTHREAD
                if snapshot == -1isize as Handle {
                    return Err(format!("inspect compiler thread: {}", io::Error::last_os_error()));
                }
                let snapshot = OwnedHandle(snapshot);
                let mut entry: ThreadEntry = zeroed();
                entry.size = size_of::<ThreadEntry>() as u32;
                let mut found = Thread32First(snapshot.0, &mut entry);
                while found != 0 {
                    if entry.owner == process_id {
                        let thread = OpenThread(2, 0, entry.id); // THREAD_SUSPEND_RESUME
                        if thread.is_null() {
                            return Err(format!("open compiler thread: {}", io::Error::last_os_error()));
                        }
                        let thread = OwnedHandle(thread);
                        if ResumeThread(thread.0) == u32::MAX {
                            return Err(format!("resume compiler: {}", io::Error::last_os_error()));
                        }
                        return Ok(());
                    }
                    found = Thread32Next(snapshot.0, &mut entry);
                }
                Err("suspended compiler primary thread not found".into())
            }
        }
        pub fn terminate(&self) {
            // SAFETY: private job owns this compiler's entire process tree.
            unsafe {
                TerminateJobObject(self.0.0, 1);
            }
        }
    }
}
