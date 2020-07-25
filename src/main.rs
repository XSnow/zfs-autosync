use std::env;
use std::process::Command;


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

trait ZfsThing {
    const TYPE_STR: &'static str;
    fn fromfs(fsname: &str, name: &str) -> Self;
    fn name(&self) -> &str;
    fn fsname(&self) -> &str;
    fn list<T: ZfsThing>(&self)  -> Result<Vec<T>, String> {
        let v = call_cmd("zfs", &["list", "-t", T::TYPE_STR, "-H", "-o", "name",
                                  "-p", "-S", "creation", self.name()])?;

        Ok(v.lines().map(|v| T::fromfs(self.fsname(), v)).collect())
    }
    fn destroy(&self) -> Result<(), String> {
        call_cmd("zfs", &["destroy", self.name()])?;
        Ok(())
    }
}

struct Filesystem { name: String }
struct Bookmark   { fsname: String, name: String }
struct Snapshot   { fsname: String, name: String }

impl Filesystem {
    fn from_str(name: &str) -> Self { Filesystem { name: name.to_string() } }
}
impl ZfsThing for Filesystem {
    const TYPE_STR: &'static str = "filesystem";
    fn fromfs(fsname: &str, name: &str) -> Self {
        assert_eq!(fsname, name);
        Filesystem::from_str(name)
    }
    fn name(&self) -> &str { &self.name }
    fn fsname(&self) -> &str { &self.name }
}
impl ZfsThing for Bookmark {
    const TYPE_STR: &'static str = "bookmark";
    fn fromfs(fsname: &str, name: &str) -> Self {
        Bookmark{ fsname: fsname.to_string(), name: name.to_string() }
    }
    fn name(&self) -> &str { &self.name }
    fn fsname(&self) -> &str { &self.fsname }
}
impl ZfsThing for Snapshot {
    const TYPE_STR: &'static str = "snapshot";
    fn fromfs(fsname: &str, name: &str) -> Self {
        Snapshot { fsname: fsname.to_string(), name: name.to_string() }
    }
    fn name(&self) -> &str { &self.name }
    fn fsname(&self) -> &str { &self.fsname }
}
impl Snapshot {
    fn bookmark(&self, bookmark: &str) -> Result<(), String> {
        let full_bookmark = &format!("{}#{}", self.fsname(), bookmark);
        call_cmd("zfs", &["bookmark", self.name(), full_bookmark])?;
        Ok(())
    }
}


fn main() {

    // read path from cmd arguments
    let args: Vec<String> = env::args().collect();

    let source = Filesystem::from_str(&args[1]);
    let dest = Filesystem::from_str(&args[2]);

    println!("Backup from {} to {}", source.name, dest.name());

    match &source.list::<Filesystem>().unwrap()[..] {
        [_] =>(), _ => panic!("WTF! IMPOSSIBLE!"),
    }

    let snapshots: Vec<Snapshot> = source.list().unwrap();
    println!("Latest Snapshot is {} of {}",
             snapshots[0].name, snapshots[0].fsname);

    let bookmarks: Vec<Bookmark> = source.list().unwrap();
    // println!("Bookmarks {?:}", )
    //destroy_bookmark(source, "latest_new");
    //create_bookmark(source, latest_snapshot, "latest_new");

}
