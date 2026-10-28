//! A reader whose console is gone ends, on Windows (TASK-69e774ef9d64).
//!
//! The Unix half is `terminal::a_reader_whose_terminal_hangs_up_ends_by_itself`:
//! a reader orphaned by a suite whose pseudo-terminal closed read end of file
//! in a loop, about 1.3 million times a second, for days. CLAUDE.md does not
//! let OS-dependent behaviour count as verified until it has run on all three
//! platforms, and Windows reaches the console by another road entirely --
//! crossterm reads console records there, not bytes -- so what happens when
//! the console goes is a fact to measure on Windows and not one to infer from
//! the other two.
//!
//! **ConPTY, opened here with kernel32's own calls.** The `terminal` harness
//! is Unix only because reaching ConPTY means the console API this workspace
//! does not otherwise call. That rule is about the reader's sources, which
//! `dependencies.rs` holds to no foreign symbol; a test declares what it needs
//! to drive the binary, as `terminal::pty` declares `posix_openpt`. This file
//! is the least of it: a pseudo-console, a process attached to it, and the
//! console closed under that process once it has gone quiet.
//!
//! **The bound is a deadline and not a measurement**, for the reason the Unix
//! test gives: what is asserted is that the reader ends at all, and a reader
//! that never does is terminated before the failure is reported, so it does
//! not outlive the run.

use std::ffi::c_void;
use std::io::{Read, Write};
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::{FromRawHandle, RawHandle};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

type Handle = *mut c_void;
type Bool = i32;

#[repr(C)]
#[derive(Clone, Copy)]
struct Coord {
    x: i16,
    y: i16,
}

#[repr(C)]
struct StartupInfoW {
    cb: u32,
    reserved: *mut u16,
    desktop: *mut u16,
    title: *mut u16,
    x: u32,
    y: u32,
    x_size: u32,
    y_size: u32,
    x_count_chars: u32,
    y_count_chars: u32,
    fill_attribute: u32,
    flags: u32,
    show_window: u16,
    reserved2_size: u16,
    reserved2: *mut u8,
    std_input: Handle,
    std_output: Handle,
    std_error: Handle,
}

#[repr(C)]
struct StartupInfoExW {
    startup_info: StartupInfoW,
    attribute_list: *mut c_void,
}

#[repr(C)]
struct ProcessInformation {
    process: Handle,
    thread: Handle,
    process_id: u32,
    thread_id: u32,
}

#[link(name = "kernel32")]
extern "system" {
    fn CreatePipe(
        read: *mut Handle,
        write: *mut Handle,
        attributes: *const c_void,
        size: u32,
    ) -> Bool;
    fn CreatePseudoConsole(
        size: Coord,
        input: Handle,
        output: Handle,
        flags: u32,
        console: *mut Handle,
    ) -> i32;
    fn ClosePseudoConsole(console: Handle);
    fn InitializeProcThreadAttributeList(
        list: *mut c_void,
        count: u32,
        flags: u32,
        size: *mut usize,
    ) -> Bool;
    fn UpdateProcThreadAttribute(
        list: *mut c_void,
        flags: u32,
        attribute: usize,
        value: *const c_void,
        size: usize,
        previous: *mut c_void,
        returned: *mut usize,
    ) -> Bool;
    fn DeleteProcThreadAttributeList(list: *mut c_void);
    fn CreateProcessW(
        application: *const u16,
        command_line: *mut u16,
        process_attributes: *const c_void,
        thread_attributes: *const c_void,
        inherit: Bool,
        flags: u32,
        environment: *const c_void,
        directory: *const u16,
        startup: *const StartupInfoExW,
        information: *mut ProcessInformation,
    ) -> Bool;
    fn WaitForSingleObject(handle: Handle, milliseconds: u32) -> u32;
    fn TerminateProcess(process: Handle, code: u32) -> Bool;
    fn CloseHandle(handle: Handle) -> Bool;
}

const PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE: usize = 0x0002_0016;
const EXTENDED_STARTUPINFO_PRESENT: u32 = 0x0008_0000;
const STARTF_USESTDHANDLES: u32 = 0x0000_0100;
const WAIT_OBJECT_0: u32 = 0;
const WAIT_TIMEOUT: u32 = 0x102;

/// The `ank` this workspace just built, beside the test executable's
/// directory, as `terminal::ank` finds it on Unix.
fn ank() -> PathBuf {
    let mut at = std::env::current_exe().expect("a test executable has a path");
    at.pop();
    if at.file_name().is_some_and(|n| n == "deps") {
        at.pop();
    }
    let binary = at.join("ank.exe");
    assert!(
        binary.is_file(),
        "the ank binary is not at {}: build it first (cargo build -p ank-cli)",
        binary.display()
    );
    binary
}

/// A corpus of two entities, the shape of `terminal::Repo::seeded`, removed
/// when the test ends.
struct Repo(PathBuf);

impl Repo {
    fn seeded() -> Repo {
        let root = std::env::temp_dir().join(format!("ank-conpty-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src")).expect("the temporary directory is writable");
        std::fs::write(root.join("src/lib.rs"), "// code\n").unwrap();
        let repo = Repo(root);
        repo.run("git", &["init", "--initial-branch=main"]);
        repo.run("git", &["config", "user.email", "suite@example.invalid"]);
        repo.run("git", &["config", "user.name", "The Suite"]);
        repo.run("git", &["config", "commit.gpgsign", "false"]);
        repo.run("git", &["config", "gc.auto", "0"]);
        let ank = ank();
        let ank = ank.to_str().expect("the binary's path is text");
        repo.run(ank, &["init"]);
        repo.run(
            ank,
            &[
                "new",
                "adr",
                "--title",
                "Every byte shown is a byte the CLI printed",
                "--scope",
                "src/**",
                "--constraint",
                "The reader reaches the corpus by running the CLI.",
            ],
        );
        repo.run(
            ank,
            &[
                "new",
                "task",
                "--title",
                "A task the reader lists",
                "--scope",
                "src/**",
                "--criteria",
                "The frame names this entity.",
            ],
        );
        repo
    }

