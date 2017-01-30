extern crate nix;

use nix::sys::memfd::*;
use std::ffi::CString;
use std::fs::File;
use std::io::{self};
use std::os::unix::io::FromRawFd;

pub struct OpenOptions(MemFdCreateFlag);

impl OpenOptions {
    pub fn new() -> OpenOptions {
        OpenOptions(MemFdCreateFlag::empty())
    }

    pub fn allow_sealing(&mut self, allow_sealing: bool) -> &mut OpenOptions {
        if allow_sealing {
            self.0.insert(MFD_ALLOW_SEALING)
        } else {
            self.0.remove(MFD_ALLOW_SEALING)
        }
        self
    }

    pub fn close_on_exec(&mut self, cloexec: bool) -> &mut OpenOptions {
        if cloexec {
            self.0.insert(MFD_CLOEXEC)
        } else {
            self.0.remove(MFD_CLOEXEC)
        }
        self
    }

    /// Creates a memfd file at `name` with the options specified by `self`.
    pub fn create<S: Into<Vec<u8>>>(&self, name: S) -> io::Result<File> {
        let name = CString::new(name).unwrap();
        let rawfd = memfd_create(&name, self.0)?;

        unsafe {
            Ok(File::from_raw_fd(rawfd))
        }
    }
}

/// Creates a memfd file at `name`
pub fn create<S: Into<Vec<u8>>>(name: S) -> io::Result<File> {
    OpenOptions::new().create(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write, Seek, SeekFrom};

    #[test]
    fn it_works() {
        let _fd = create("foobar").unwrap();
    }

    #[test]
    fn can_write() {
        let mut fd = create("foobar").unwrap();

        let buf = b"hello world";
        assert_eq!(buf.len(), fd.write(&buf[..]).unwrap());
    }

    #[test]
    fn can_read_after_write_and_seek() {
        let mut fd = create("foobar").unwrap();

        let buf = b"hello world";
        assert_eq!(buf.len(), fd.write(&buf[..]).unwrap());

        let mut s = Vec::new();
        assert_eq!(0, fd.read_to_end(&mut s).unwrap());

        assert_eq!(0, fd.seek(SeekFrom::Start(0)).unwrap());

        assert_eq!(buf.len(), fd.read_to_end(&mut s).unwrap());
        assert_eq!(buf, &s[..]);

    }

    #[test]
    fn name_difference() {
        let mut fd1 = create("foo1").unwrap();
        let mut fd2 = create("foo2").unwrap();

        let buf = b"hello world";
        assert_eq!(buf.len(), fd1.write(&buf[..]).unwrap());

        let mut s = Vec::new();
        assert_eq!(0, fd2.read_to_end(&mut s).unwrap());
    }

    #[test]
    fn same_name() {
        let mut fd1 = create("foobar").unwrap();
        let mut fd2 = create("foobar").unwrap();

        let buf = b"hello world";
        assert_eq!(buf.len(), fd1.write(&buf[..]).unwrap());

        assert_eq!(0, fd1.seek(SeekFrom::Start(0)).unwrap());
        assert_eq!(0, fd2.seek(SeekFrom::Start(0)).unwrap());

        let mut s = Vec::new();
        assert_eq!(0, fd2.read_to_end(&mut s).unwrap());
    }

    #[test]
    fn set_size() {
        let mut fd = create("foobar").unwrap();

        assert_eq!(0, fd.seek(SeekFrom::End(0)).unwrap());

        fd.set_len(42).unwrap();

        assert_eq!(42, fd.seek(SeekFrom::End(0)).unwrap());
    }

    #[test]
    fn set_openoptions() {
        let _fd = OpenOptions::new()
            .close_on_exec(true)
            .allow_sealing(true)
            .create("foobar").unwrap();
    }
}
