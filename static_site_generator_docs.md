# Static Site Generator

## Overview
A static site generator built in Rust that converts Markdown files with YAML front matter into HTML pages using templates. The generator supports CSS styling, configurable output directories, and comprehensive error handling.

## Features
- Markdown to HTML conversion using `pulldown-cmark`
- YAML front matter parsing for page metadata
- Template rendering with `tera` templating engine
- CSS integration and copying
- Configurable source and output directories
- Recursive directory processing
- Comprehensive error handling and logging

## Dependencies
```toml
[dependencies]
anyhow = "1.0"
pulldown-cmark = "0.9"
serde = { version = "1.0", features = ["derive"] }
tera = "1.19"
walkdir = "2.3"
serde_yaml = "0.9"
toml = "0.8"
```

## Configuration

### Config File (`config.toml`)
```toml
source_dir = "content"
output_dir = "dist"
template_file = "template.html"
css_file = "style.css"  # Optional
```

### Configuration Structure
```rust
#[derive(Deserialize, Serialize)]
struct Config {
    source_dir: String,
    output_dir: String,
    template_file: String,
    css_file: Option<String>,
}
```

## Content Format

### Markdown Files with Front Matter
```markdown
---
title: Page Title
description: Page description for meta tags
---

# Main Content

This is **Markdown** content that will be converted to HTML.

## Features
- List items
- *Italic text*
- **Bold text**
```

### Metadata Structure
```rust
#[derive(Deserialize, Serialize)]
struct PageMetadata {
    title: String,
    #[serde(default)]
    description: String,
}
```

## Template System

### Template Variables
- `{{ title }}` - Page title from front matter or "Untitled"
- `{{ description }}` - Page description from front matter
- `{{ content | safe }}` - Rendered HTML content
- `{{ css | safe }}` - CSS content (if provided)

### Example Template
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>{{ title }}</title>
    <meta name="description" content="{{ description }}">
    {% if css %}
    <style>{{ css | safe }}</style>
    {% endif %}
</head>
<body>
    {{ content | safe }}
</body>
</html>
```

## Processing Pipeline

### 1. Initialization
- Read and parse `config.toml`
- Initialize Tera templating engine
- Load template file
- Read CSS file (if specified)

### 2. File Discovery
- Recursively scan source directory
- Filter for `.md` files
- Preserve directory structure

### 3. Markdown Processing
```rust
fn parse_markdown_file(path: &Path) -> Result<(Option<PageMetadata>, String)>
```
- Extract YAML front matter
- Parse metadata with serde_yaml
- Convert Markdown to HTML with pulldown-cmark

### 4. Template Rendering
- Create Tera context with page data
- Insert title, description, content, and CSS
- Render HTML output

### 5. File Output
- Create output directory structure
- Write HTML files with `.html` extension
- Copy CSS file to output directory

## Usage

### Basic Usage
```bash
cargo run
```

### Directory Structure
```
project/
├── config.toml          # Configuration file
├── template.html        # HTML template
├── style.css           # Optional CSS file
├── content/            # Source markdown files
│   ├── index.md
│   ├── about.md
│   └── posts/
│       └── first-post.md
└── dist/               # Generated output
    ├── index.html
    ├── about.html
    ├── style.css
    └── posts/
        └── first-post.html
```

## Error Handling

### File Processing Errors
- Missing or unreadable files logged and skipped
- Invalid YAML front matter handled gracefully
- Template rendering errors reported per file
- Directory creation errors handled

### YAML Parsing
```rust
// Handles malformed YAML
if let Some(end) = content.find("\n---\n") {
    let yaml = &content[4..end];
    metadata = Some(serde_yaml::from_str(yaml).context("Failed to parse YAML")?);
}
```

### Logging
- Processing status for each file
- Error messages with file paths
- Directory creation notifications
- CSS copying status

## Testing

### Test Environment Setup
```rust
fn setup_test_env() -> Result<(Config, String)> {
    // Creates temporary test directories
    // Sets up sample files
    // Returns test configuration
}
```

### Test Coverage
1. **Markdown Parsing**: Tests front matter extraction and HTML conversion
2. **YAML Validation**: Tests malformed YAML handling
3. **File Processing**: Tests complete generation pipeline
4. **Template Rendering**: Tests template variable substitution

### Running Tests
```bash
cargo test
```

## File Structure
```
src/
└── main.rs              # Complete implementation

content/                 # Example content
├── about.md
├── example.md
├── malformed-yaml.md
└── no-front-matter.md

config.toml             # Configuration
template.html           # HTML template
style.css              # Stylesheet
```

## Advanced Features

### Markdown Extensions
Currently uses basic Markdown. Can be extended with:
```rust
let mut options = Options::empty();
options.insert(Options::ENABLE_STRIKETHROUGH);
options.insert(Options::ENABLE_TABLES);
options.insert(Options::ENABLE_TASKLISTS);
```

### CSS Integration
- CSS file automatically copied to output directory
- CSS content embedded in template if specified
- Supports external CSS file references

### Directory Structure Preservation
- Maintains source directory hierarchy in output
- Creates nested directories as needed
- Handles deep directory structures

## Error Recovery

### Graceful Degradation
- Individual file errors don't stop processing
- Missing metadata uses defaults
- Malformed files skipped with error logging

### Default Values
- Title: "Untitled" for missing front matter
- Description: Empty string for missing descriptions
- CSS: Optional, skipped if not provided

## Performance Considerations

### Optimization Opportunities
1. **Parallel Processing**: Process files concurrently
2. **Incremental Builds**: Only process changed files
3. **Template Caching**: Cache compiled templates
4. **Memory Optimization**: Stream large files

### Current Limitations
- Sequential file processing
- Full regeneration on each run
- Memory-based template compilation

## Extension Ideas

### Content Management
1. **Asset Copying**: Copy images and other assets
2. **Collection Support**: Generate index pages for collections
3. **Pagination**: Split large collections into pages
4. **Taxonomy**: Tag and category support

### Template Enhancements
1. **Partial Templates**: Include/extend template support
2. **Helper Functions**: Custom template functions
3. **Conditional Rendering**: Advanced template logic
4. **Theme Support**: Multiple template themes

### Build Features
1. **Watch Mode**: Automatic rebuilding on file changes
2. **Development Server**: Local preview server
3. **Live Reload**: Browser refresh on changes
4. **Build Optimization**: Minification and optimization

### Content Features
1. **Syntax Highlighting**: Code block highlighting
2. **Math Rendering**: LaTeX math support
3. **Diagram Support**: Mermaid diagrams
4. **Table of Contents**: Automatic TOC generation