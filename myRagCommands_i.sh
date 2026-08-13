#cargo clean
#cargo build --release
./target/release/curalit search -k "study" -k "protein" -d ./data --logic or -o results
./target/release/curalit stats -c 0_out/results_*.csv
uv run 0_out/results_*_visualize.py


echo Build RAG index:
./target/release/curalit rag-build -c 0_out/results_23Jun2026_214734.csv

echo Build verification database:
./target/release/curalit db-build -k "study" -k "protein"  -d ./data

echo Query with verified citations:
./target/release/curalit rag-generate -q "Please provide references of three articles where the keywords were relevant." -m llama3.2 --use-db 0_out/database.db
