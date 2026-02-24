//! Diff-aware file writer for ArchDoc
//!
//! This module handles writing generated documentation to files while preserving
//! manual content and only updating generated sections.

use crate::errors::ArchDocError;
use std::path::Path;
use std::fs;
use chrono::Utc;

#[derive(Debug)]
pub struct SectionMarker {
    pub name: String,
    pub start_pos: usize,
    pub end_pos: usize,
}

#[derive(Debug)]
pub struct SymbolMarker {
    pub symbol_id: String,
    pub start_pos: usize,
    pub end_pos: usize,
}

pub struct DiffAwareWriter {
    // Configuration
}

impl Default for DiffAwareWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl DiffAwareWriter {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn update_file_with_markers(
        &self,
        file_path: &Path,
        generated_content: &str,
        section_name: &str,
    ) -> Result<(), ArchDocError> {
        // Read existing file
        let existing_content = if file_path.exists() {
            fs::read_to_string(file_path)
                .map_err(ArchDocError::Io)?
        } else {
            // Create new file with template
            let template_content = self.create_template_file(file_path, section_name)?;
            // Write template to file
            fs::write(file_path, &template_content)
                .map_err(ArchDocError::Io)?;
            template_content
        };
        
        // Find section markers
        let markers = self.find_section_markers(&existing_content, section_name)?;
        
        if let Some(marker) = markers.first() {
            // Replace content between markers
            let new_content = self.replace_section_content(
                &existing_content,
                marker,
                generated_content,
            )?;
            
            // Check if content has changed
            let content_changed = existing_content != new_content;
            
            // Write updated content
            if content_changed {
                let updated_content = self.update_timestamp(new_content)?;
                fs::write(file_path, updated_content)
                    .map_err(ArchDocError::Io)?;
            } else {
                // Content hasn't changed, but we might still need to update timestamp
                // TODO: Implement timestamp update logic based on config
                fs::write(file_path, new_content)
                    .map_err(ArchDocError::Io)?;
            }
        }
        
        Ok(())
    }
    
    pub fn update_symbol_section(
        &self,
        file_path: &Path,
        symbol_id: &str,
        generated_content: &str,
    ) -> Result<(), ArchDocError> {
        // Read existing file
        let existing_content = if file_path.exists() {
            fs::read_to_string(file_path)
                .map_err(ArchDocError::Io)?
        } else {
            // If file doesn't exist, create it with a basic template
            let template_content = self.create_template_file(file_path, "symbol")?;
            fs::write(file_path, &template_content)
                .map_err(ArchDocError::Io)?;
            template_content
        };
        
        // Find symbol markers
        let markers = self.find_symbol_markers(&existing_content, symbol_id)?;
        
        if let Some(marker) = markers.first() {
            // Replace content between markers
            let new_content = self.replace_symbol_content(
                &existing_content,
                marker,
                generated_content,
            )?;
            
            // Check if content has changed
            let content_changed = existing_content != new_content;
            
            // Write updated content
            if content_changed {
                let updated_content = self.update_timestamp(new_content)?;
                fs::write(file_path, updated_content)
                    .map_err(ArchDocError::Io)?;
            } else {
                // Content hasn't changed, but we might still need to update timestamp
                // TODO: Implement timestamp update logic based on config
                fs::write(file_path, new_content)
                    .map_err(ArchDocError::Io)?;
            }
        } else {
            eprintln!("Warning: No symbol marker found for {} in {}", symbol_id, file_path.display());
        }
        
        Ok(())
    }
    
    fn find_section_markers(&self, content: &str, section_name: &str) -> Result<Vec<SectionMarker>, ArchDocError> {
        let begin_marker = format!("<!-- ARCHDOC:BEGIN section={} -->", section_name);
        let end_marker = format!("<!-- ARCHDOC:END section={} -->", section_name);
        
        let mut markers = Vec::new();
        let mut pos = 0;
        
        while let Some(begin_pos) = content[pos..].find(&begin_marker) {
            let absolute_begin = pos + begin_pos;
            let search_start = absolute_begin + begin_marker.len();
            
            if let Some(end_pos) = content[search_start..].find(&end_marker) {
                let absolute_end = search_start + end_pos + end_marker.len();
                markers.push(SectionMarker {
                    name: section_name.to_string(),
                    start_pos: absolute_begin,
                    end_pos: absolute_end,
                });
                pos = absolute_end;
            } else {
                break;
            }
        }
        
        Ok(markers)
    }
    
    fn find_symbol_markers(&self, content: &str, symbol_id: &str) -> Result<Vec<SymbolMarker>, ArchDocError> {
        let begin_marker = format!("<!-- ARCHDOC:BEGIN symbol id={} -->", symbol_id);
        let end_marker = format!("<!-- ARCHDOC:END symbol id={} -->", symbol_id);
        
        let mut markers = Vec::new();
        let mut pos = 0;
        
        while let Some(begin_pos) = content[pos..].find(&begin_marker) {
            let absolute_begin = pos + begin_pos;
            let search_start = absolute_begin + begin_marker.len();
            
            if let Some(end_pos) = content[search_start..].find(&end_marker) {
                let absolute_end = search_start + end_pos + end_marker.len();
                markers.push(SymbolMarker {
                    symbol_id: symbol_id.to_string(),
                    start_pos: absolute_begin,
                    end_pos: absolute_end,
                });
                pos = absolute_end;
            } else {
                break;
            }
        }
        
        Ok(markers)
    }
    
