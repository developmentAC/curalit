# CuraLit Poster

This directory contains a professional academic poster about CuraLit, perfect for conferences, lab displays, or printed materials.

## Contents

- **gpbio2026_curalit_poster.qmd** - Main Quarto poster source (Typst format)
- **README.md** - This file

## Poster Formats

### Option 1: Typst Poster (Professional Print) - REQUIRES EXTENSION

The main poster uses Quarto's Typst format for high-quality PDF output suitable for professional printing.

**Note**: This requires the Quarto poster extension which is not yet officially released. For now, use the HTML version (Option 2) which works perfectly.

**Size**: 36" × 24" (standard conference poster)

**Rendering** (when extension available):
```bash
cd poster
quarto render gpbio2026_curalit_poster.qmd
```

**Output**: `gpbio2026_curalit_poster.pdf`

**Alternative PDF**: Export the HTML version to PDF via browser (File → Print → Save as PDF)

### Option 2: HTML Poster (Web Display) - RECOMMENDED

The HTML version is fully functional and ready to use. Great for web display, printing via browser, or quick previewing.

**Rendering**:
```bash
cd poster
quarto render gpbio2026_curalit_poster.qmd
```

**Output**: `gpbio2026_curalit_poster.pdf`

### Preview Changes

```bash
# Live preview (HTML version recommended for quick iteration)
quarto preview poster-html.qmd
```

## License

This poster is part of the CuraLit project and follows the same MIT license.

## Questions?

- Open an issue on GitHub
- Check main project README
- Email: obonhamcarter at allegheny.edu

---
