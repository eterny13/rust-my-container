use std::env;
use std::ffi::CString;

//extern crate nix;
use nix::sched::{unshare, CloneFlags};
use nix::sys::wait::waitpid;
use nix::unistd::{execvp, fork, sethostname, chdir, chroot, getpid, getuid, ForkResult};
use nix::mount::{mount, umount, MsFlags};

const NONE: Option<&'static [u8]> = None;

fn main() {
  let args: Vec<String> = env::args().collect();
  match &*args[2] {
    "exec" => exec(&args[3..]),
    _ => panic!("Error : {}", &args[1])
  }
}

fn exec(cmd_array: &[String]) {

  let cmd = cmd_array.join(" ");

  println!("Main PID: {}, User: {}", getpid(), getuid());

  unshare(
    CloneFlags::CLONE_NEWNS
      | CloneFlags::CLONE_NEWPID
      | CloneFlags::CLONE_NEWUTS
      | CloneFlags::CLONE_NEWUSER,
  )
  .expect("failed unshare");

  println!("unshare PID: {}, User: {}", getpid(), getuid());

  match unsafe { fork().expect("failed fork") } {
    ForkResult::Parent { child } => {
      println!("Parent PID: {}, User: {} Child: {}", getpid(), getuid(), child);

      println!("Running [{}]", &cmd);
      waitpid(child, None).expect("failed waitpid");

      let bash = CString::new("bash").expect("CString::new failed");
      let argv = vec![
        CString::new("bash").unwrap(),
        CString::new("-c").unwrap(),
        CString::new("/proc/self/exe").unwrap(),
        CString::new(cmd).unwrap(),
      ];
      execvp(&bash, &argv).expect("failed execv");
    }
    ForkResult::Child => {
      println!("Child PID: {}, User: {}", getpid(), getuid());
      println!("Running [{}]", &cmd);

      sethostname("newhost").expect("failed sethostname");
      chroot("/").expect("failed chroot");
      chdir("/").expect("failed chdir");
      mount(
        Some("proc"),
        "proc",
        Some("proc"),
        MsFlags::empty(),
        NONE
      ).unwrap_or_else(|e| panic!("mount failed: {e}"));

      let bash = CString::new("bash").expect("CString::new failed");
      let argv = vec![
        CString::new("bash").unwrap(),
        CString::new("-c").unwrap(),
        CString::new(cmd).unwrap(),
      ];
      execvp(&bash, &argv).expect("failed execv");

      umount("proc").unwrap_or_else(|e| panic!("umount failed: {e}"));
    }
  }
}
