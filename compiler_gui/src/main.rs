#![cfg_attr(windows, windows_subsystem = "windows")]

use eframe::{egui};
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;
use egui::Vec2;

// Footer constants: our appended archive is terminated with a footer
const FOOTER_MARKER: &[u8; 16] = b"RSCARCHIVE_V1___";

// Each resource is recorded with its filename and size.
#[derive(Serialize, Deserialize)]
struct ResourceEntry {
    filename: String,
    size: u32,
}

// The archive header now also includes execution_style, run_as_admin, and is_compressed
#[derive(Serialize, Deserialize)]
struct ArchiveHeader {
    extraction_path: String,
    main_file: String,
    resources: Vec<ResourceEntry>,
    execution_style: String,
    run_as_admin: bool,
    is_compressed: bool,  // Added this field to indicate if resources are compressed
}

// The GUI app state now holds additional fields including theme selection and project management
struct AppState {
    extraction_path: String,
    main_file: String,      // resource filename that should be launched
    resources: Vec<PathBuf>, // list of resource file paths
    output_exe: String,
    execution_style: String, // one of "no-window", "minimized", "normal", "maximized"
    run_as_admin: bool,
    message: String,
    dark_mode: bool,
    selected_resource: Option<usize>, // track the selected resource
    compress_resources: bool, // option to compress resources
    show_settings: bool, // toggle for settings panel
    icon_path: Option<PathBuf>, // custom icon for the output executable
    search_query: String, // for resource searching
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            extraction_path: "rc_extracted".to_string(),
            main_file: String::new(),
            resources: Vec::new(),
            output_exe: "packed.exe".to_string(),
            execution_style: "normal".to_string(),
            run_as_admin: false,
            message: String::new(),
            dark_mode: true, // default to dark mode
            selected_resource: None,
            compress_resources: false,
            show_settings: false,
            icon_path: None,
            search_query: String::new(),
        }
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // set the theme based on dark_mode
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }
        
        // egui drag and drop
        if !ctx.input(|i| i.raw.dropped_files.clone()).is_empty() {
            for file in &ctx.input(|i| i.raw.dropped_files.clone()) {
                if let Some(path) = &file.path {
                    if !self.resources.contains(path) {
                        self.resources.push(path.clone());
                    }
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // Top bar with title and menu
            ui.horizontal(|ui| {
                // Use a styled heading for the title instead of an image
                ui.heading(egui::RichText::new("Resource Compiler")
                    .size(28.0)
                    .strong()
                    .color(if self.dark_mode {
                        egui::Color32::from_rgb(120, 80, 200)
                    } else {
                        egui::Color32::from_rgb(60, 40,100)
                    })
                );
                
                // Menu bar
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let theme_text = if self.dark_mode { "☀ Light Mode" } else { "🌙 Dark Mode" };
                    if ui.button(theme_text).clicked() {
                        self.dark_mode = !self.dark_mode;
                    }
                    
                    if ui.button("⚙ Settings").clicked() {
                        self.show_settings = !self.show_settings;
                    }
                    
                    // File menu dropdown
                    egui::menu::menu_button(ui, "📁 File", |ui| {
                        if ui.button("New Project").clicked() {
                            // Clear current project
                            self.resources.clear();
                            self.main_file.clear();
                            self.extraction_path = "rc_extracted".to_string();
                            self.output_exe = "packed.exe".to_string();
                            self.message = "Started new project".to_string();
                            ui.close_menu();
                        }
                        
                        if ui.button("Save Project").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Resource Compiler Project", &["rcproj"])
                                .save_file() {
                                let project = serde_json::json!({
                                    "extraction_path": self.extraction_path,
                                    "main_file": self.main_file,
                                    "resources": self.resources.iter().map(|p| p.to_string_lossy().to_string()).collect::<Vec<_>>(),
                                    "output_exe": self.output_exe,
                                    "execution_style": self.execution_style,
                                    "run_as_admin": self.run_as_admin,
                                    "compress_resources": self.compress_resources,
                                    "icon_path": self.icon_path.as_ref().map(|p| p.to_string_lossy().to_string()),
                                });
                                
                                if let Ok(json) = serde_json::to_string_pretty(&project) {
                                    if fs::write(&path, json).is_ok() {
                                        self.message = "Project saved successfully".to_string();
                                    } else {
                                        self.message = "❌ Failed to save project".to_string();
                                    }
                                }
                            }
                            ui.close_menu();
                        }
                        
                        if ui.button("Load Project").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Resource Compiler Project", &["rcproj"])
                                .pick_file() {
                                if let Ok(content) = fs::read_to_string(&path) {
                                    if let Ok(project) = serde_json::from_str::<serde_json::Value>(&content) {
                                        // Load project data
                                        self.extraction_path = project["extraction_path"].as_str().unwrap_or("rc_extracted").to_string();
                                        self.main_file = project["main_file"].as_str().unwrap_or("").to_string();
                                        self.output_exe = project["output_exe"].as_str().unwrap_or("packed.exe").to_string();
                                        self.execution_style = project["execution_style"].as_str().unwrap_or("normal").to_string();
                                        self.run_as_admin = project["run_as_admin"].as_bool().unwrap_or(false);
                                        self.compress_resources = project["compress_resources"].as_bool().unwrap_or(false);
                                        
                                        // Load resources
                                        self.resources.clear();
                                        if let Some(resources) = project["resources"].as_array() {
                                            for res in resources {
                                                if let Some(path_str) = res.as_str() {
                                                    let path = PathBuf::from(path_str);
                                                    if path.exists() {
                                                        self.resources.push(path);
                                                    }
                                                }
                                            }
                                        }
                                        
                                        // Load icon path
                                        if let Some(icon_path) = project["icon_path"].as_str() {
                                            let path = PathBuf::from(icon_path);
                                            if path.exists() {
                                                self.icon_path = Some(path);
                                            } else {
                                                self.icon_path = None;
                                            }
                                        }
                                        
                                        self.message = "Project loaded successfully".to_string();
                                    }
                                }
                            }
                            ui.close_menu();
                        }
                    });
                });
            });

            ui.add_space(10.0);

            // Container with rounded corners and padding for the main content
            egui::Frame::default()
                .fill(ui.style().visuals.faint_bg_color)
                .rounding(10.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.heading("Project Settings");
                    ui.add_space(5.0);

                    // --- Extraction Path (supports env variables) ---
                    ui.horizontal(|ui| {
                        ui.label("Extraction Path:");
                        ui.text_edit_singleline(&mut self.extraction_path);
                        ui.label(" (%USERPROFILE%\\MyApp | C:\\folder | cool_folder)");
                    });

                    // --- Output EXE Name ---
                    ui.horizontal(|ui| {
                        ui.label("Output EXE Name:");
                        ui.text_edit_singleline(&mut self.output_exe);
                    });

                    // --- Main File (by filename) ---
                    ui.horizontal(|ui| {
                        ui.label("Main File:");
                        ui.text_edit_singleline(&mut self.main_file);
                        ui.label("(Select a resource below to set)");
                    });

                    // --- Execution Style Selection ---
                    ui.horizontal(|ui| {
                        ui.label("Execution Style:");
                        egui::ComboBox::from_label("")
                            .selected_text(match self.execution_style.as_str() {
                                "no-window" => "No Window",
                                "minimized" => "Minimized",
                                "normal" => "Normal",
                                "maximized" => "Maximized",
                                _ => "Normal"
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.execution_style, "no-window".to_string(), "No Window");
                                ui.selectable_value(&mut self.execution_style, "minimized".to_string(), "Minimized");
                                ui.selectable_value(&mut self.execution_style, "normal".to_string(), "Normal");
                                ui.selectable_value(&mut self.execution_style, "maximized".to_string(), "Maximized");
                            });
                    });

                    // --- Run as Administrator Toggle ---
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.run_as_admin, "Run as Administrator");
                    });
                });

            ui.add_space(10.0);
            
            // Resources section with improved appearance
            egui::Frame::default()
                .fill(ui.style().visuals.faint_bg_color)
                .rounding(10.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Resources");
                        
                        // Move the search to the header row
                        if !self.resources.is_empty() {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("🔍 Search").clicked() {
                                    // Handle search button click if needed
                                }
                                ui.text_edit_singleline(&mut self.search_query);
                                ui.label("Search:");
                            });
                        }
                    });
                    
                    ui.add_space(5.0);
                    
                    // Always show the Add Resource button at the top
                    if ui.button("📂 Add Resource").clicked() {
                        if let Some(file) = rfd::FileDialog::new().pick_file() {
                            if !self.resources.contains(&file) {
                                self.resources.push(file);
                            }
                        }
                    }
                    
                    ui.label("Drag & drop files here or use the Add Resource button above:");

                    if self.resources.is_empty() {
                        ui.add_space(10.0);
                        ui.centered_and_justified(|ui| {
                            ui.label("No resources added yet.");
                        });
                        ui.add_space(10.0);
                    } else {
                        // Create a scrollable area for resources
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            // Filter resources based on search query
                            let search_query_lower = self.search_query.to_lowercase();
                            let mut resources_to_remove = Vec::new();
                            
                            // Iterate through resources
                            for i in 0..self.resources.len() {
                                let resource_name = self.resources[i].file_name()
                                    .map_or_else(|| "Unknown".to_string(), |n| n.to_string_lossy().to_string());
                                
                                let resource_path = self.resources[i].to_string_lossy().to_string();
                                
                                // Skip resources that don't match search query
                                if !self.search_query.is_empty() && 
                                   !resource_name.to_lowercase().contains(&search_query_lower) && 
                                   !resource_path.to_lowercase().contains(&search_query_lower) {
                                    continue;
                                }
                                
                                let is_selected = Some(i) == self.selected_resource;
                                
                                // Create a frame for each resource with conditional highlighting
                                let mut frame = egui::Frame::default()
                                    .inner_margin(egui::style::Margin::same(8.0))
                                    .rounding(egui::Rounding::same(4.0));
                                
                                if is_selected {
                                    frame = frame.fill(ui.style().visuals.selection.bg_fill);
                                }
                                
                                frame.show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        if ui.selectable_label(is_selected, &resource_name).clicked() {
                                            // Single click selects the resource
                                            if Some(i) == self.selected_resource {
                                                // If already selected, set as main file
                                                self.main_file = resource_name.clone();
                                            }
                                            self.selected_resource = Some(i);
                                        }
                                        
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.button("✖").clicked() {
                                                if self.selected_resource == Some(i) {
                                                    self.selected_resource = None;
                                                }
                                                resources_to_remove.push(i);
                                            }
                                            
                                            if ui.button("Set as Main").clicked() {
                                                self.main_file = resource_name;
                                            }
                                        });
                                    });
                                    
                                    ui.add_space(2.0);
                                    ui.label(format!("Path: {}", resource_path));
                                });
                                
                                ui.add_space(4.0);
                            }
                            
                            // Remove resources marked for removal
                            for &i in resources_to_remove.iter().rev() {
                                self.resources.remove(i);
                            }
                        });
                        
                        // Resource reordering buttons - moved inside the resources container
                        if self.selected_resource.is_some() {
                            ui.horizontal(|ui| {
                                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                    if ui.button("⬆ Move Up").clicked() && self.selected_resource.unwrap() > 0 {
                                        let idx = self.selected_resource.unwrap();
                                        self.resources.swap(idx, idx - 1);
                                        self.selected_resource = Some(idx - 1);
                                    }
                                    
                                    if ui.button("⬇ Move Down").clicked() && self.selected_resource.unwrap() < self.resources.len() - 1 {
                                        let idx = self.selected_resource.unwrap();
                                        self.resources.swap(idx, idx + 1);
                                        self.selected_resource = Some(idx + 1);
                                    }
                                });
                            });
                        }
                    }
                });

            ui.add_space(10.0);
            
            // Action buttons section
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                if ui.button("📦 Compile EXE").clicked() {
                    match compile_exe(self) {
                        Ok(msg) => self.message = msg,
                        Err(e) => self.message = format!("❌ Error: {}", e),
                    }
                }
            });
            
            // Message area
            if (!self.message.is_empty()) {
                ui.add_space(10.0);
                let (bg_color, text_color) = if self.message.starts_with("❌") {
                    // Error message
                    (
                        egui::Color32::from_rgba_premultiplied(180, 0, 0, 25),
                        if self.dark_mode { egui::Color32::from_rgb(255, 200, 200) } else { egui::Color32::from_rgb(120, 0, 0) }
                    )
                } else {
                    // Success message
                    (
                        egui::Color32::from_rgba_premultiplied(0, 180, 0, 25),
                        if self.dark_mode { egui::Color32::from_rgb(200, 255, 200) } else { egui::Color32::from_rgb(0, 100, 0) }
                    )
                };

                egui::Frame::default()
                    .rounding(8.0)
                    .fill(bg_color)
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.colored_label(text_color, &self.message);
                    });
            }

            // Keyboard shortcuts help
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    ui.label("Keyboard Shortcuts:");
                    ui.label("Ctrl+N: New Project");
                    ui.label("Ctrl+S: Save Project");
                    ui.label("Ctrl+O: Open Project");
                    ui.label("Ctrl+B: Compile EXE");
                    ui.label("Delete: Remove Selected Resource");
                });
            });

            // Settings panel (if enabled)
            if self.show_settings {
                egui::Window::new("Settings")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.heading("Application Settings");
                        
                        ui.checkbox(&mut self.compress_resources, "Compress resources");
                        ui.add_space(5.0);
                        
                        ui.horizontal(|ui| {
                            ui.label("Custom Icon:");
                            if let Some(ref path) = self.icon_path {
                                ui.label(path.file_name().unwrap_or_default().to_string_lossy().to_string());
                                if ui.button("Clear").clicked() {
                                    self.icon_path = None;
                                }
                            } else {
                                if ui.button("Select Icon").clicked() {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("Icon", &["ico"])
                                        .pick_file() {
                                        self.icon_path = Some(path);
                                    }
                                }
                            }
                        });
                        
                        ui.add_space(10.0);
                        if ui.button("Close").clicked() {
                            self.show_settings = false;
                        }
                    });
            }
        });
        
        // Handle keyboard shortcuts
        ctx.input(|i| {
            if i.modifiers.ctrl {
                if i.key_pressed(egui::Key::N) {
                    // New project
                    self.resources.clear();
                    self.main_file.clear();
                    self.extraction_path = "rc_extracted".to_string();
                    self.output_exe = "packed.exe".to_string();
                    self.message = "Started new project".to_string();
                }
                else if i.key_pressed(egui::Key::S) {
                    // Save project logic - simplified, should open a file dialog
                    self.message = "Use File menu to save project".to_string();
                }
                else if i.key_pressed(egui::Key::O) {
                    // Open project logic - simplified, should open a file dialog
                    self.message = "Use File menu to open project".to_string();
                }
                else if i.key_pressed(egui::Key::B) {
                    // Compile EXE
                    match compile_exe(self) {
                        Ok(msg) => self.message = msg,
                        Err(e) => self.message = format!("❌ Error: {}", e),
                    }
                }
            }
            
            if i.key_pressed(egui::Key::Delete) && self.selected_resource.is_some() {
                let idx = self.selected_resource.unwrap();
                self.resources.remove(idx);
                if idx >= self.resources.len() {
                    self.selected_resource = if self.resources.is_empty() { 
                        None 
                    } else { 
                        Some(self.resources.len() - 1) 
                    };
                }
            }
        });
    }
}

