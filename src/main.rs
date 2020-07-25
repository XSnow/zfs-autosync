use std::env;
use std::process::Command;

trait ZfsThing {
    fn typestr(&self) -> &str;

    fn list(&self, filesystem: &str)  -> Result<Vec<String>, String> {
        let v = call_cmd("zfs", &["list", "-t", self.typestr(), "-H", "-o", "name",
                                  "-p", "-S", "creation", filesystem]) ? ;

        Ok(v.lines().map(str::to_string).collect())
    }
}

struct Filesystem { name: &'static str }
struct Bookmark   { name: &'static str }
struct Snapshot   { name: &'static str }


impl ZfsThing for Filesystem {
    fn typestr(&self) -> &str {
        "filesystem";
    }
}
impl ZfsThing for Bookmark {
    fn typestr(&self) -> &str {
        "bookmark";
    }
}
impl ZfsThing for Snapshot {
    fn typestr(&self) -> &str {
        "snapshot";
    }
}

enum ZfsType {
    Filesystem,
    Bookmark,
    Snapshot
}

fn call_cmd(cmd_exec: &str, cmd_line: &[&str]) -> Result<String, String> {
    println!("Calling {} with {:?}", cmd_exec, cmd_line);

    let cmd_result = Command::new(cmd_exec)
        .args(cmd_line)
        .output()
        .expect(&format!("Execute {} command failed", cmd_exec));

    let ret = if cmd_result.status.success() {
        Ok(String::from_utf8(cmd_result.stdout).unwrap())
    }else{
        Err(String::from_utf8(cmd_result.stderr).unwrap())
    };

    return ret;
}

fn destroy_bookmark(filesystem: &str, bookmark: &str)
                    -> Result<String, String> {

    let full_bookmark = &format!("{}#{}", filesystem, bookmark);

    call_cmd("zfs", &["destroy", full_bookmark])
}

fn create_bookmark(filesystem: &str, snapshot: &str, bookmark: &str)
                   -> Result<String, String> {

    let full_bookmark = &format!("{}#{}", filesystem, bookmark);

    call_cmd("zfs", &["bookmark", snapshot, full_bookmark])
}

fn zfs_list<T> (filesystem: &str) -> Result<Vec<String>, String> {

    let typestr = match T {
        _ => "test",
        // ZfsType::Filesystem => "filesystem",
        // ZfsType::Bookmark => "bookmark",
        // ZfsType::Snapshot => "snapshot",
    };
    let v = call_cmd("zfs", &["list", "-t", typestr, "-H", "-o", "name",
                              "-p", "-S", "creation", filesystem]) ? ;

    Ok(v.lines().map(str::to_string).collect())
}

fn main() {

    // read path from cmd arguments
    let args: Vec<String> = env::args().collect();

    let source = &args[1];
    let dest = &args[2];

    println!("Backup from {} to {}", source, dest);

    // find datasets
    // for each datasets, decide whether should initialize a copy

    // zfs list -t filesystem -H -o name

    // return 1
    if let [ret] = &zfs_list(ZfsType::Filesystem, source).unwrap()[..] {
        assert_eq!(ret, source);
    }else {
        panic!("WTF!");
    }

    let snapshots = zfs_list(ZfsType::Snapshot, source).unwrap();
    println!("Latest Snapshot is {}", snapshots[0]);

    //destroy_bookmark(source, "latest_new");
    //create_bookmark(source, latest_snapshot, "latest_new");

}
