# CuraLit Poster

This directory contains a professional academic poster about CuraLit, perfect for conferences, lab displays, or printed materials.

## Contents

- **poster.qmd** - Main Quarto poster source (Typst format)
- **poster-html.qmd** - Alternative HTML poster version
- **README.md** - This file

## Poster Formats

### Option 1: Typst Poster (Professional Print) - REQUIRES EXTENSION

The main poster uses Quarto's Typst format for high-quality PDF output suitable for professional printing.

**Note**: This requires the Quarto poster extension which is not yet officially released. For now, use the HTML version (Option 2) which works perfectly.

**Size**: 36" × 24" (standard conference poster)

**Rendering** (when extension available):
```bash
cd poster
quarto render poster.qmd
```

**Output**: `poster.pdf`

**Alternative PDF**: Export the HTML version to PDF via browser (File → Print → Save as PDF)

### Option 2: HTML Poster (Web Display) - RECOMMENDED

The HTML version is fully functional and ready to use. Great for web display, printing via browser, or quick previewing.

**Rendering**:
```bash
cd poster
quarto render poster-html.qmd
```

**Output**: `poster-html.html`

## Poster Sections

### 📋 Content Organization

1. **Abstract** - Quick overview of CuraLit's purpose and capabilities
2. **Introduction** - Research challenge and solution
3. **System Architecture** - Technical components and data flow
4. **Use Cases** - Real-world applications with examples
5. **Quick Start Guide** - Installation and workflow
6. **Results & Performance** - Benchmarks and metrics
7. **Advanced Features** - Keyword strategies and distribution
8. **Comparison** - vs. existing tools
9. **Limitations & Future** - Current constraints and roadmap
10. **Conclusion** - Key takeaways and getting started
11. **Acknowledgments** - Credits and resources

### 🎨 Visual Elements

- **Mermaid Diagram** - Workflow visualization
- **Code Examples** - Real command usage
- **Tables** - Performance metrics and comparisons
- **Callout Boxes** - Highlighted tips and notes
- **Color Coding** - Section backgrounds for organization

## Customization

### Change Poster Size

Edit the YAML header in `poster.qmd`:

```yaml
format:
  poster-typst: 
    size: "48x36"  # Options: "36x24", "48x36", "A0", "A1", etc.
```

### Modify Colors

Sections use fill colors. Edit block headers:

```markdown
# Section Name {.block fill="lightblue" color="black"}
```

Available colors:
- `lightblue`, `lightgreen`, `lightyellow`, `lightcyan`
- `lavender`, `lightpink`, `lightsalmon`, `lightgoldenrodyellow`
- `lightsteelblue`, `lightseagreen`, etc.

### Update Content

Each section is a level-1 heading with `.block` class:

```markdown
# Your Section {.block fill="yourcolor"}

Content goes here...
```

## Rendering Tips

### Prerequisites

Install Quarto with Typst support:

```bash
# Quarto includes Typst by default
quarto --version  # Should be 1.4+ for poster support
```

### Generate PDF

```bash
# From poster directory
quarto render poster.qmd

# From project root
quarto render poster/poster.qmd
```

### Preview Changes

```bash
# Live preview (HTML version recommended for quick iteration)
quarto preview poster-html.qmd
```

### Print Settings

**Recommended Settings**:
- **Size**: 36" × 24" (standard) or 48" × 36" (large)
- **Resolution**: 300 DPI minimum
- **Format**: PDF (vector graphics)
- **Paper**: Matte or glossy photo paper
- **Mounting**: Foam board or poster board

**Cost Estimate**:
- 36"×24" print: $50-100
- 48"×36" print: $100-150
- Mounting: +$30-50

**Online Printers**:
- Vistaprint
- FedEx Office
- UPS Store
- Local university print shops (often cheaper!)

## Display Options

### Conference Booth
- Print at recommended size
- Mount on foam board
- Add QR code for GitHub repo
- Bring handouts with key commands

### Lab Wall Display
- Laminate for durability
- Update regularly with new features
- Add post-it notes for team tips

### Virtual Presentation
- Use HTML version
- Share PDF link
- Enable screen reader accessibility

### Handouts
- Print at 11"×17" (tabloid)
- Double-sided with quick reference
- Include QR code and contact info

## Editing Workflow

