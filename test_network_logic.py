#!/usr/bin/env python3
"""
Test script to verify network generation logic without requiring plotly/pyvis
"""

from datetime import datetime

# Sample data (from actual output)
SEARCH_KEYWORDS = ['cancer', 'immunotherapy']
ARTICLES_DATA = [
    {'pmid': '1145', 'title': 'Immunotherapy for cancer', 'authors': 'Proctor, J W', 
     'journal': 'Canadian journal', 'pub_date': '1976-Jan', 
     'abstract': 'Immunotherapy of cancer is of interest', 'mesh_terms': 'Immunotherapy; Neoplasms'},
    {'pmid': '4748', 'title': 'New acquisitions in tumor biology', 'authors': 'Badellino, F', 
     'journal': 'Minerva medica', 'pub_date': '1976-Mar-24', 
     'abstract': '', 'mesh_terms': 'Antigens, Neoplasm; Immunotherapy'},
]

def test_network_filtering(max_articles=None, recent_years=3, show_all=False):
    """Test the network filtering logic"""
    
    # Filter articles by date if not showing all
    current_year = datetime.now().year
    filtered_articles = ARTICLES_DATA
    
    print(f"Current year: {current_year}")
    print(f"Total articles: {len(ARTICLES_DATA)}")
    print(f"Recent years filter: {recent_years}")
    print(f"Show all: {show_all}")
    print()
    
    if not show_all:
        filtered_articles = []
        for article in ARTICLES_DATA:
            pub_date = article.get('pub_date', '')
            if pub_date:
                year_str = pub_date.split('-')[0]
                if year_str.isdigit():
                    age = current_year - int(year_str)
                    print(f"  Article {article['pmid']}: year {year_str}, age {age} years")
                    if age <= recent_years:
                        filtered_articles.append(article)
                        print(f"    ✓ Included (within {recent_years} years)")
                    else:
                        print(f"    ✗ Excluded (older than {recent_years} years)")
        
        # If no recent articles found, show all articles (likely historical data)
        if len(filtered_articles) == 0:
            print(f"\n  ℹ No articles from last {recent_years} years found.")
            print(f"  ℹ Showing all {len(ARTICLES_DATA)} articles (likely historical data).")
            filtered_articles = ARTICLES_DATA
    
    print(f"\nFiltered articles count: {len(filtered_articles)}")
    
    # Check keyword matches
    articles_with_matches = 0
    for article in filtered_articles:
        title = article.get('title', '')
        abstract = article.get('abstract', '')
        mesh = article.get('mesh_terms', '')
        searchable = (title + ' ' + abstract + ' ' + mesh).lower()
        
        matched_keywords = [kw for kw in SEARCH_KEYWORDS if kw.lower() in searchable]
        if matched_keywords:
            articles_with_matches += 1
            print(f"  Article {article['pmid']}: matches {matched_keywords}")
    
    print(f"\nArticles with keyword matches: {articles_with_matches}")
    
    if articles_with_matches == 0:
        print("\n⚠ WARNING: No articles matched the criteria for network visualization")
        return None
    else:
        print(f"\n✓ SUCCESS: Would generate network with {articles_with_matches} articles")
        return f"Network with {articles_with_matches} nodes"

if __name__ == '__main__':
    print("=" * 70)
    print("Testing Network Filtering Logic")
    print("=" * 70)
    print()
    
    print("Test 1: Default settings (recent_years=3, show_all=False)")
    print("-" * 70)
    result = test_network_filtering(recent_years=3, show_all=False)
    print()
    
    print("\n" + "=" * 70)
    print("Test 2: Show all articles (show_all=True)")
    print("-" * 70)
    result = test_network_filtering(show_all=True)
    print()
