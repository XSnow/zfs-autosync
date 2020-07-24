use std::env;
use std::process::Command;

fn main() {

    // read path from cmd arguments
    let args: Vec<String> = env::args().collect();

    let source = &args[1];
    let dest = &args[2];

    println!("Backup from {} to {}", source, dest);

    // find datasets
    // for each datasets, decide whether should initialize a copy

    // zfs list -t filesystem -H -o name

    // zfs list -t snapshot -H -o name -p -S creation snowdisk/pictures
    // return 1

    let latest_snapshot = "snowdisk/pictures@zfs-auto-snap_hourly-2020-07-24-1417";

    // update #latest_new
    let new_bookmark = &format!("{}#latest_new", source);

    let mut cmd_remove_bookmark = Command::new("zfs");
    cmd_remove_bookmark.arg("destroy");
    cmd_remove_bookmark.arg(new_bookmark);

    println!("{:?}", cmd_remove_bookmark);

    let cmd_remove_bookmark =
        cmd_remove_bookmark
        .output()
        .expect("zfs remove output fails");

    let mut cmd_create_bookmark = Command::new("zfs");
    cmd_create_bookmark.arg("bookmark");
    cmd_create_bookmark.arg(latest_snapshot);
    cmd_create_bookmark.arg(new_bookmark);

    println!("{:?}", cmd_create_bookmark);

    let cmd_create_bookmark =
        cmd_create_bookmark
        .output()
        .expect("zfs create output fails");

    if cmd_create_bookmark.status.success() {

        println!("bookmark created: {}", std::str::from_utf8(&cmd_create_bookmark.stdout).expect("converting fails"));

    }else{

        println!("{}", std::str::from_utf8(&cmd_create_bookmark.stderr).expect("converting fails"));

    }


}
