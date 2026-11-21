LOOPERT_IP=192.168.0.37

cargo build --release --target aarch64-unknown-linux-gnu && 
scp target/aarch64-unknown-linux-gnu/release/loopert loopert@${LOOPERT_IP}:~/loopert