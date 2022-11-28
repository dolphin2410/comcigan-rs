fn main() {
    std::process::Command::new("git")
        .args([
            "submodule",
            "update",
            "--init",
            "--depth 1",
            "--recommend-shallow",
        ])
        .output()
        .expect("Failed to fetch git submodules!");
}