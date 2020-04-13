use std::process::Command;

pub fn get_docker_processes() -> Option<Vec<String>> {
    let mut docker: Vec<String> = Vec::with_capacity(4);
    match Command::new("docker").arg("ps").arg("-a").output() {
        Ok(x) => {
            for line in String::from_utf8(x.stdout).unwrap().lines().skip(1) {
                if line
                    .split_whitespace()
                    .skip(1)
                    .next()
                    .unwrap()
                    .to_string()
                    .contains('/')
                {
                    docker.push(str::replace(
                        line.split_whitespace()
                            .skip(1)
                            .next()
                            .unwrap()
                            .split('/')
                            .skip(1)
                            .next()
                            .unwrap(),
                        ':',
                        " ",
                    ))
                } else {
                    docker.push(str::replace(
                        line.split_whitespace().skip(1).next().unwrap(),
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
