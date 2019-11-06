use std::io;

/// Get the current terminal attributes
fn get_termios() -> io::Result<libc::termios> {
    unsafe {
        let mut buf = std::mem::MaybeUninit::<libc::termios>::uninit();
        if libc::tcgetattr(libc::STDIN_FILENO, buf.as_mut_ptr()) == 0 {
            Ok(buf.assume_init())
        } else {
            Err(io::Error::from_raw_os_error(*libc::__errno_location()))
        }
    }
}

///
fn set_termios(termios: libc::termios) -> io::Result<()> {
    unsafe {
        if libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &termios) == 0 {
            Ok(())
        } else {
            Err(io::Error::from_raw_os_error(*libc::__errno_location()))
        }
    }
}

/// Restore the terminal attributes on Drop
#[must_use = "a termios guard has no effect if not stored"]
pub struct TermiosGuard(libc::termios);

impl Drop for TermiosGuard {
    fn drop(&mut self) {
        set_termios(self.0).unwrap();
    }
}

/// Set "raw mode" in the terminal
pub fn raw_mode() -> io::Result<TermiosGuard> {
    let mut termios = get_termios()?;
    let guard = TermiosGuard(termios);
    termios.c_iflag &= !(libc::BRKINT | libc::ICRNL | libc::INPCK | libc::ISTRIP | libc::IXON);
    termios.c_oflag &= !(libc::OPOST);
    termios.c_cflag |= libc::CS8;
    termios.c_lflag &= !(libc::ECHO | libc::ICANON | libc::IEXTEN | libc::ISIG);
    set_termios(termios)?;
    Ok(guard)
}
