build:
    #!/bin/bash
    RUSTFLAGS='-C target-cpu=native -C link-arg=-s' cargo run --release   

coverage:
    #!/bin/bash
    cargo tarpaulin -v  

size:
    #!/bin/bash
    /bin/du -h target/release/nmotd

time:
    #!/usr/bin/ion
    hyperfine target/release/nmotd