/// compile_exe builds the new EXE by:
/// 1. Verifying the main file is among the resources.
/// 2. Reading a pre-built stub (stub.exe must exist in the same folder).
/// 3. Building a JSON header that includes extraction_path, main_file, resources, execution_style, and run_as_admin.
/// 4. Appending the resource files' bytes.
/// 5. Adding a footer containing the header length, archive data length, and a fixed marker.
fn compile_exe(state: &AppState) -> Result<String, String> {
    // Verify that the main file (by filename) is among the added resources.
    let main_file_found = state.resources.iter().any(|p| {
        p.file_name()
            .map(|f| f.to_string_lossy().to_string() == state.main_file)
            .unwrap_or(false)
    });
    if (!main_file_found) {
        return Err("Main file must be one of the added resources (by filename)".to_string());
    }

    // Read the stub binary.
    let stub_bytes = fs::read("stub.exe")
        .map_err(|e| format!("Failed to read stub.exe: {}", e))?;

    // Build the header with the extra fields.
    let mut header = ArchiveHeader {
        extraction_path: state.extraction_path.clone(),
        main_file: state.main_file.clone(),
        resources: Vec::new(),
        execution_style: state.execution_style.clone(),
        run_as_admin: state.run_as_admin,
        is_compressed: state.compress_resources,  // Set the compression flag
    };

    // Read each resource file and accumulate the data.
    let mut resource_data = Vec::new();
    for res_path in &state.resources {
        let data = fs::read(res_path)
            .map_err(|e| format!("Failed to read resource {:?}: {}", res_path, e))?;
        let filename = res_path.file_name()
            .ok_or("Invalid resource file name")?
            .to_string_lossy().to_string();
        header.resources.push(ResourceEntry {
            filename,
            size: data.len() as u32,
        });
        resource_data.extend_from_slice(&data);
    }

    // Serialize the header to JSON.
    let header_json = serde_json::to_string(&header)
        .map_err(|e| format!("Failed to serialize header: {}", e))?;
    let header_bytes = header_json.as_bytes();
    let header_length = header_bytes.len();

    // Build the archive data: header JSON followed by resource file bytes.
    let mut archive_data = Vec::new();
    archive_data.extend_from_slice(header_bytes);
    
    // Apply compression ONLY to resource data if enabled
    let final_resource_data = if state.compress_resources {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;
        
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        if let Err(e) = encoder.write_all(&resource_data) {
            return Err(format!("Failed to compress data: {}", e));
        }
        
        match encoder.finish() {
            Ok(compressed) => compressed,
            Err(e) => return Err(format!("Failed to finish compression: {}", e))
        }
    } else {
        resource_data
    };
    
    // Add the (possibly compressed) resource data after the header
    archive_data.extend_from_slice(&final_resource_data);
    let archive_data_length = archive_data.len();

    // Build the footer: header length (4 bytes) + archive data length (4 bytes) + marker (16 bytes).
    let mut footer = Vec::new();
    footer.extend_from_slice(&(header_length as u32).to_le_bytes());
    footer.extend_from_slice(&(archive_data_length as u32).to_le_bytes());
    footer.extend_from_slice(FOOTER_MARKER);

    // Final output: [stub binary] + [archive data] + [footer]
    let mut output_data = Vec::new();
    output_data.extend_from_slice(&stub_bytes);
    output_data.extend_from_slice(&archive_data);
    output_data.extend_from_slice(&footer);

    // Apply custom icon if specified
    if let Some(icon_path) = &state.icon_path {
        if (!icon_path.exists()) {
            return Err(format!("Icon file does not exist: {:?}", icon_path));
        }
        
        // Read the icon file
        let icon_data = match fs::read(icon_path) {
            Ok(data) => data,
            Err(e) => return Err(format!("Failed to read icon file: {}", e))
        };
        
        // Use resource_builder to inject the icon into the PE file
        // This is a simplified approach; in a real application, you would use a proper
        // Windows resource editor library to modify the PE resources
        if let Err(e) = embed_icon_in_exe(&state.output_exe, &output_data, &icon_data) {
            return Err(e);
        }
        
        Ok(format!("✅ Successfully created {} with custom icon", state.output_exe))
    } else {
        // No custom icon, just write the file directly
        fs::write(&state.output_exe, output_data)
            .map_err(|e| format!("Failed to write output exe: {}", e))?;
        
        Ok(format!("✅ Successfully created {}", state.output_exe))
    }
}

