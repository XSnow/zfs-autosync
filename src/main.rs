use std::env;
use std::process::{Command, Stdio};

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

fn call_cmd_piped(cmd1_exec: &str, cmd1_line: &[&str], cmd2_exec: &str, cmd2_line: &[&str])
                  -> Result<String, String> {

    println!("Calling {} with {:?} piped to {} with {:?}", cmd1_exec, cmd1_line, cmd2_exec, cmd2_line);

    let cmd1 = Command::new(cmd1_exec)
        .args(cmd1_line)
        .stdout(Stdio::piped())
        .spawn()
        .expect(&format!("Execute {} command with {:?} as 1st part of piped failed", cmd1_exec, cmd1_line));

    let out1 = cmd1.stdout.unwrap();

    let cmd2 = Command::new(cmd2_exec)
        .args(cmd2_line)
        .stdin(out1)
        .output()
        .expect(&format!("Execute {} command with {:?} as 2st part of piped failed", cmd2_exec, cmd2_line));

    //cmd1.wait().unwrap();

    let ret = if cmd2.status.success() {
        Ok(String::from_utf8(cmd2.stdout).unwrap())
    }else{
        Err(String::from_utf8(cmd2.stderr).unwrap())
    };

    return ret;

}

trait ZfsThing {
    const TYPE_STR: &'static str;
    fn fromfs(fsname: &str, name: &str) -> Self;
    fn name(&self) -> &str;
    fn fsname(&self) -> &str;
    fn list<T: ZfsThing>(&self, childname: Option<&str>)
                         -> Result<Vec<T>, String> {

        let appendname = childname.map(|s| format!("{}#{}",self.name(), s));
        let unpackname = appendname.as_ref().map_or(self.name(), |s| s.as_ref());

        let v = call_cmd("zfs", &["list", "-t", T::TYPE_STR, "-H", "-o", "name",
                                  "-p", "-S", "creation", unpackname])?;

        Ok(v.lines().map(|v| T::fromfs(self.fsname(), v)).collect())
    }

    fn destroy(&self) -> Result<(), String> {
        call_cmd("zfs", &["destroy", self.name()])?;
        Ok(())
    }
}

trait ZfsBookmarkable : ZfsThing {
    fn bookmark(&self, bookmark: &str) -> Result<Bookmark, String> {
        let full_bookmark = &format!("{}#{}", self.fsname(), bookmark);
        call_cmd("zfs", &["bookmark", self.name(), full_bookmark])?;
        Ok( Bookmark {
            fsname: self.fsname().to_string(),
            name: full_bookmark.to_string()
        } )
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
impl ZfsBookmarkable for Bookmark { }
impl ZfsThing for Snapshot {
    const TYPE_STR: &'static str = "snapshot";
    fn fromfs(fsname: &str, name: &str) -> Self {
        Snapshot { fsname: fsname.to_string(), name: name.to_string() }
    }
    fn name(&self) -> &str { &self.name }
    fn fsname(&self) -> &str { &self.fsname }
}
impl ZfsBookmarkable for Snapshot { }
impl Snapshot {
    fn send(&self, bookmark : Option<&Bookmark>, dest : &Filesystem) -> Result<(), String> {
        let args = bookmark.map_or(
            vec!["send", self.name()],
            |b| vec!["send", "-i", b.name(), self.name()]
        );
        call_cmd_piped(
            "zfs", &args,
            "zfs", &["receive", dest.name()])?;
        Ok(())
    }
}


fn main() {

    // read path from cmd arguments
    let args: Vec<String> = env::args().collect();

    let source = Filesystem::from_str(&args[1]);
    let dest = Filesystem::from_str(&args[2]);

    println!("Backup from {} to {}", source.name, dest.name());

    match &source.list::<Filesystem>(None).unwrap()[..] {
        [_] =>(), _ => panic!("WTF! IMPOSSIBLE!"),
    }

    let snapshots: Vec<Snapshot> = source.list(None).unwrap();
    let src_s = &snapshots[0];
    println!("Latest Snapshot is {} of {}",
             src_s.name, src_s.fsname);

    const NEW_BM: &str = "latest_new";
    match source.list::<Bookmark>(Some(NEW_BM)) {
        Ok(vec) => if let [s] = &vec[..] {
            s.destroy().expect("Destroy old bookmark fails");
        } else {
            panic!("Zero or more than one bookmark with name {}", NEW_BM);
        }
        _ => println!("No {} bookmark detected", NEW_BM),
    };

    let _new_bookmark = src_s.bookmark(NEW_BM).expect("Create new bookmark fails");

    const OLD_BM: &str = "latest";

    match source.list::<Bookmark>(Some(OLD_BM)) {
        Ok(bookmarks) => if let [src_i] = &bookmarks[..] {
            src_s.send(Some(&src_i), &dest).expect("Failed to send");
            println!("Successfully sent");
            src_i.destroy().expect("Destroy old bookmark fails");
        }else {
            panic!("Zero or more than one bookmark with name {}", OLD_BM);
        }
        _ => {
            src_s.send(None, &dest).expect("Failed to send");
            println!("Successfully sent");
        }
    };

    src_s.bookmark(OLD_BM).expect("Failed to create bookmark");

    // println!("grep: {}", std::str::from_utf8(&cmd2.stdout).unwrap());
}
