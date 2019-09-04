use std::env;
use std::io;
use std::fs;
use std::path::Path;

mod hashdb;

fn visit_dirs(dir: &Path, cb: &mut FnMut(&Path)) -> io::Result<()> {
    if dir.is_dir() {
        println!("processing {}...", dir.to_str().unwrap());
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            if let Ok(tp) = entry.file_type() {
                if tp.is_symlink() {
                    continue;
                }
            }
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry.path());
            }
        }
    }
    Ok(())
}

fn main() {
    let argv: Vec<String> = env::args().collect();
    let argc = argv.len();
    if argc != 1 {
        let mut db = hashdb::HashDB::new();
        let mut add_callback = |p: &Path| db.process_file(p);
        for i in 1..argc {
            let dir = Path::new(&argv[i]);
            visit_dirs(dir, &mut add_callback).expect("unknown error occured while scanning dirs");
        }
    } else {
        println!("usage: {} <path1> ... <pathN>", argv[0]);
    }
}
