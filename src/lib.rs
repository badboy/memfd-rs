extern crate nix;

use nix::sys::memfd::*;
use std::ffi::CString;
use std::fs::File;
use std::io::{self};
use std::os::unix::io::{RawFd, AsRawFd, IntoRawFd, FromRawFd};
use std::fmt;

pub struct MemFd {
    file: File,
}

impl MemFd {
    pub fn create<S: Into<Vec<u8>>>(name: S) -> io::Result<MemFd> {
        let name = CString::new(name).unwrap();
        let flags = MemFdCreateFlag::empty();
        let rawfd = memfd_create(&name, flags)?;

        unsafe {
            Ok(MemFd {
                file: File::from_raw_fd(rawfd)
            })
        }
    }

    pub fn set_len(&self, size: u64) -> io::Result<()> {
        self.file.set_len(size)
    }
}

impl io::Write for MemFd {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl io::Read for MemFd {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file.read(buf)
    }
}

impl io::Seek for MemFd {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.file.seek(pos)
    }
}

impl fmt::Debug for MemFd {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.file.fmt(f)
    }
}

impl AsRawFd for MemFd {
    fn as_raw_fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }
}

impl IntoRawFd for MemFd {
    fn into_raw_fd(self) -> RawFd {
        self.file.into_raw_fd()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write, Seek, SeekFrom};

    #[test]
    fn it_works() {
        let _fd = MemFd::create("foobar").unwrap();
    }

    #[test]
    fn can_write() {
        let mut fd = MemFd::create("foobar").unwrap();

        let buf = b"hello world";
        assert_eq!(buf.len(), fd.write(&buf[..]).unwrap());
    }

    #[test]
    fn can_read_after_write_and_seek() {
        let mut fd = MemFd::create("foobar").unwrap();

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
        let mut fd1 = MemFd::create("foo1").unwrap();
        let mut fd2 = MemFd::create("foo2").unwrap();

        let buf = b"hello world";
        assert_eq!(buf.len(), fd1.write(&buf[..]).unwrap());

        let mut s = Vec::new();
        assert_eq!(0, fd2.read_to_end(&mut s).unwrap());
    }

    #[test]
    fn same_name() {
        let mut fd1 = MemFd::create("foobar").unwrap();
        let mut fd2 = MemFd::create("foobar").unwrap();

        let buf = b"hello world";
        assert_eq!(buf.len(), fd1.write(&buf[..]).unwrap());

        assert_eq!(0, fd1.seek(SeekFrom::Start(0)).unwrap());
        assert_eq!(0, fd2.seek(SeekFrom::Start(0)).unwrap());

        let mut s = Vec::new();
        assert_eq!(0, fd2.read_to_end(&mut s).unwrap());
    }

    #[test]
    fn set_size() {
        let mut fd = MemFd::create("foobar").unwrap();

        assert_eq!(0, fd.seek(SeekFrom::End(0)).unwrap());

        fd.set_len(42).unwrap();

        assert_eq!(42, fd.seek(SeekFrom::End(0)).unwrap());
    }
}
