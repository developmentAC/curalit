# CuraLit Quick Start Guide

Get started with CuraLit in 5 minutes!

## Installation

```bash
# Clone and build
git clone https://github.com/yourusername/curalit.git
cd curalit
cargo build --release

# Install Python dependencies
pip install plotly pandas seaborn matplotlib numpy
```

## Basic Usage

### 1. Search for Articles

```bash
./target/release/curalit search \
  -k "cancer" \
  -k "immunotherapy" \
  -d ./data \
  -o my_research
```

This will:
- Search PubMed XML files in `./data`
- Find articles containing both "cancer" AND "immunotherapy"
- Save results to `my_research.csv`
- Generate statistics automatically

### 2. Review Statistics

```bash
./target/release/curalit stats -c my_research.csv
```

Check:
- `my_research_stats.log` - Summary report
- `my_research_stats.json` - Detailed data

### 3. Visualize Your Corpus

```bash
python my_research_visualize.py
```

Opens interactive HTML plots in your browser showing:
- Publication timeline
- Top MeSH terms
- Author networks
- Journal distribution

### 4. Generate LLM

```bash
./target/release/curalit generate \
  -c my_research.csv \
  -m cancer-assistant \
  -b llama3
```

### 5. Use Your Model

```bash
# Create model with Ollama
ollama create cancer-assistant -f Modelfile

# Run your custom LLM
ollama run cancer-assistant

# Now ask questions about your research corpus!
>>> What are the main findings about cancer immunotherapy?
```

## Common Patterns

### Broad Search (OR logic)
```bash
curalit search -k "diabetes" -k "insulin" -k "glucose" --logic OR -d ./data
```

### Keywords from File
```bash
echo "cancer" > keywords.txt
echo "treatment" >> keywords.txt
echo "clinical trial" >> keywords.txt

curalit search -f keywords.txt -d ./data -o clinical
```

### Resume Interrupted Search
```bash
curalit search -k "neuroscience" -d ./data --resume
```

## Tips

**Too many results?** (>1000 articles)
- Use AND logic (default)
- Add more specific keywords
- Include technical terms

**Too few results?** (<50 articles)
- Use OR logic (`--logic OR`)
- Use broader keywords
- Check for typos

**Need help?**
```bash
curalit big-help
```

## File Outputs

| File | Description |
|------|-------------|
| `results.csv` | All matched articles |
| `results_stats.json` | Statistics (machine-readable) |
| `results_stats.log` | Statistics (human-readable) |
| `results_visualize.py` | Visualization script |
| `Modelfile` | Ollama configuration |
| `results_training.jsonl` | Training data |
| `results_system_prompt.txt` | System prompt |

## Example Session

```bash
# 1. Search
$ curalit search -k "alzheimer" -k "treatment" -d ./pubmed_data -o alz

Found 156 articles!

# 2. Check stats
$ cat alz_stats.log | grep "Total Articles"
Total Articles: 156

# 3. Visualize
$ python alz_visualize.py
✓ Generated alz_dashboard.html

# 4. Generate model
$ curalit generate -c alz.csv -m alz-expert -b llama3
✓ Modelfile generated

# 5. Create and run
$ ollama create alz-expert -f Modelfile
$ ollama run alz-expert
>>> Tell me about current Alzheimer's treatments
[Your custom LLM responds based on the 156 articles!]
```

## Next Steps

- Read [README.md](README.md) for comprehensive guide
- Check [CONTRIBUTING.md](CONTRIBUTING.md) to contribute
- Run `curalit big-help` for detailed examples
- Visit the GitHub repo for updates

## Troubleshooting

**"No XML files found"**
- Check your `-d` directory path
- Ensure XML files exist in that directory

**"No keywords provided"**
- Use `-k` flag or `-f` with keywords file

**"Ollama command not found"**
- Install Ollama from https://ollama.ai

**Python visualization fails**
- Install dependencies: `pip install plotly pandas seaborn matplotlib numpy`

---

Happy researching! 🔬
