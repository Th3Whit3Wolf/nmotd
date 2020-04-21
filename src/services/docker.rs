use std::process::Command;

pub fn get_docker_processes() -> Option<Vec<(String, String)>> {
    let mut docker: Vec<(String, String)> = Vec::with_capacity(4);
    match Command::new("docker")
        .arg("ps")
        .arg("--format")
        .arg("{{.Image}}:#: {{.Status}}")
        .output()
    {
        Ok(x) => {
            for line in String::from_utf8(x.stdout).unwrap().lines().skip(1) {
                docker.push((
                    line.split(":#:").next().unwrap().to_string(),
                    line.split(":#:").nth(1).unwrap().to_string(),
                ))
            }
            Some(docker)
        }
        Err(_) => None,
    }
}
