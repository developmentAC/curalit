cargo clean
cargo build --release
./target/release/curalit search -k "study" -k "alpha" -k "protein" -d ./data --logic or -o results
./target/release/curalit stats -c 0_out/results_*.csv
python3 0_out/results_*_visualize.py


