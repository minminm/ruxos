use alloc::boxed::Box;
use alloc::string::String;
use axerrno::{AxError, AxResult};
use ruxfs::api::ReadDir;
use ruxfs::fops::File;
use ruxfs::fops::{FileAttr, FileType, OpenOptions};
use axio::SeekFrom;

#[allow(dead_code)]
pub struct StdDirEntry {
    path: String,
    fname: String,
    ftype: FileType,
}

impl StdDirEntry {
    fn new(path: String, fname: String, ftype: FileType) -> Self {
        Self { path, fname, ftype }
    }
}

#[no_mangle]
pub fn sys_read_dir(path: &str) -> Result<usize, AxError> {
    let rd = ruxfs::api::read_dir(path)?;
    let ptr = Box::leak(Box::new(rd));
    Ok(ptr as *mut ReadDir as usize)
}

#[no_mangle]
pub unsafe fn sys_read_dir_next(handle: usize) -> Option<Result<StdDirEntry, AxError>> {
    let ptr = handle as *mut ReadDir;
    if let Some(Ok(ref de)) = ptr.as_mut().unwrap().next() {
        return Some(Ok(StdDirEntry::new(
            de.path(),
            de.file_name(),
            de.file_type(),
        )));
    }
    None
}

#[no_mangle]
pub fn sys_stat(path: &str) -> Result<FileAttr, AxError> {
    Ok(*ruxfs::api::metadata(path)?.raw_metadata())
}

#[no_mangle]
pub fn sys_open(path: &str, flags: u32) -> Result<usize, AxError> {
    const F_READ: u32 = 0x01;
    const F_WRITE: u32 = 0x02;
    const F_APPEND: u32 = 0x04;
    const F_TRUNC: u32 = 0x08;
    const F_CREATE: u32 = 0x10;
    const F_NEW: u32 = 0x20; /* for create_new */

    axlog::info!("sys_open... {} {:X}", path, flags);
    let mut opts = OpenOptions::new();
    opts.read(flags & F_READ != 0);
    opts.write(flags & F_WRITE != 0);
    opts.append(flags & F_APPEND != 0);
    opts.truncate(flags & F_TRUNC != 0);
    opts.create(flags & F_CREATE != 0);
    opts.create_new(flags & F_NEW != 0);

    axlog::info!("sys_open opts {:?}", opts);
    let f = File::open(path, &opts)?;
    let ptr = Box::leak(Box::new(f));
    Ok(ptr as *mut File as usize)
}

#[no_mangle]
pub fn sys_write(handle: usize, buf: &[u8]) -> Result<usize, AxError> {
    let f = handle as *mut File;
    unsafe { f.as_mut().unwrap().write(buf) }
}

#[no_mangle]
pub fn sys_read(handle: usize, buf: &mut [u8]) -> Result<usize, AxError> {
    let f = handle as *mut File;
    unsafe { f.as_mut().unwrap().read(buf) }
}

#[no_mangle]
pub fn sys_write_at(handle: usize, buf: &[u8], offset: u64) -> usize {
    let f = handle as *mut File;
    unsafe { f.as_mut().unwrap().write_at(offset, buf).unwrap() }
}

#[no_mangle]
pub fn sys_read_at(handle: usize, buf: &mut [u8], offset: u64) -> usize {
    let f = handle as *mut File;
    unsafe { f.as_mut().unwrap().read_at(offset, buf).unwrap() }
}


#[no_mangle]
pub fn sys_seek(handle: usize, pos: SeekFrom) -> Result<u64, AxError> {
    let f = handle as *mut File;
    unsafe { f.as_mut().unwrap().seek(pos) }
}


#[no_mangle]
pub fn sys_mkdir(path: &str) -> Result<(), AxError> {
    ruxfs::api::create_dir(path)
}

#[no_mangle]
pub fn sys_rmdir(path: &str) -> Result<(), AxError> {
    ruxfs::api::remove_dir(path)
}

#[no_mangle]
pub fn sys_unlink(path: &str) -> Result<(), AxError> {
    ruxfs::api::remove_file(path)
}

#[no_mangle]
pub fn sys_getcwd() -> Result<String, AxError> {
    ruxfs::api::current_dir()
}

#[no_mangle]
pub fn sys_chdir(path: &str) -> Result<(), AxError> {
    ruxfs::api::set_current_dir(path)
}

#[no_mangle]
pub fn sys_close_file(handle: usize) {
    unsafe { core::ptr::drop_in_place(handle as *mut File) }
}

#[no_mangle]
pub fn sys_close_dir(handle: usize) {
    unsafe { core::ptr::drop_in_place(handle as *mut ReadDir) }
}

#[no_mangle]
pub fn sys_sync(handle: usize) -> Result<(), AxError> {
    let f = handle as *mut File;
    unsafe { f.as_mut().unwrap().flush() }
}

#[no_mangle]
pub fn sys_truncate(handle: usize, size: u64) -> Result<(), AxError> {
    let f = handle as *mut File;
    unsafe { f.as_mut().unwrap().truncate(size) }
}

#[no_mangle]
pub fn sys_canonicalize(path: &str) -> Result<String, AxError> {
    ruxfs::api::canonicalize(path)
}

/// This only works then the new path is in the same mounted fs.
#[no_mangle]
pub fn sys_rename(old: &str, new: &str) -> Result<(), AxError> {
    ruxfs::api::rename(old, new)
}

pub fn current_dir() -> AxResult<String> {
    ruxfs::api::current_dir()
}

pub fn set_current_dir(path: &str) -> AxResult {
    ruxfs::api::set_current_dir(path)
}