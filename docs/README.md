# CuraLit Presentation

This directory contains a comprehensive Quarto presentation about CuraLit, designed for researchers new to the tool.

## Contents

- **presentation.qmd** - Main Quarto slide deck source
- **custom.css** - Custom styling for colorful, accessible slides
- **README.md** - This file

## Rendering the Presentation

### Prerequisites

You need Quarto installed. Download from: https://quarto.org/docs/get-started/

```bash
# Install Quarto (Linux/Mac)
# Download the appropriate installer from quarto.org

# Verify installation
quarto --version
```

### Generate HTML Slides

```bash
# From the docs directory
cd docs
quarto render presentation.qmd

# Or from project root
quarto render docs/presentation.qmd
```

This creates `presentation.html` in the `docs/` directory.

### View the Presentation

```bash
# Open in your browser
open presentation.html

# Or use Python's built-in server
python -m http.server 8000
# Then visit: http://localhost:8000/presentation.html
```

### Live Preview While Editing

```bash
# Auto-reload on changes
quarto preview presentation.qmd
```

## Presentation Features

### 🎨 Design
- **Theme:** Sky (blue gradient, professional)
- **Format:** RevealJS (interactive web-based)
- **Style:** Beginner-friendly, colorful, engaging

### 📋 Content Coverage

1. **Introduction** - What is CuraLit and why use it
2. **Installation** - Step-by-step setup guide
3. **Workflow** - Complete 6-step process
4. **Commands** - Search, stats, visualize, generate, package
5. **Ollama Integration** - Managing and using AI models
6. **Best Practices** - Tips for research success
7. **Troubleshooting** - Common issues and solutions
8. **Real Example** - Complete walkthrough with diabetes research
9. **Advanced Features** - Beyond the basics
10. **Ethics & Privacy** - Responsible usage

### 🎯 Target Audience

- Novice researchers
- Users new to command-line tools
- Anyone wanting to create custom AI assistants
- Research teams looking to adopt CuraLit

### 💡 Interactive Features

- Click through slides with arrow keys
- Press 'F' for fullscreen
- Press 'S' for speaker notes
- Press 'O' for overview mode
- Incremental reveals for key points
- Embedded diagrams and examples

## Customization

### Change Theme

Edit the YAML header in `presentation.qmd`:

```yaml
format:
  revealjs:
    theme: moon  # Options: sky, moon, solarized, blood, league, etc.
```

### Modify Styling

Edit `custom.css` to adjust:
- Colors
- Fonts
- Spacing
- Callout box styles
- Code block appearance

### Add Content

Slides use Quarto markdown:

```markdown
# New Slide Title {background-color="#e3f2fd"}

## Slide content here

- Bullet points
- More content
```

## Export Options

### PDF Export

```bash
# Create PDF (requires Chrome/Chromium)
quarto render presentation.qmd --to pdf
```

### PowerPoint

```bash
# Create PPTX
quarto render presentation.qmd --to pptx
```

### Standalone HTML

The default HTML is standalone with embedded resources - perfect for sharing via email or USB drive.

## Presenting Tips

### For Virtual Presentations

1. Use fullscreen mode (F key)
2. Share your screen
3. Use speaker notes (S key) for reference
4. Consider screen resolution of viewers

### For In-Person Presentations

1. Test on presentation computer beforehand
2. Have PDF backup ready
3. Ensure internet connection (for emoji/fonts)
4. Practice transitions and timing

### Interactive Elements

- Use overview mode (O key) to jump to sections
- Zoom in on code examples
- Pause for questions at marked slides
- Use chalkboard feature (if enabled) to annotate

## Maintenance

### Updating Content

When CuraLit is updated:

1. Update version numbers in presentation
2. Add new features to relevant slides
3. Update command examples if syntax changes
4. Refresh screenshots/examples

### Version Control

Consider committing rendered HTML alongside source for easy access:

```bash
git add docs/presentation.qmd docs/presentation.html docs/custom.css
git commit -m "Update presentation for v0.2.0"
```

## Troubleshooting

### Quarto Not Found

```bash
# Check PATH
which quarto

# Reinstall from quarto.org
```

### Rendering Fails

```bash
# Check YAML syntax
# Ensure all files are in docs/
# Try rendering with verbose output
quarto render presentation.qmd --verbose
```

### Fonts/Emoji Not Displaying

- Requires internet connection for webfonts
- Or use `embed-resources: true` (already set)
- Check browser compatibility

### Styling Not Applied

- Verify `custom.css` exists in same directory
- Check CSS file path in YAML header
- Clear browser cache

## Additional Resources

- **Quarto Documentation:** https://quarto.org/docs/presentations/revealjs/
- **RevealJS Guide:** https://revealjs.com/
- **Markdown Reference:** https://quarto.org/docs/authoring/markdown-basics.html

## License

This presentation is part of the CuraLit project and follows the same MIT license.

---

**Questions?** Open an issue on GitHub or consult the main README.
