use anyhow::{Context, Result};
use pulldown_cmark::{html, Options, Parser};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tera::{Context as TeraContext, Tera};
use walkdir::WalkDir;
use toml;

#[derive(Deserialize, Serialize)]
struct Config {
    source_dir: String,
    output_dir: String,
    template_file: String,
    css_file: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct PageMetadata {
    title: String,
    #[serde(default)]
    description: String,
}

fn parse_markdown_file(path: &Path) -> Result<(Option<PageMetadata>, String)> {
    let content = fs::read_to_string(path).context(format!("Failed to read {}", path.display()))?;
    let mut metadata = None;
    let markdown;

    // Check for YAML front matter
    if content.starts_with("---\n") {
        if let Some(end) = content.find("\n---\n") {
            let yaml = &content[4..end];
            metadata = Some(serde_yaml::from_str(yaml).context("Failed to parse YAML")?);
            markdown = content[end + 5..].to_string();
        } else {
            markdown = content;
        }
    } else {
        markdown = content;
    }

    let mut options = Options::empty();
    // options.insert(Options::ENABLE_STRIKETHROUGH);
    // options.insert(Options::ENABLE_LISTS); // Added for list rendering
    let parser = Parser::new_ext(&markdown, options);
    let mut html_content = String::new();
    html::push_html(&mut html_content, parser);
    Ok((metadata, html_content))
}

fn generate_site(config: &Config) -> Result<()> {
    // Initialize Tera
    let mut tera = Tera::default();
    let template_content = fs::read_to_string(&config.template_file)
        .context(format!("Failed to read template file {}", config.template_file))?;
    tera.add_raw_template("page", &template_content)
        .context("Failed to add template")?;
    
    // Read CSS if provided
    let css_content = config.css_file
        .as_ref()
        .map(|path| {
            eprintln!("Reading CSS file: {}", path);
            fs::read_to_string(path).context(format!("Failed to read CSS file {}", path))
        })
        .transpose()?;

    // Create output directory
    eprintln!("Creating output directory: {}", config.output_dir);
    fs::create_dir_all(&config.output_dir)
        .context(format!("Failed to create output directory {}", config.output_dir))?;

    // Process Markdown files
    for entry in WalkDir::new(&config.source_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("md"))
    {
        let input_path = entry.path();
        eprintln!("Processing Markdown file: {}", input_path.display());
        let relative_path = match input_path.strip_prefix(&config.source_dir) {
            Ok(path) => path,
            Err(e) => {
                eprintln!("Failed to strip prefix for {}: {}", input_path.display(), e);
                continue;
            }
        };
        let output_path = Path::new(&config.output_dir)
            .join(relative_path)
            .with_extension("html");

        // Ensure output directory exists
        if let Some(parent) = output_path.parent() {
            eprintln!("Creating parent directory: {}", parent.display());
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("Failed to create directory {}: {}", parent.display(), e);
                continue;
            }
        }

        // Parse Markdown
        let (metadata, html_content) = match parse_markdown_file(input_path) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Failed to parse {}: {}", input_path.display(), e);
                continue;
            }
        };

        // Render template
        let mut context = TeraContext::new();
        context.insert("content", &html_content);
        context.insert("title", &metadata.as_ref().map(|m| m.title.as_str()).unwrap_or("Untitled"));
        context.insert("description", &metadata.as_ref().map(|m| m.description.as_str()).unwrap_or(""));
        if let Some(css) = &css_content {
            eprintln!("Inserting CSS content: {}", css);
            context.insert("css", css);
        }
        let html_output = match tera.render("page", &context) {
            Ok(output) => output,
            Err(e) => {
                eprintln!("Failed to render template for {}: {}", input_path.display(), e);
                continue;
            }
        };

        // Write output
        eprintln!("Writing output to: {}", output_path.display());
        let mut file = match File::create(&output_path) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Failed to create {}: {}", output_path.display(), e);
                continue;
            }
        };
        if let Err(e) = file.write_all(html_output.as_bytes()) {
            eprintln!("Failed to write {}: {}", output_path.display(), e);
            continue;
        }
    }

    // Copy CSS if provided
    if let Some(css_path) = &config.css_file {
        let css_output = Path::new(&config.output_dir).join("style.css");
        eprintln!("Copying CSS from {} to {}", css_path, css_output.display());
        if let Err(e) = fs::copy(css_path, &css_output) {
            eprintln!("Failed to copy CSS from {} to {}: {}", css_path, css_output.display(), e);
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let config: Config = toml::from_str(&fs::read_to_string("config.toml")?)
        .context("Failed to parse config.toml")?;
    generate_site(&config)?;
    println!("Site generated in {}", config.output_dir);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_test_env() -> Result<(Config, String)> {
        let source_dir = "test_source";
        let output_dir = "test_output";
        eprintln!("Setting up test environment: source={}, output={}", source_dir, output_dir);
        fs::create_dir_all(source_dir).context(format!("Failed to create {}", source_dir))?;
        fs::write(
            format!("{}/test.md", source_dir),
            "---
title: Test Page
description: A test page
---
# Hello
This is **Markdown**."
        )?;
        fs::write(
            "test_template.html",
            "<html><head><title>{{ title }}</title>{% if css %}<style>{{ css | safe }}</style>{% endif %}</head><body>{{ content | safe }}</body></html>"
        )?;
        fs::write("test_style.css", "body { color: blue; }")?;
        let config = Config {
            source_dir: source_dir.to_string(),
            output_dir: output_dir.to_string(),
            template_file: "test_template.html".to_string(),
            css_file: Some("test_style.css".to_string()),
        };
        Ok((config, source_dir.to_string()))
    }

    fn cleanup_test_env(source_dir: &str) {
        eprintln!("Cleaning up: {}", source_dir);
        fs::remove_dir_all(source_dir).unwrap_or(());
        fs::remove_dir_all("test_output").unwrap_or(());
        fs::remove_dir_all("test_source_no_fm").unwrap_or(());
        fs::remove_dir_all("test_output_no_fm").unwrap_or(());
        fs::remove_dir_all("test_source_samples").unwrap_or(());
        fs::remove_dir_all("test_output_samples").unwrap_or(());
        fs::remove_file("test_template.html").unwrap_or(());
        fs::remove_file("test_style.css").unwrap_or(());
        fs::remove_file("test.md").unwrap_or(());
        fs::remove_file("test_malformed.md").unwrap_or(());
    }

    #[test]
    fn test_parse_markdown_file() -> Result<()> {
        fs::write("test.md", "---
title: Test
description: Desc
---
# Hello")?;
        let (metadata, html) = parse_markdown_file(Path::new("test.md"))?;
        assert_eq!(metadata.unwrap().title, "Test");
        assert!(html.contains("<h1>Hello</h1>"));
        fs::remove_file("test.md")?;
        Ok(())
    }


    #[test]
    fn test_malformed_yaml() -> Result<()> {
        fs::write("test_malformed.md", "---
title: Malformed
title: Duplicate  # Duplicate key
---
# Test")?;
        let result = parse_markdown_file(Path::new("test_malformed.md"));
        assert!(result.is_err());
        fs::remove_file("test_malformed.md")?;
        Ok(())
    }
}
