use std::{
    io::BufRead,
    process::{Command, Stdio},
};

pub fn get_include_dirs() -> anyhow::Result<Vec<String>> {
    let child = Command::new("cmake")
        .args(["--build", "build", "-t", "_ast_includes"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    let output = child.wait_with_output()?;
    if output.status.success() {
        Ok(output
            .stdout
            .lines()
            .map_while(Result::ok)
            .find(|l| l.starts_with("@@INCLUDE_DIRS="))
            .ok_or_else(|| anyhow::anyhow!("No include dirs found"))?
            .trim_start_matches("@@INCLUDE_DIRS=")
            .split(';')
            .map(|i| i.trim().to_owned())
            .collect())
    } else {
        Err(anyhow::anyhow!(
            "cmake: bad status code - {:?}",
            output.status.code()
        ))
    }
}