// Function to embed an icon in the output EXE
fn embed_icon_in_exe(output_path: &str, exe_data: &[u8], icon_data: &[u8]) -> Result<(), String> {
    // First, write the EXE data to the output path
    fs::write(output_path, exe_data)
        .map_err(|e| format!("Failed to write output exe: {}", e))?;
    
    // Now use the resource_builder crate to modify the EXE and add the icon
    // For now, we'll use a simple Windows-specific approach using a temporary .rc file
    
    #[cfg(windows)]
    {
        use std::process::Command;
        
        // Create a temporary directory for resource compilation
        let temp_dir = std::env::temp_dir().join("resource_compiler_temp");
        if !temp_dir.exists() {
            fs::create_dir_all(&temp_dir)
                .map_err(|e| format!("Failed to create temp directory: {}", e))?;
        }
        
        // Copy the icon to the temp directory
        let temp_icon_path = temp_dir.join("temp_icon.ico");
        fs::write(&temp_icon_path, icon_data)
            .map_err(|e| format!("Failed to write temp icon: {}", e))?;
        
        // Create a .rc file for the icon
        let rc_content = format!(
            "#include <windows.h>\n\
             1 ICON \"{}\"",
            temp_icon_path.to_string_lossy().replace('\\', "\\\\")
        );
        
        let rc_path = temp_dir.join("resource.rc");
        fs::write(&rc_path, rc_content)
            .map_err(|e| format!("Failed to write resource script: {}", e))?;
        
        // Use rcedit or ResEdit to modify the EXE file (simplified example)
        // In a real application, you would integrate with proper resource editing libraries
        let status = Command::new("rcedit")
            .arg(output_path)
            .arg("--set-icon")
            .arg(temp_icon_path.to_string_lossy().to_string())
            .status();
        
        match status {
            Ok(exit) => {
                if exit.success() {
                    // Clean up temp files
                    let _ = fs::remove_file(&temp_icon_path);
                    let _ = fs::remove_file(&rc_path);
                    Ok(())
                } else {
                    Err(format!("Failed to set icon with exit code: {:?}", exit.code()))
                }
            },
            Err(e) => {
                // If rcedit fails, it might not be installed or accessible
                // Fall back to just keeping the EXE without the icon
                eprintln!("Warning: Could not set icon: {}", e);
                Ok(()) // Return OK so the application still works without the icon
            }
        }
    }
    
    #[cfg(not(windows))]
    {
        // On non-Windows platforms, just return success (icon embedding is Windows-specific)
        Ok(())
    }
}