    fn run(&self, program: &str, args: &[&str]) {
        let out = Command::new(program)
            .args(args)
            .current_dir(&self.0)
            .output()
            .expect("the fixture's command runs");
        assert!(
            out.status.success(),
            "{program} {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn wide(text: &str) -> Vec<u16> {
    std::ffi::OsStr::new(text)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn wide_path(path: &Path) -> Vec<u16> {
    path.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

#[test]
fn a_reader_whose_console_closes_ends_by_itself() {
    let repo = Repo::seeded();

    // SAFETY: the calls below are the documented ConPTY sequence, in order,
    // and every return is checked before the next call is made. Each pipe end
    // is owned by a `File` from the moment it is not handed to the
    // pseudo-console, the attribute list lives in a buffer of this function's
    // for as long as `CreateProcessW` reads it, and the process and thread
    // handles are closed on every road out.
    unsafe {
        let (mut console_in, mut ours_in) = (std::ptr::null_mut(), std::ptr::null_mut());
        let (mut ours_out, mut console_out) = (std::ptr::null_mut(), std::ptr::null_mut());
        assert!(CreatePipe(&mut console_in, &mut ours_in, std::ptr::null(), 0) != 0);
        assert!(CreatePipe(&mut ours_out, &mut console_out, std::ptr::null(), 0) != 0);
        let mut console: Handle = std::ptr::null_mut();
        let made = CreatePseudoConsole(
            Coord { x: 120, y: 30 },
            console_in,
            console_out,
            0,
            &mut console,
        );
        assert!(made >= 0, "CreatePseudoConsole answered {made:#x}");
        // The pseudo-console holds its own copies now.
        CloseHandle(console_in);
        CloseHandle(console_out);
        let input = std::fs::File::from_raw_handle(ours_in as RawHandle);
        let mut output = std::fs::File::from_raw_handle(ours_out as RawHandle);

        let mut size = 0usize;
        InitializeProcThreadAttributeList(std::ptr::null_mut(), 1, 0, &mut size);
        let mut list = vec![0u8; size];
        let list = list.as_mut_ptr() as *mut c_void;
        assert!(InitializeProcThreadAttributeList(list, 1, 0, &mut size) != 0);
        assert!(
            UpdateProcThreadAttribute(
                list,
                0,
                PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE,
                console as *const c_void,
                std::mem::size_of::<Handle>(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            ) != 0
        );

        let mut startup: StartupInfoExW = std::mem::zeroed();
        startup.startup_info.cb = std::mem::size_of::<StartupInfoExW>() as u32;
        // Standard handles that are not the test's own: with
        // `STARTF_USESTDHANDLES` and none given, the child reaches the console
        // it is attached to and nothing this process holds.
        startup.startup_info.flags = STARTF_USESTDHANDLES;
        startup.attribute_list = list;
        let mut command_line = wide(&format!("\"{}\" tui", ank().display()));
        let directory = wide_path(&repo.0);
        let mut information: ProcessInformation = std::mem::zeroed();
        let spawned = CreateProcessW(
            std::ptr::null(),
            command_line.as_mut_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            0,
            EXTENDED_STARTUPINFO_PRESENT,
            std::ptr::null(),
            directory.as_ptr(),
            &startup,
            &mut information,
        );
        DeleteProcThreadAttributeList(list);
        assert!(
            spawned != 0,
            "CreateProcessW failed: {}",
            std::io::Error::last_os_error()
        );
        CloseHandle(information.thread);
        let process = information.process;
        let fail = |why: String| -> ! {
            TerminateProcess(process, 1);
            WaitForSingleObject(process, 10_000);
            CloseHandle(process);
            panic!("{why}");
        };

        // Drained on a thread of its own, for the reason `terminal::Live`
        // drains: a pseudo-console nobody reads blocks the process writing to
        // it, and closing one whose output is unread can block the closer.
        let (chunks, arriving) = std::sync::mpsc::channel::<usize>();
        let drain = std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match output.read(&mut buf) {
                    Ok(0) | Err(_) => return,
                    Ok(n) => {
                        if chunks.send(n).is_err() {
                            return;
                        }
                    }
                }
            }
        });

        // Painted, and then quiet for a second: the reader the Unix test
        // closes the terminal under is one waiting on a keystroke.
        let deadline = Instant::now() + Duration::from_secs(60);
        let mut painted = 0usize;
        loop {
            if Instant::now() >= deadline {
                fail(format!(
                    "the reader never went quiet: {painted} bytes painted in 60 s"
                ));
            }
            match arriving.recv_timeout(Duration::from_secs(1)) {
                Ok(n) => painted += n,
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) if painted > 0 => break,
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    fail("the console's output closed before the reader went quiet".into())
                }
            }
        }
        if WaitForSingleObject(process, 0) != WAIT_TIMEOUT {
            CloseHandle(process);
            panic!(
                "the reader ended before its console was closed, so nothing was \
                 measured: {painted} bytes painted"
            );
        }

        // What a test process dying under a session leaves: every handle it
        // held on the pseudo-console, closed.
        let mut input = input;
        let _ = input.flush();
        drop(input);
        ClosePseudoConsole(console);
        let ended = WaitForSingleObject(process, 30_000);
        let _ = drain.join();
        if ended != WAIT_OBJECT_0 {
            fail("the reader was still running 30 s after its console closed".into());
        }
        CloseHandle(process);
    }
}