### 1. Edit Content
```bash
# Open in editor
code poster.qmd  # VS Code
vim poster.qmd   # Vim
```

### 2. Preview Changes
```bash
# Quick HTML preview
quarto render poster-html.qmd
open poster-html.html
```

### 3. Final PDF
```bash
# Generate print-ready PDF
quarto render poster.qmd
open poster.pdf
```

### 4. Version Control
```bash
git add poster/
git commit -m "Update poster with new features"
```

## Troubleshooting

### Typst Not Found

The Quarto poster-typst extension is not yet officially released. **Solution**: Use the HTML version (poster-html.qmd) which works perfectly and can be printed to PDF via your browser.

```bash
# Use HTML version instead
quarto render poster-html.qmd

# Then print to PDF in browser
# File → Print → Save as PDF
# Set paper size to Tabloid (11"×17") or A3 for best results
```

### Rendering Fails

```bash
# Check YAML syntax
# Ensure all sections are properly formatted
# Try HTML version first to debug content
quarto render poster-html.qmd --verbose
```

### Layout Issues

- Adjust column widths: `{.column width="48%"}`
- Change block sizes: Content automatically flows
- Modify margins in YAML header

### Text Too Small

- Reduce content in sections
- Use bullet points instead of paragraphs
- Increase poster size to 48×36

### Images Not Displaying

- Ensure Mermaid diagrams render
- Check file paths for any linked images
- Use embedded resources in YAML

## Accessibility

### Screen Readers

The HTML version includes proper semantic markup:
- Heading hierarchy
- Alt text for diagrams
- ARIA labels

### Color Contrast

All text/background combinations meet WCAG AA standards:
- Black text on light backgrounds
- High contrast callout boxes

### Font Size

Poster uses large, readable fonts:
- Headings: 24-36pt
- Body text: 14-18pt
- Code: 12-14pt monospace

## Updating for New Versions

When CuraLit is updated:

1. Update version number in footer
2. Add new features to appropriate sections
3. Update command examples if syntax changed
4. Refresh performance benchmarks
5. Add new use cases if applicable

**Quick checklist**:
```markdown
- [ ] Version number in YAML footer
- [ ] All command examples tested
- [ ] Performance metrics current
- [ ] Links and contacts updated
- [ ] New features documented
```

## Exporting Options

### High-Resolution PDF
```bash
quarto render poster.qmd --to pdf
# Output: poster.pdf (print-ready)
```

### PNG Image
```bash
# Convert PDF to PNG (requires ImageMagick)
convert -density 300 poster.pdf -quality 100 poster.png
```

### PowerPoint
```bash
# For easy editing in PowerPoint
quarto render poster.qmd --to pptx
```

## Templates

### Creating Variations

Copy and customize for different audiences:

```bash
# Conference version (technical)
cp poster.qmd poster-conference.qmd

# Undergraduate version (simpler)
cp poster.qmd poster-undergrad.qmd

# Clinical version (medical focus)
cp poster.qmd poster-clinical.qmd
```

Edit each for specific audience needs.

## Best Practices

### Content
- ✅ Use short, punchy sentences
- ✅ Include visual elements every section
- ✅ Highlight key numbers and metrics
- ✅ Show real code examples
- ❌ Avoid dense paragraphs
- ❌ Don't overcrowd sections

### Design
- ✅ Consistent color scheme
- ✅ Clear visual hierarchy
- ✅ White space for readability
- ✅ Logical flow top-to-bottom
- ❌ Too many colors
- ❌ Tiny fonts

### Presenting
- ✅ Practice 2-minute pitch
- ✅ Prepare for common questions
- ✅ Have laptop demo ready
- ✅ Bring business cards/handouts
- ❌ Read directly from poster
- ❌ Block poster while talking

## Resources

### Quarto Documentation
- Poster guide: https://quarto.org/docs/output-formats/typst.html
- Typst format: https://typst.app/docs

### Design Inspiration
- BetterPosters.com
- Academic poster galleries
- Conference poster sessions

### Printing Services
- Local university print shops
- FedEx Office / UPS Store
- Online: Vistaprint, PosterSession.ai
- Scientific poster specialists

## License

This poster is part of the CuraLit project and follows the same MIT license.

## Questions?

- Open an issue on GitHub
- Check main project README
- Email: contact@curalit-project.org

---

**Happy Presenting! 📊✨**
