extern crate getopts;

use getopts::{getopts,OptGroup};
use std::os;
use std::io;

struct FileInfo {
    full_path: Path,
    rel_path: Path,
    stat: io::FileStat,
}

impl Clone for FileInfo {
    fn clone(&self) -> FileInfo {
        FileInfo {
            full_path: self.full_path.clone(),
            rel_path: self.rel_path.clone(),
            stat: self.stat,
        }
    }
}

fn print_usage(program: String, _opts: &[OptGroup]) {
    println!("Usage: {:s} SRCS [DEST]", program);
}

fn perm2str (perm: io::FilePermission) -> String {
    let mut ret = String::with_capacity(9);
    let get_char = |bit : uint, ch| if perm.bits() & (1 << bit) == (1 << bit) { ch } else { '-' };

    ret.push_char(get_char(8, 'r'));
    ret.push_char(get_char(7, 'w'));
    ret.push_char(get_char(6, 'x'));
    ret.push_char(get_char(5, 'r'));
    ret.push_char(get_char(4, 'w'));
    ret.push_char(get_char(3, 'x'));
    ret.push_char(get_char(2, 'r'));
    ret.push_char(get_char(1, 'w'));
    ret.push_char(get_char(0, 'x'));

    ret
}

fn path2str(info: &FileInfo) -> String {
    use std::io::TypeDirectory;

    let dir = if info.stat.kind == TypeDirectory { 'd' } else { '-' };
    let rights = format!("{}{}", dir, perm2str(info.stat.perm));

    format!("{rights:s} {size:>9u} {name:s}", rights=rights, size=info.stat.size, name=info.full_path.as_str().unwrap())
}

fn print_sources(sources: &[String]) {
    let paths = get_paths(sources);
    for info in paths.iter() {
        println!("{:s}", path2str(info));
    }
}

fn get_dir_entries(path: Path) -> io::IoResult<Vec<FileInfo>> {
    use std::collections::{Deque, RingBuf};
    use std::io::fs;

    let mut ret = Vec::new();
    let mut path_queue = RingBuf::new();
    path_queue.push_back(path.clone());

    loop {
        let dir = match path_queue.pop_front() {
            Some(d) => d,
            None => break,
        };

        let contents = try!(fs::readdir(&dir));

        for entry in contents.iter() {
            if entry.is_dir() {
                path_queue.push_back(entry.clone());
            }

            ret.push(FileInfo {
                full_path: entry.clone(),
                rel_path: entry.path_relative_from(&path).unwrap(),
                stat: try!(entry.lstat()),
            });
        }
    }

    Ok(ret)
}

fn get_paths(sources: &[String]) -> Vec<FileInfo> {
    let mut path_vecs = sources.iter()
        .map(|s| Path::new(s.clone()))
        .map(|ref path| {
            if path.is_dir() {
                get_dir_entries(path.clone()).unwrap()
            } else {
                vec!( FileInfo {
                    full_path: path.clone(),
                    rel_path: path.dir_path(),
                    stat: path.lstat().unwrap(),
                } )
            }
        });

    path_vecs
        .fold(vec!(), |n, o| n.append(o.as_slice()))
}

fn do_copy(sources: &[String], destination: String) {
    let source_paths = get_paths(sources);
    let destination = Path::new(destination);

    for info in source_paths.iter() {
        let mut new_dest = destination.clone();
        new_dest.push(&info.rel_path);
        println!("{}", new_dest.as_str().unwrap());
        //copy(f, destination.push(f));
    }
}

fn main() {
    let args: Vec<String> = os::args().iter()
                                      .map(|x| x.to_string())
                                      .collect();

    let program = args[0].clone();

    let opts = [
    ];
    let matches = match getopts(args.tail(), opts) {
        Ok(m) => { m }
        Err(f) => { fail!(f) }
    };
    let (sources, destination) = if !matches.free.is_empty() {
        if matches.free.len() == 1 {
            (matches.free.as_slice(), None)
        } else {
            (matches.free.init(), matches.free.last())
        }
    } else {
        print_usage(program, opts);
        return;
    };

    match destination {
        None => print_sources(sources),
        Some(dest) => do_copy(sources, dest.clone()),
    }
}