    fn replace_section_content(
        &self,
        content: &str,
        marker: &SectionMarker,
        new_content: &str,
    ) -> Result<String, ArchDocError> {
        let before = &content[..marker.start_pos];
        let after = &content[marker.end_pos..];
        
        let begin_marker = format!("<!-- ARCHDOC:BEGIN section={} -->", marker.name);
        let end_marker = format!("<!-- ARCHDOC:END section={} -->", marker.name);
        
        Ok(format!(
            "{}{}{}{}{}",
            before, begin_marker, new_content, end_marker, after
        ))
    }
    
    fn replace_symbol_content(
        &self,
        content: &str,
        marker: &SymbolMarker,
        new_content: &str,
    ) -> Result<String, ArchDocError> {
        let before = &content[..marker.start_pos];
        let after = &content[marker.end_pos..];
        
        let begin_marker = format!("<!-- ARCHDOC:BEGIN symbol id={} -->", marker.symbol_id);
        let end_marker = format!("<!-- ARCHDOC:END symbol id={} -->", marker.symbol_id);
        
        Ok(format!(
            "{}{}{}{}{}",
            before, begin_marker, new_content, end_marker, after
        ))
    }
    
    fn update_timestamp(&self, content: String) -> Result<String, ArchDocError> {
        // Update the "Updated" field in the document metadata section
        // Find the metadata section and update the timestamp
        let today = Utc::now().format("%Y-%m-%d").to_string();
        
        // Look for the "Updated:" line and replace it
        let lines: Vec<&str> = content.lines().collect();
        let mut updated_lines = Vec::new();
        
        for line in lines {
            if line.trim_start().starts_with("- **Updated:**") {
                updated_lines.push(format!("- **Updated:** {}", today));
            } else {
                updated_lines.push(line.to_string());
            }
        }
        
        Ok(updated_lines.join("\n"))
    }
    
    fn create_template_file(&self, _file_path: &Path, template_type: &str) -> Result<String, ArchDocError> {
        // Create file with appropriate template based on type
        match template_type {
            "architecture" => {
                let template = r#"# ARCHITECTURE — <PROJECT_NAME>

<!-- MANUAL:BEGIN -->
## Project summary
**Name:** <PROJECT_NAME>
**Description:** <FILL_MANUALLY: what this project does in 3–7 lines>

## Key decisions (manual)
- <FILL_MANUALLY>

## Non-goals (manual)
- <FILL_MANUALLY>
<!-- MANUAL:END -->

---

## Document metadata
- **Created:** <AUTO_ON_INIT: YYYY-MM-DD>
- **Updated:** <AUTO_ON_CHANGE: YYYY-MM-DD>
- **Generated by:** archdoc (cli) v0.1

---

## Rails / Tooling
<!-- ARCHDOC:BEGIN section=rails -->
> Generated. Do not edit inside this block.
<AUTO: rails summary + links to config files>
<!-- ARCHDOC:END section=rails -->

---

## Repository layout (top-level)
<!-- ARCHDOC:BEGIN section=layout -->
> Generated. Do not edit inside this block.
<AUTO: table of top-level folders + heuristic purpose + link to layout.md>
<!-- ARCHDOC:END section=layout -->

---

## Modules index
<!-- ARCHDOC:BEGIN section=modules_index -->
> Generated. Do not edit inside this block.
<AUTO: table modules + deps counts + links to module docs>
<!-- ARCHDOC:END section=modules_index -->

---

## Critical dependency points
<!-- ARCHDOC:BEGIN section=critical_points -->
> Generated. Do not edit inside this block.
<AUTO: top fan-in/out symbols + cycles>
<!-- ARCHDOC:END section=critical_points -->

---

<!-- MANUAL:BEGIN -->
## Change notes (manual)
- <FILL_MANUALLY>
<!-- MANUAL:END -->
"#;
                Ok(template.to_string())
            }
            "symbol" => {
                // Template for symbol documentation files
                let template = r#"# File: <relative_path>

- **Module:** <AUTO: module_id>
- **Defined symbols:** <AUTO>
- **Imports:** <AUTO>

<!-- MANUAL:BEGIN -->
## File intent (manual)
<FILL_MANUALLY>
<!-- MANUAL:END -->

---

## Imports & file-level dependencies
<!-- ARCHDOC:BEGIN section=file_imports -->
> Generated. Do not edit inside this block.
<AUTO: imports list + outbound modules + inbound files>
<!-- ARCHDOC:END section=file_imports -->

---

## Symbols index
<!-- ARCHDOC:BEGIN section=symbols_index -->
> Generated. Do not edit inside this block.
<AUTO: list of links to symbol anchors>
<!-- ARCHDOC:END section=symbols_index -->

---

## Symbol details
<!-- AUTOGENERATED SYMBOL CONTENT WILL BE INSERTED HERE -->
"#;
                Ok(template.to_string())
            }
            _ => {
                Ok("".to_string())
            }
        }
    }
}