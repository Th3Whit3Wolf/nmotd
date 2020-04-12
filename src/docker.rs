use std::process::Command;

pub fn get_docker_processes() -> Vec<String> {
    let mut docker: Vec<String> = Vec::with_capacity(4);
    let output = Command::new("docker").arg("ps").arg("-a").output().unwrap();
    let output = String::from_utf8(output.stdout).unwrap();
    for line in output.lines().skip(1) {
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
    docker
}
