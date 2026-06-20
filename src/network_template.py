#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
CuraLit Network Graph Generator
Auto-generated interactive network visualization showing keyword-article relationships

Requirements:
    pip install pandas networkx pyvis

Usage:
    python network.py --datafile output.csv --threshold 1 --auto_render yes
"""

import os
import pandas as pd
import networkx as nx
from pyvis.network import Network
import argparse


# Configuration
DATA_PATH = "0_out"
OUT_FILE = "network_graph"


def check_data_dir(dir_str):
    """Check if data output directory exists, create if not."""
    try:
        os.makedirs(dir_str, exist_ok=True)
        return True
    except OSError:
        return False


def load_csv_file(file_path):
    """Load the CSV file and return a dataframe."""
    if file_path is None:
        print("\t No file to load! Exiting ...")
        return None
    
    print(f"\t Loading data from: {file_path}")
    try:
        df = pd.read_csv(file_path)
        print(f"\t * Data loaded successfully: {len(df)} articles")
        return df
    except Exception as e:
        print(f"\t * Error loading file: {e}")
        return None


def create_graph_from_csv(df, threshold, keywords_to_search):
    """Create network graph from article CSV data."""
    G = nx.Graph()
    
    # Add keyword nodes
    for keyword in keywords_to_search:
        G.add_node(keyword, type="keyword", node_type="keyword")
    
    # Process each article
    for idx, row in df.iterrows():
        pmid = str(row.get('PMID', f'Article_{idx}'))
        title = str(row.get('Title', 'No Title'))[:100]  # Truncate long titles
        authors = str(row.get('Authors', 'Unknown'))
        journal = str(row.get('Journal', 'Unknown'))
        pub_date = str(row.get('Publication Date', 'Unknown'))
        
        # Create citation
        first_author = authors.split(';')[0].strip() if authors != 'Unknown' else 'Unknown'
        citation = f"{first_author} et al. ({pub_date[:4] if len(pub_date) >= 4 else pub_date}). {journal}"
        
        # Create PubMed URL
        pubmed_url = f"https://pubmed.ncbi.nlm.nih.gov/{pmid}/"
        
        # Get all searchable text
        searchable_text = (
            str(row.get('Title', '')) + ' ' +
            str(row.get('Abstract', '')) + ' ' +
            str(row.get('MeSH Terms', '')) + ' ' +
            str(row.get('Chemicals', '')) + ' ' +
            str(row.get('Keywords', ''))
        ).lower()
        
        # Find matching keywords
        matched_keywords = []
        for keyword in keywords_to_search:
            if keyword.lower() in searchable_text:
                matched_keywords.append(keyword)
        
        # Only add article node if it matches at least threshold keywords
        if len(matched_keywords) >= threshold:
            # Create tooltip with article information
            tooltip = f"""
            <b>PMID:</b> {pmid}<br>
            <b>Title:</b> {title}<br>
            <b>Citation:</b> {citation}<br>
            <b>Keywords Found:</b> {', '.join(matched_keywords)}<br>
            <b>PubMed URL:</b> <a href='{pubmed_url}' target='_blank'>{pubmed_url}</a>
            """
            
            # Add article node
            article_label = f"PMID:{pmid}"
            G.add_node(
                article_label,
                type="article",
                node_type="article",
                pmid=pmid,
                title=title,
                citation=citation,
                url=pubmed_url,
                tooltip=tooltip,
                keywords=matched_keywords
            )
            
            # Add edges to matched keywords
            for keyword in matched_keywords:
                G.add_edge(article_label, keyword, weight=1)
    
    return G


def visualize_graph(
    G,
    layout_algorithm="forceAtlas2",
    auto_render=True,
    keyword_color="lightblue",
    article_color="lightgreen",
    threshold=1,
    output_prefix="",
    timestamp=""
):
    """Create interactive network visualization with Pyvis."""
    net = Network(height="800px", width="100%", notebook=False, bgcolor="#ffffff")
    
    # Convert the NetworkX graph to Pyvis format
    net.from_nx(G)
    
    # Customize nodes
    for node in net.nodes:
        node_id = node["id"]
        node_data = G.nodes.get(node_id, {})
        node_type = node_data.get("node_type", "unknown")
        
        if node_type == "keyword":
            # Keyword nodes
            node["color"] = keyword_color
            node["title"] = f"<b>Keyword:</b> {node_id}"
            node["size"] = 25
            node["font"] = {"size": 16, "color": "black", "face": "arial"}
            node["shape"] = "box"
        elif node_type == "article":
            # Article nodes
            node["color"] = article_color
            node["title"] = node_data.get("tooltip", f"Article: {node_id}")
            node["size"] = 15
            node["font"] = {"size": 12, "color": "black", "face": "arial"}
            node["shape"] = "dot"
    
    # Customize edges
    for edge in net.edges:
        edge["color"] = "#888888"
        edge["width"] = 1
    
    # Set the layout algorithm
    if layout_algorithm == "forceAtlas2":
        net.force_atlas_2based(
            gravity=-50,
            central_gravity=0.01,
            spring_length=100,
            spring_strength=0.08,
            damping=0.4
        )
    elif layout_algorithm == "barnesHut":
        net.barnes_hut()
    
    # Enable physics controls
    net.show_buttons(filter_=['physics'])
    net.toggle_physics(True)
    
    # Set options for better interactivity
    net.set_options("""
    {
      "physics": {
        "enabled": true,
        "stabilization": {
          "enabled": true,
          "iterations": 100
        }
      },
      "interaction": {
        "hover": true,
        "tooltipDelay": 100,
        "navigationButtons": true,
        "keyboard": true
      }
    }
    """)
    
    # Ensure data directory exists
    check_data_dir(DATA_PATH)
    
    # Save the network
    save_file = os.path.join(
        DATA_PATH,
        f"{output_prefix}_{timestamp}_threshold{threshold}_{OUT_FILE}.html"
    )
    
    net.save_graph(save_file)
    
    if auto_render:
        print(f"\n\t ✓ Interactive network graph saved as: {save_file}")
        print(f"\t   Open this file in a web browser to explore the network.")
        print(f"\t   - Keywords are shown as boxes ({keyword_color})")
        print(f"\t   - Articles are shown as dots ({article_color})")
        print(f"\t   - Hover over articles to see citation, PMID, and PubMed URL")
    
    return save_file


def main():
    """Main function to create keyword-article network visualization."""
    parser = argparse.ArgumentParser(
        description="Generate a keyword-article network visualization from CuraLit results."
    )
    
    parser.add_argument(
        "--datafile",
        type=str,
        required=True,
        help="Path to the CSV file containing article data"
    )
    
    parser.add_argument(
        "--threshold",
        type=int,
        default=1,
        help="Minimum number of keyword matches for an article to be included (default: 1)"
    )
    
    parser.add_argument(
        "--layout",
        type=str,
        choices=["forceAtlas2", "barnesHut"],
        default="forceAtlas2",
        help="Layout algorithm (default: forceAtlas2)"
    )
    
    parser.add_argument(
        "--auto_render",
        type=str,
        choices=["yes", "no"],
        default="yes",
        help="Automatically save graph? (default: yes)"
    )
    
    parser.add_argument(
        "--keyword_color",
        type=str,
        default="lightblue",
        help="Color for keyword nodes (default: lightblue)"
    )
    
    parser.add_argument(
        "--article_color",
        type=str,
        default="lightgreen",
        help="Color for article nodes (default: lightgreen)"
    )
    
    parser.add_argument(
        "--keywords",
        type=str,
        nargs='+',
        help="List of keywords to visualize (space-separated)"
    )
    
    args = parser.parse_args()
    
    # Load the CSV data
    df = load_csv_file(args.datafile)
    if df is None:
        return
    
    # Determine keywords to search
    # If keywords are provided via command line, use those
    # Otherwise, try to extract from the Keywords column
    keywords_to_search = args.keywords
    
    if not keywords_to_search:
        # Try to extract unique keywords from the Keywords column
        if 'Keywords' in df.columns:
            all_keywords = set()
            for keywords_str in df['Keywords'].dropna():
                # Split by semicolon or comma
                keywords_list = [k.strip() for k in str(keywords_str).replace(';', ',').split(',')]
                all_keywords.update(keywords_list)
            keywords_to_search = list(all_keywords)
            print(f"\t Extracted {len(keywords_to_search)} unique keywords from data")
        else:
            print("\t Warning: No keywords specified and no Keywords column found")
            keywords_to_search = []
    
    if not keywords_to_search:
        print("\t Error: No keywords to visualize!")
        return
    
    print(f"\t Visualizing network for keywords: {', '.join(keywords_to_search)}")
    
    # Create the graph
    G = create_graph_from_csv(df, args.threshold, keywords_to_search)
    
    if G.number_of_nodes() == 0:
        print("\t Warning: No nodes in graph. Try lowering the threshold.")
        return
    
    print(f"\t Network created: {G.number_of_nodes()} nodes, {G.number_of_edges()} edges")
    
    # Extract output prefix and timestamp from filename
    import re
    filename = os.path.basename(args.datafile)
    match = re.match(r'(.+?)_(\d+\w+\d+_\d+)\.csv', filename)
    if match:
        output_prefix = match.group(1)
        timestamp = match.group(2)
    else:
        output_prefix = "network"
        timestamp = "default"
    
    # Visualize the graph
    visualize_graph(
        G,
        args.layout,
        args.auto_render.lower() == "yes",
        args.keyword_color,
        args.article_color,
        args.threshold,
        output_prefix,
        timestamp
    )


if __name__ == "__main__":
    print("\n" + "="*70)
    print("  CuraLit Network Graph Generator")
    print("  Interactive keyword-article network visualization")
    print("="*70)
    print("\n  For help: python3 {} --help\n".format(__file__))
    main()
    print("\n" + "="*70)
    print("  ✓ Network generation complete!")
    print("="*70 + "\n")
