use std::process::Command;

pub fn run() {
    let output = Command::new("git")
        .args([
            "log",
            "--oneline",
            "--graph",
            "--decorate",
            "--all",
            "--color=always",
            "--pretty=format:%C(auto)%h%d %s %C(dim)(%an, %ar)%C(reset)",
        ])
        .output()
        .expect("Failed to run git log");

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout);
        println!("{result}");
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        eprintln!("❌ git log failed:\n{err}");
    }
}
