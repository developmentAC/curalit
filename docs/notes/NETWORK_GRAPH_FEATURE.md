# Network Graph Visualization Feature

## Overview

CuraLit now includes an interactive network graph visualization that shows the relationships between keywords and articles from your PubMed corpus analysis.

## Features

The network graph visualization:
- Shows **keywords as blue boxes** and **articles as green dots**
- Displays edges connecting articles to their matching keywords
- Provides interactive tooltips on mouse-over with:
  - Article PMID
  - Article title
  - Citation information (author, year, journal)
  - List of matching keywords
  - Direct link to PubMed article (https://pubmed.ncbi.nlm.nih.gov/PMID/)
- Interactive controls for physics simulation, zooming, and navigation
- Customizable colors and layout algorithms

## How It Works

When you run a keyword search with CuraLit, two Python visualization scripts are automatically generated in the `0_out/` directory:

1. `<name>_<timestamp>_visualize.py` - Statistical visualizations
2. `<name>_<timestamp>_network.py` - Network graph visualization (NEW!)

## Usage

### Basic Usage

After running a CuraLit search, generate the network graph with:

```bash
uv run 0_out/<name>_<timestamp>_network.py --datafile 0_out/<name>_<timestamp>.csv
```

This will create an interactive HTML file in `0_out/` that you can open in your web browser.

### Advanced Options

```bash
uv run 0_out/network.py --datafile output.csv \
    --threshold 2 \
    --layout forceAtlas2 \
    --keyword_color purple \
    --article_color lightblue \
    --keywords diabetes "insulin resistance" obesity
```

#### Parameters:

- `--datafile` (required): Path to the CSV file containing article data
- `--threshold` (default: 1): Minimum number of keyword matches for an article to be included
- `--layout` (default: forceAtlas2): Layout algorithm (choices: forceAtlas2, barnesHut)
- `--auto_render` (default: yes): Automatically save the graph (choices: yes, no)
- `--keyword_color` (default: lightblue): Color for keyword nodes
- `--article_color` (default: lightgreen): Color for article nodes
- `--keywords`: Specific keywords to visualize (space-separated). If not provided, all keywords from the CSV will be used.

## Requirements

The network visualization uses UV for dependency management:

```bash
# Install UV (one-time setup)
curl -LsSf https://astral.sh/uv/install.sh | sh
# Or: pip install uv

# Dependencies (pandas, networkx, pyvis) are auto-installed when you run the script
```

## Output

The network graph is saved as an HTML file with the naming pattern:

```
<name>_<timestamp>_threshold<N>_network_graph.html
```

Open this file in any modern web browser to explore the interactive visualization.

## Interacting with the Visualization

- **Zoom**: Use mouse wheel or pinch gesture
- **Pan**: Click and drag on empty space
- **Select node**: Click on a node
- **View tooltip**: Hover over any node
- **Physics controls**: Use the control panel on the right to adjust simulation parameters

## Example Workflow

1. Search PubMed data:
   ```bash
   curalit search -k diabetes "insulin resistance" -d ./data -o diabetes_results
   ```

2. CuraLit generates the network script automatically

3. Run the network visualization:
   ```bash
   uv run 0_out/diabetes_results_*_network.py --datafile 0_out/diabetes_results_*.csv
   ```

4. Open the generated HTML file in your browser to explore the keyword-article network

## Tips

- **Large datasets**: Use a higher `--threshold` value to reduce the number of nodes and make the graph more readable
- **Specific focus**: Use the `--keywords` parameter to visualize only specific keywords of interest
- **Color coding**: Customize node colors to match your presentation or publication theme
- **Layout**: Try different layout algorithms (forceAtlas2 vs barnesHut) to find the best visualization for your data

## Troubleshooting

If you encounter issues:

1. Ensure UV is installed: `curl -LsSf https://astral.sh/uv/install.sh | sh`
2. Verify the CSV file path is correct
3. Check that the CSV file contains the expected columns (PMID, Title, Authors, etc.)
4. For large datasets, try increasing the threshold to reduce memory usage
