use super::changes::Changes;
pub fn print(changes: &Changes) {
    for entry in &changes.added {
        println!("+ {:?}", entry.path);
    }
    for entry in &changes.removed {
        println!("- {:?}", entry.path);
    }
    for entry in &changes.modified {
        println!("~ {:?}", entry.path);
    }
}