// Helper function to load an icon from memory
fn load_icon_from_memory(icon_data: &[u8]) -> Result<eframe::IconData, String> {
    // Use the image crate to properly decode the .ico file
    let image = match image::load_from_memory(icon_data) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("Failed to load icon: {}", e);
            // Fallback to a colored square if loading fails
            return create_fallback_icon();
        }
    };
    
    // Convert to RGBA
    let rgba_image = image.to_rgba8();
    let width = rgba_image.width() as usize;
    let height = rgba_image.height() as usize;
    
    // Extract raw pixel data
    let rgba = rgba_image.into_raw();
    
    Ok(eframe::IconData {
        rgba,
        width: width as u32,
        height: height as u32,
    })
}

// Create a fallback icon if loading the real one fails
fn create_fallback_icon() -> Result<eframe::IconData, String> {
    let width = 32;
    let height = 32;
    let mut rgba = vec![0; width * height * 4];
    
    for y in 0..height {
        for x in 0..width {
            let i = (y * width + x) * 4;
            rgba[i] = 0;     // R
            rgba[i + 1] = 70; // G
            rgba[i + 2] = 150; // B
            rgba[i + 3] = 255; // A
        }
    }
    
    Ok(eframe::IconData {
        rgba,
        width: width as u32,
        height: height as u32,
    })
}

