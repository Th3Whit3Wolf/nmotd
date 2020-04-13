use std::process::Command;

pub fn get_docker_processes() -> Option<Vec<String>> {
    let mut docker: Vec<String> = Vec::with_capacity(4);
    match Command::new("docker").arg("ps").arg("-a").output() {
        Ok(x) => {
            if let Some(line) = String::from_utf8(x.stdout).unwrap().lines().nth(1) {
                if line
                    .split_whitespace()
                    .nth(1)
                    .unwrap()
                    .to_string()
                    .contains('/')
                {
                    docker.push(str::replace(
                        line.split_whitespace()
                            .nth(1)
                            .unwrap()
                            .split('/')
                            .nth(1)
                            .unwrap(),
                        ':',
                        " ",
                    ))
                } else {
                    docker.push(str::replace(
                        line.split_whitespace().nth(1).unwrap(),
                        ':',
                        " ",
                    ))
                }
            }
            Some(docker)
        }
        Err(_) => None,
    }
}
