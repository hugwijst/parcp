extern crate getopts;

use getopts::{getopts,OptGroup};
use std::os;
use std::io::FilePermission;

fn print_usage(program: String, _opts: &[OptGroup]) {
    println!("Usage: {:s} SRCS [DEST]", program);
}

/*
fn expand_path(path: Path) -> Vec<Path> {
    let stat = lstat(&path).unwrap();
    let targets = match stat.kind {
        TypeFile => vec!(path),
        TypeDirectory => {
        }
}
*/

fn perm2str (perm: FilePermission) -> String {
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

fn path2str(path: Path) -> String {
    use std::io::TypeDirectory;

    let stat = path.stat().unwrap();
    let dir = if stat.kind == TypeDirectory { 'd' } else { '-' };
    let rights = format!("{}{}", dir, perm2str(stat.perm));

    format!("{rights:s} {size:>9u} {name:s}", rights=rights, size=stat.size, name=path.as_str().unwrap())
}

fn print_sources(sources: &[String]) {
    use std::io::{TypeDirectory,TypeFile};
    use std::io::fs::{walk_dir,lstat};

    let paths = get_paths(sources);
    for &(ref p1, ref p2) in paths.iter() {
        println!("{:s}", path2str(p1.join(p2)));
    }
}

fn get_paths(sources: &[String]) -> Vec<(Path, Path)> {
    use std::io::fs::walk_dir;

    let mut path_vecs = sources.iter()
        .map(|s| Path::new(s.clone()))
        .map(|ref path| {
            if path.is_dir() {
                walk_dir(path).unwrap()
                    .map(|p| (path.clone(), p.path_relative_from(path).unwrap()) )
                    .inspect(|&(ref p1, ref p2)| println!("({}, {})", p1.as_str().unwrap(), p2.as_str().unwrap()))
                    .collect()
            }
            else { vec!( (Path::new("."),path.clone()) ) }
        });
    path_vecs
        .fold(vec!(), |n, o| n.append(o.as_slice()))
}

fn do_copy(sources: &[String], destination: String) {
    let source_paths = get_paths(sources);
    let mut destination = Path::new(destination);

    for &(ref base, ref fs) in source_paths.iter() {
destination.push(fs);
        println!("{}", destination.as_str().unwrap());
        //copy(f, destination.push(f));
    }
}

fn main() {
    let args: Vec<String> = os::args().iter()
                                      .map(|x| x.to_string())
                                      .collect();

    let program = args.get(0).clone();

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