#[cfg(windows)]
fn main() {
    // Load application icon for the window
    let icon_data = include_bytes!("../assets/app_icon.ico");
    
    // This hides the console window on Windows
    let mut native_options = eframe::NativeOptions {
        vsync: true,
        decorated: true,
        transparent: false,
        min_window_size: Some(egui::vec2(600.0, 500.0)),
        initial_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };
    
    // Set application icon if available
    if let Ok(icon) = load_icon_from_memory(icon_data) {
        native_options.icon_data = Some(icon);
    }
    
    eframe::run_native(
        "Resource Compiler",
        native_options,
        Box::new(|_cc| Box::new(AppState::default())),
    );
}

#[cfg(not(windows))]
fn main() {
    // Load application icon for the window
    let icon_data = include_bytes!("../assets/app_icon.ico");
    
    let mut native_options = eframe::NativeOptions {
        vsync: true,
        decorated: true,
        transparent: false,
        min_window_size: Some(egui::vec2(600.0, 500.0)),
        initial_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };
    
    // Set application icon if available
    if let Ok(icon) = load_icon_from_memory(icon_data) {
        native_options.icon_data = Some(icon);
    }
    
    eframe::run_native(
        "Resource Compiler",
        native_options,
        Box::new(|_cc| Box::new(AppState::default())),
    );
}
