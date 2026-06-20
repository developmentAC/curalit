cargo clean
cargo build --release
./target/release/curalit search -k "cancer" -k "immunotherapy" -d ./data -o results
curalit stats -c 0_out/results_*.csv
python 0_out/results_*_visualize.py


