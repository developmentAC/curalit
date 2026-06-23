use crate::statistics::Statistics;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Generator for Python visualization scripts
pub struct VisualizationGenerator {
    stats: Statistics,
    output_prefix: String,
    output_dir: String,
    timestamp: String,
}

impl VisualizationGenerator {
    pub fn new(stats: Statistics, output_prefix: &str, output_dir: &Path, timestamp: &str) -> Self {
        Self {
            stats,
            output_prefix: output_prefix.to_string(),
            output_dir: output_dir.to_string_lossy().to_string(),
            timestamp: timestamp.to_string(),
        }
    }

    /// Generate Python visualization script
    pub fn generate(&self) -> Result<()> {
        // Ensure output directory exists
        fs::create_dir_all(&self.output_dir)
            .with_context(|| format!("Failed to create output directory: {}", self.output_dir))?;

        let script_path = format!(
            "{}/{}_{}_visualize.py",
            self.output_dir, self.output_prefix, self.timestamp
        );
        let script_content = self.create_visualization_script();
        fs::write(&script_path, script_content)?;

        // Make script executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&script_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&script_path, perms)?;
        }

        Ok(())
    }

    /// Create the Python visualization script
    fn create_visualization_script(&self) -> String {
        // Prepare data for the script
        let year_data = self.prepare_year_data();
        let mesh_data = self.prepare_mesh_data();
        let author_data = self.prepare_author_data();
        let journal_data = self.prepare_journal_data();
        let search_keywords = self.prepare_search_keywords();
        let articles_data = self.prepare_articles_data();

        format!(
r#"#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
CuraLit Visualization Script
Auto-generated interactive visualizations for literature analysis

Requirements:
    pip install plotly pandas seaborn matplotlib numpy pyvis networkx

Usage:
    python {}_{}_visualize.py
"""

import plotly.graph_objects as go
import plotly.express as px
from plotly.subplots import make_subplots
import pandas as pd
import json
import os
import re
from datetime import datetime

# Statistics data
TOTAL_ARTICLES = {}
ARTICLES_WITH_ABSTRACTS = {}
ARTICLES_WITH_DOI = {}
AVG_AUTHORS = {:.2}
AVG_MESH_TERMS = {:.2}

# Year distribution data
year_data = {}

# Top MeSH terms data (top 20)
mesh_data = {}

# Top authors data (top 20)
author_data = {}

# Top journals data (top 20)
journal_data = {}

# Search keywords
SEARCH_KEYWORDS = {}

# Article data
ARTICLES_DATA = {}


def create_year_distribution_plot():
    """Create interactive bar chart of publication years"""
    if not year_data:
        return None
    
    df = pd.DataFrame(list(year_data.items()), columns=['Year', 'Count'])
    df = df.sort_values('Year')
    
    fig = px.bar(
        df, 
        x='Year', 
        y='Count',
        title='Publication Year Distribution',
        labels={{'Year': 'Publication Year', 'Count': 'Number of Articles'}},
        color='Count',
        color_continuous_scale='viridis'
    )
    
    fig.update_layout(
        xaxis_title='Year',
        yaxis_title='Article Count',
        hovermode='x unified',
        template='plotly_white'
    )
    
    return fig


def create_keyword_article_network(max_articles=None, recent_years=3, show_all=False, use_mesh=False):
    """Create interactive network graph showing keyword-article relationships.
    
    Args:
        max_articles: Maximum number of articles to display (None for all recent)
        recent_years: Number of recent years to include by default
        show_all: If True, show all articles regardless of date
        use_mesh: If True, also show MeSH term connections
    """
    try:
        import networkx as nx
        from pyvis.network import Network
    except ImportError:
        print("  ⚠ pyvis and networkx required for network visualization")
        print("    Install with: pip install pyvis networkx")
        return None
    
    # Create network graph
    G = nx.Graph()
    
    # Add keyword nodes
    for keyword in SEARCH_KEYWORDS:
        G.add_node(keyword, node_type='keyword', label=keyword)
    
    # Filter articles by date if not showing all
    current_year = datetime.now().year
    filtered_articles = ARTICLES_DATA
    
    if not show_all:
        filtered_articles = []
        for article in ARTICLES_DATA:
            pub_date = article.get('pub_date', '')
            if pub_date:
                year_str = pub_date.split('-')[0]
                if year_str.isdigit() and (current_year - int(year_str)) <= recent_years:
                    filtered_articles.append(article)
        
        # If no recent articles found, show all articles (likely historical data)
        if len(filtered_articles) == 0:
            print(f"  ℹ No articles from last {{recent_years}} years found. Showing all {{len(ARTICLES_DATA)}} articles.")
            filtered_articles = ARTICLES_DATA
    
    # Limit number of articles if specified
    if max_articles and len(filtered_articles) > max_articles:
        # Randomly sample articles
        import random
        filtered_articles = random.sample(filtered_articles, max_articles)
    
    # Add article nodes and edges
    articles_added = 0
    for article in filtered_articles:
        pmid = article.get('pmid', 'Unknown')
        title = article.get('title', 'No Title')[:100]
        authors = article.get('authors', 'Unknown')
        journal = article.get('journal', 'Unknown')
        pub_date = article.get('pub_date', 'Unknown')
        abstract_text = article.get('abstract', '')
        mesh_terms = article.get('mesh_terms', '')
        
        # Create citation
        first_author = authors.split(';')[0].strip() if authors != 'Unknown' else 'Unknown'
        year = pub_date[:4] if len(pub_date) >= 4 else pub_date
        citation = f"{{first_author}} et al. ({{year}}). {{journal}}"
        
        # Create PubMed URL
        pubmed_url = f"https://pubmed.ncbi.nlm.nih.gov/{{pmid}}/"
        
        # Get matched keywords from article data (populated during search)
        article_keywords = article.get('keywords', '')
        if article_keywords:
            # Keywords are stored as semicolon-separated string
            matched_keywords = [kw.strip() for kw in article_keywords.split(';') if kw.strip()]
        else:
            # Fallback: check which keywords match by searching text
            searchable = (title + ' ' + abstract_text + ' ' + mesh_terms).lower()
            matched_keywords = [kw for kw in SEARCH_KEYWORDS if kw.lower() in searchable]
        
        if matched_keywords:
            # Create tooltip
            tooltip = f'''
            <div style="max-width:400px">
            <b>PMID:</b> {{pmid}}<br>
            <b>Title:</b> {{title}}<br>
            <b>Citation:</b> {{citation}}<br>
            <b>Keywords Found:</b> {{', '.join(matched_keywords)}}<br>
            <b>Year:</b> {{year}}<br>
            <b>Click to view on PubMed</b>
            </div>
            '''
            
            # Add article node
            article_node = f"PMID:{{pmid}}"
            G.add_node(
                article_node,
                node_type='article',
                label=f"PMID:{{pmid}}",
                title=tooltip,
                pmid=pmid,
                url=pubmed_url
            )
            
            # Add edges to matched keywords
            for keyword in matched_keywords:
                G.add_edge(article_node, keyword)
            
            # Optionally add MeSH term connections
            if use_mesh and mesh_terms:
                for mesh_term in mesh_terms.split(';')[:3]:  # Limit to top 3 MeSH terms
                    mesh_term = mesh_term.strip()
                    if mesh_term:
                        mesh_node = f"MeSH:{{mesh_term}}"
                        if not G.has_node(mesh_node):
                            G.add_node(mesh_node, node_type='mesh', label=mesh_term)
                        G.add_edge(article_node, mesh_node)
            
            articles_added += 1
    
    if articles_added == 0:
        print("  ⚠ No articles matched the criteria for network visualization")
        return None
    
    # Create PyVis network with CDN resources (no local files needed)
    net = Network(
        height='800px', 
        width='95%', 
        bgcolor='#ffffff', 
        font_color='black',
        cdn_resources='remote'  # Use CDN instead of local lib files
    )
    
    # Convert NetworkX to PyVis
    net.from_nx(G)
    
    # Customize node appearance
    for node in net.nodes:
        node_id = node['id']
        node_data = G.nodes.get(node_id, {{}})
        node_type = node_data.get('node_type', 'unknown')
        
        if node_type == 'keyword':
            node['color'] = '#5DADE2'  # Blue
            node['shape'] = 'box'
            node['size'] = 25
            node['font'] = {{'size': 16, 'face': 'arial', 'color': 'white'}}
            node['title'] = f"<b>Search Keyword:</b> {{node_id}}"
        elif node_type == 'article':
            node['color'] = '#58D68D'  # Green
            node['shape'] = 'dot'
            node['size'] = 15
            node['font'] = {{'size': 10, 'face': 'arial'}}
            # Add click handler for PubMed URL
            pmid = node_data.get('pmid', '')
            if pmid:
                url = f"https://pubmed.ncbi.nlm.nih.gov/{{pmid}}/"
                node['title'] = node_data.get('title', '') + f"<br><br><a href='{{url}}' target='_blank'>🔗 Open in PubMed</a>"
        elif node_type == 'mesh':
            node['color'] = '#F39C12'  # Orange
            node['shape'] = 'diamond'
            node['size'] = 12
            node['font'] = {{'size': 10, 'face': 'arial'}}
            node['title'] = f"<b>MeSH Term:</b> {{node_id.replace('MeSH:', '')}}"
    
    # Customize edges
    for edge in net.edges:
        edge['color'] = '#95A5A6'
        edge['width'] = 1
    
    # Enable interactive physics controls
    net.show_buttons(filter_=['physics'])
    
    return net, articles_added, len(filtered_articles)


def create_mesh_terms_plot():
    """Create horizontal bar chart of top MeSH terms"""
    if not mesh_data:
        return None
    
    df = pd.DataFrame(list(mesh_data.items()), columns=['MeSH Term', 'Frequency'])
    df = df.sort_values('Frequency', ascending=True).tail(20)
    
    fig = go.Figure(go.Bar(
        x=df['Frequency'],
        y=df['MeSH Term'],
        orientation='h',
        marker=dict(
            color=df['Frequency'],
            colorscale='blues',
            showscale=True,
            colorbar=dict(title='Frequency')
        ),
        text=df['Frequency'],
        textposition='auto',
    ))
    
    fig.update_layout(
        title='Top 20 MeSH Terms',
        xaxis_title='Frequency',
        yaxis_title='MeSH Term',
        height=600,
        template='plotly_white'
    )
    
    return fig


def create_author_network_plot():
    """Create bubble chart of top authors"""
    if not author_data:
        return None
    
    df = pd.DataFrame(list(author_data.items()), columns=['Author', 'Publications'])
    df = df.sort_values('Publications', ascending=False).head(20)
    
    fig = px.scatter(
        df,
        x=list(range(len(df))),
        y='Publications',
        size='Publications',
        color='Publications',
        hover_data=['Author'],
        title='Top 20 Authors by Publication Count',
        labels={{'x': 'Author Index', 'Publications': 'Number of Publications'}},
        color_continuous_scale='reds'
    )
    
    fig.update_layout(
        xaxis_title='Author Rank',
        yaxis_title='Publication Count',
        showlegend=False,
        template='plotly_white'
    )
    
    return fig


def create_journal_distribution():
    """Create pie chart of top journals"""
    if not journal_data:
        return None
    
    df = pd.DataFrame(list(journal_data.items()), columns=['Journal', 'Count'])
    df = df.sort_values('Count', ascending=False).head(15)
    
    # Add "Others" category
    other_count = TOTAL_ARTICLES - df['Count'].sum()
    if other_count > 0:
        df = pd.concat([df, pd.DataFrame([['Others', other_count]], columns=['Journal', 'Count'])])
    
    fig = px.pie(
        df,
        values='Count',
        names='Journal',
        title='Top 15 Journals (Distribution)',
        hole=0.3
    )
    
    fig.update_traces(textposition='inside', textinfo='percent+label')
    fig.update_layout(template='plotly_white')
    
    return fig


def create_summary_heatmap():
    """Create heatmap showing corpus characteristics"""
    categories = ['Total Articles', 'With Abstracts', 'With DOI', 'Unique Authors', 'Unique MeSH Terms']
    values = [
        TOTAL_ARTICLES,
        ARTICLES_WITH_ABSTRACTS,
        ARTICLES_WITH_DOI,
        len(author_data),
        len(mesh_data)
    ]
    
    fig = go.Figure(data=go.Bar(
        x=categories,
        y=values,
        marker=dict(
            color=values,
            colorscale='greens',
            showscale=True
        ),
        text=values,
        textposition='auto',
    ))
    
    fig.update_layout(
        title='Corpus Summary Statistics',
        xaxis_title='Category',
        yaxis_title='Count',
        template='plotly_white'
    )
    
    return fig


def create_comprehensive_dashboard():
    """Create a comprehensive dashboard with all visualizations"""
    fig = make_subplots(
        rows=3, cols=2,
        subplot_titles=(
            'Publication Year Distribution',
            'Top MeSH Terms',
            'Top Authors',
            'Journal Distribution',
            'Corpus Summary',
            'Statistics'
        ),
        specs=[
            [{{'type': 'bar'}}, {{'type': 'bar'}}],
            [{{'type': 'scatter'}}, {{'type': 'pie'}}],
            [{{'type': 'bar'}}, {{'type': 'table'}}]
        ],
        vertical_spacing=0.12,
        horizontal_spacing=0.1
    )
    
    # Year distribution
    if year_data:
        df_year = pd.DataFrame(list(year_data.items()), columns=['Year', 'Count'])
        df_year = df_year.sort_values('Year')
        fig.add_trace(
            go.Bar(x=df_year['Year'], y=df_year['Count'], name='Year', marker_color='lightblue'),
            row=1, col=1
        )
    
    # Top MeSH terms
    if mesh_data:
        df_mesh = pd.DataFrame(list(mesh_data.items()), columns=['MeSH', 'Count'])
        df_mesh = df_mesh.sort_values('Count', ascending=False).head(10)
        fig.add_trace(
            go.Bar(x=df_mesh['Count'], y=df_mesh['MeSH'], orientation='h', marker_color='lightgreen'),
            row=1, col=2
        )
    
    # Top authors
    if author_data:
        df_author = pd.DataFrame(list(author_data.items()), columns=['Author', 'Count'])
        df_author = df_author.sort_values('Count', ascending=False).head(15)
        fig.add_trace(
            go.Scatter(
                x=list(range(len(df_author))),
                y=df_author['Count'],
                mode='markers',
                marker=dict(size=df_author['Count'], color=df_author['Count'], colorscale='reds'),
                text=df_author['Author'],
                name='Authors'
            ),
            row=2, col=1
        )
    
    # Journal distribution
    if journal_data:
        df_journal = pd.DataFrame(list(journal_data.items()), columns=['Journal', 'Count'])
        df_journal = df_journal.sort_values('Count', ascending=False).head(10)
        fig.add_trace(
            go.Pie(labels=df_journal['Journal'], values=df_journal['Count'], name='Journals'),
            row=2, col=2
        )
    
    # Summary statistics
    summary_categories = ['Total', 'With Abstract', 'With DOI', 'Avg Authors', 'Avg MeSH']
    summary_values = [
        TOTAL_ARTICLES,
        ARTICLES_WITH_ABSTRACTS,
        ARTICLES_WITH_DOI,
        int(AVG_AUTHORS * 100) / 100,
        int(AVG_MESH_TERMS * 100) / 100
    ]
    fig.add_trace(
        go.Bar(x=summary_categories, y=summary_values, marker_color='lightsalmon'),
        row=3, col=1
    )
    
    # Statistics table
    stats_data = [
        ['Total Articles', str(TOTAL_ARTICLES)],
        ['With Abstracts', f'{{ARTICLES_WITH_ABSTRACTS}} ({{ARTICLES_WITH_ABSTRACTS*100//TOTAL_ARTICLES}}%)'],
        ['With DOI', f'{{ARTICLES_WITH_DOI}} ({{ARTICLES_WITH_DOI*100//TOTAL_ARTICLES}}%)'],
        ['Avg Authors/Article', f'{{AVG_AUTHORS:.2f}}'],
        ['Avg MeSH Terms/Article', f'{{AVG_MESH_TERMS:.2f}}'],
    ]
    fig.add_trace(
        go.Table(
            header=dict(values=['Metric', 'Value']),
            cells=dict(values=[[row[0] for row in stats_data], [row[1] for row in stats_data]])
        ),
        row=3, col=2
    )
    
    fig.update_layout(
        height=1200,
        showlegend=False,
        title_text="CuraLit Literature Analysis Dashboard",
        title_font_size=20
    )
    
    return fig


def main():
    """Generate and display all visualizations"""
    import os
    
    print("🔬 CuraLit Visualization Generator")
    print("=" * 60)
    print(f"Total Articles: {{TOTAL_ARTICLES}}")
    print(f"Articles with Abstracts: {{ARTICLES_WITH_ABSTRACTS}}")
    print(f"Articles with DOI: {{ARTICLES_WITH_DOI}}")
    print(f"Average Authors per Article: {{AVG_AUTHORS:.2f}}")
    print(f"Average MeSH Terms per Article: {{AVG_MESH_TERMS:.2f}}")
    print("=" * 60)
    print()
    
    # Create html subdirectory for output
    html_dir = os.path.join('{}', 'html')
    os.makedirs(html_dir, exist_ok=True)
    print(f"📁 Output directory: {{html_dir}}")
    print()
    
    # Generate individual plots
    print("Generating visualizations...")
    
    # Year distribution
    fig_year = create_year_distribution_plot()
    if fig_year:
        output_path = os.path.join(html_dir, '{}_{}_year_distribution.html')
        fig_year.write_html(output_path)
        print(f"✓ Year distribution: {{output_path}}")
    
    # MeSH terms
    fig_mesh = create_mesh_terms_plot()
    if fig_mesh:
        output_path = os.path.join(html_dir, '{}_{}_mesh_terms.html')
        fig_mesh.write_html(output_path)
        print(f"✓ MeSH terms: {{output_path}}")
    
    # Authors
    fig_authors = create_author_network_plot()
    if fig_authors:
        output_path = os.path.join(html_dir, '{}_{}_authors.html')
        fig_authors.write_html(output_path)
        print(f"✓ Authors: {{output_path}}")
    
    # Journals
    fig_journals = create_journal_distribution()
    if fig_journals:
        output_path = os.path.join(html_dir, '{}_{}_journals.html')
        fig_journals.write_html(output_path)
        print(f"✓ Journals: {{output_path}}")
    
    # Summary
    fig_summary = create_summary_heatmap()
    if fig_summary:
        output_path = os.path.join(html_dir, '{}_{}_summary.html')
        fig_summary.write_html(output_path)
        print(f"✓ Summary: {{output_path}}")
    
    # Comprehensive dashboard
    fig_dashboard = create_comprehensive_dashboard()
    output_path = os.path.join(html_dir, '{}_{}_dashboard.html')
    fig_dashboard.write_html(output_path)
    print(f"✓ Dashboard: {{output_path}}")
    
    # Keyword-Article Network Graph
    print()
    print("Generating keyword-article network graph...")
    network_result = create_keyword_article_network(max_articles=100, recent_years=3, show_all=False)
    if network_result:
        net, articles_shown, total_filtered = network_result
        output_path = os.path.join(html_dir, '{}_{}_keyword_network.html')
        net.save_graph(output_path)
        
        # Fix pyvis bug: remove reference to non-existent lib/bindings/utils.js
        with open(output_path, 'r', encoding='utf-8') as f:
            html_content = f.read()
        
        # Check if the problematic line exists before replacement
        if 'lib/bindings/utils.js' in html_content:
            # Use regex to remove the script tag and following newline
            html_content = re.sub(r'<script src="lib/bindings/utils\.js"></script>\s*\n', '', html_content)
            with open(output_path, 'w', encoding='utf-8') as f:
                f.write(html_content)
            print("  ℹ Fixed: Removed problematic utils.js reference")
        
        print(f"✓ Keyword-Article Network: {{output_path}}")
        print(f"  → Showing {{articles_shown}} articles (filtered from {{total_filtered}} recent articles)")
        print(f"  → Keywords: {{', '.join(SEARCH_KEYWORDS)}}")
        print(f"  → Click on article nodes to open PubMed pages")
        print(f"  → To show all articles, edit script and set show_all=True")
    
    print()
    print("=" * 60)
    print("✓ All visualizations generated successfully!")
    print(f"  📂 All HTML files are in: {{html_dir}}")
    print("  Open the HTML files in your web browser to explore the data.")
    print("=" * 60)


if __name__ == '__main__':
    main()
"#,
            self.output_prefix,
            self.timestamp,
            self.stats.total_articles,
            self.stats.articles_with_abstracts,
            self.stats.articles_with_doi,
            self.stats.avg_authors_per_article,
            self.stats.avg_mesh_terms_per_article,
            year_data,
            mesh_data,
            author_data,
            journal_data,
            search_keywords,
            articles_data,
            self.output_dir,
            self.output_prefix,
            self.timestamp,
            self.output_prefix,
            self.timestamp,
            self.output_prefix,
            self.timestamp,
            self.output_prefix,
            self.timestamp,
            self.output_prefix,
            self.timestamp,
            self.output_prefix,
            self.timestamp,
            self.output_prefix,
            self.timestamp,
        )
    }

    fn prepare_year_data(&self) -> String {
        let items: Vec<String> = self
            .stats
            .year_distribution
            .iter()
            .map(|(k, v)| format!("'{}': {}", k, v))
            .collect();
        format!("{{{}}}", items.join(", "))
    }

    fn prepare_mesh_data(&self) -> String {
        let items: Vec<String> = self
            .stats
            .top_mesh_terms
            .iter()
            .map(|(k, v)| format!("'{}': {}", k.replace('\'', "\\'"), v))
            .collect();
        format!("{{{}}}", items.join(", "))
    }

    fn prepare_author_data(&self) -> String {
        let items: Vec<String> = self
            .stats
            .top_authors
            .iter()
            .map(|(k, v)| format!("'{}': {}", k.replace('\'', "\\'"), v))
            .collect();
        format!("{{{}}}", items.join(", "))
    }

    fn prepare_journal_data(&self) -> String {
        let items: Vec<String> = self
            .stats
            .top_journals
            .iter()
            .map(|(k, v)| format!("'{}': {}", k.replace('\'', "\\'"), v))
            .collect();
        format!("{{{}}}", items.join(", "))
    }

    fn prepare_search_keywords(&self) -> String {
        let items: Vec<String> = self
            .stats
            .search_keywords
            .iter()
            .map(|k| format!("'{}'", k.replace('\'', "\\'")))
            .collect();
        format!("[{}]", items.join(", "))
    }

    fn prepare_articles_data(&self) -> String {
        let articles: Vec<String> = self
            .stats
            .articles
            .iter()
            .map(|article| {
                let pmid = article.pmid.replace('\'', "\\'");
                let title = article.title.replace('\'', "\\'").replace('\n', " ");
                let authors = article.authors.join("; ").replace('\'', "\\'");
                let journal = article.journal.replace('\'', "\\'");
                let pub_date = article.pub_date.replace('\'', "\\'");
                let abstract_text = article.abstract_text.replace('\'', "\\'").replace('\n', " ");
                let mesh_terms = article.mesh_terms.join("; ").replace('\'', "\\'");
                let keywords = article.keywords.join("; ").replace('\'', "\\'");
                
                format!(
                    "{{'pmid': '{}', 'title': '{}', 'authors': '{}', 'journal': '{}', 'pub_date': '{}', 'abstract': '{}', 'mesh_terms': '{}', 'keywords': '{}'}}",
                    pmid, title, authors, journal, pub_date, abstract_text, mesh_terms, keywords
                )
            })
            .collect();
        format!("[{}]", articles.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn test_visualization_generation() {
        let stats = Statistics {
            total_articles: 100,
            keyword_frequencies: HashMap::new(),
            mesh_term_frequencies: HashMap::new(),
            chemical_frequencies: HashMap::new(),
            author_frequencies: HashMap::new(),
            journal_frequencies: HashMap::new(),
            year_distribution: HashMap::new(),
            top_keywords: Vec::new(),
            top_mesh_terms: vec![("Cancer".to_string(), 50)],
            top_authors: vec![("Smith, J".to_string(), 10)],
            top_journals: vec![("Nature".to_string(), 5)],
            avg_authors_per_article: 3.5,
            avg_mesh_terms_per_article: 8.2,
            articles_with_abstracts: 95,
            articles_with_doi: 90,
            search_keywords: vec!["cancer".to_string(), "immunotherapy".to_string()],
            articles: Vec::new(),
        };

        let output_dir = PathBuf::from("0_out");
        let generator = VisualizationGenerator::new(stats, "test", &output_dir, "20260522_120000");
        let script = generator.create_visualization_script();

        assert!(script.contains("TOTAL_ARTICLES = 100"));
        assert!(script.contains("plotly"));
        assert!(script.contains("def main():"));
    }
}
