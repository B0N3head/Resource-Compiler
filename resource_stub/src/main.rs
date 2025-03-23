use std::env;
use std::fs;
use std::io::{Read, Seek, SeekFrom, Cursor};
use std::path::Path;
use serde::{Deserialize, Serialize};
use flate2::read::GzDecoder;

// Windows API items
use windows::Win32::Foundation::{HANDLE, CloseHandle, HWND};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::{
    SW_HIDE, SW_SHOWMINIMIZED, SW_SHOWNORMAL, SW_SHOWMAXIMIZED,
    MessageBoxW, MB_OK,
};
use windows::core::PCWSTR;
use windows::Win32::Security::{TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation};
use windows::Win32::System::Threading::{OpenProcessToken, GetCurrentProcess};
use windows::Win32::Security::GetTokenInformation;
use std::ptr::null_mut;
use std::mem::size_of;
use windows::core::w;

// Archive footer format (total 24 bytes):
//   - 4 bytes: header length (u32, little-endian)
//   - 4 bytes: total archive data length (u32, little-endian)
//   - 16 bytes: fixed marker (must equal FOOTER_MARKER)
const FOOTER_SIZE: usize = 4 + 4 + 16;
const FOOTER_MARKER: &[u8; 16] = b"RSCARCHIVE_V1___";

// Structures matching the header created by the packer
#[derive(Serialize, Deserialize)]
struct ResourceEntry {
    filename: String,
    size: u32,
}

#[derive(Serialize, Deserialize)]
struct ArchiveHeader {
    extraction_path: String,
    main_file: String,
    resources: Vec<ResourceEntry>,
    execution_style: String, // "no-window", "minimized", "normal", or "maximized"
    run_as_admin: bool,
    is_compressed: bool,  // Add this field to match the GUI program
}

fn is_elevated() -> Result<bool, windows::core::Error> {
    let mut token_handle: HANDLE = HANDLE(null_mut());
    let result = unsafe {
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle)
    };
    
    if result.is_err() {
        return Err(result.err().unwrap());
    }

    let mut elevation = TOKEN_ELEVATION::default();
    let elevation_ptr: *mut TOKEN_ELEVATION = &mut elevation;
    let mut return_length: u32 = 0;

    let result = unsafe {
        GetTokenInformation(
            token_handle,
            TokenElevation,
            Some(elevation_ptr as *mut _),
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut return_length,
        )
    };
    
    unsafe { CloseHandle(token_handle); }
    
    if result.is_err() {
        return Err(result.err().unwrap());
    }

    // TOKEN_ELEVATION.TokenIsElevated is a u32, 0 is false, non-zero is true
    Ok(elevation.TokenIsElevated != 0)
}

fn show_message_box(message: &str) {
    use std::ffi::OsStr;
    use std::iter;
    use std::os::windows::ffi::OsStrExt;

    let wide_message: Vec<u16> = OsStr::new(message)
        .encode_wide()
        .chain(iter::once(0))
        .collect();

    unsafe {
        MessageBoxW(
            None,
            PCWSTR(wide_message.as_ptr()),
            w!("Admin Required"),
            MB_OK,
        );
    }
}

fn main() {
    // Open our own executable to read appended data
    let exe_path = env::current_exe().expect("Failed to get current exe path");
    let mut file = fs::File::open(&exe_path).expect("Failed to open exe file");
    let file_size = file.metadata().expect("Failed to get metadata").len();

    if file_size < FOOTER_SIZE as u64 {
        eprintln!("No appended resource archive found.");
        return;
    }

    // Read the footer (last FOOTER_SIZE bytes)
    file.seek(SeekFrom::End(-(FOOTER_SIZE as i64)))
        .expect("Failed to seek to footer");
    let mut footer_buf = [0u8; FOOTER_SIZE];
    file.read_exact(&mut footer_buf)
        .expect("Failed to read footer");

    let header_length = u32::from_le_bytes(footer_buf[0..4].try_into().unwrap()) as usize;
    let archive_data_length = u32::from_le_bytes(footer_buf[4..8].try_into().unwrap()) as usize;
    let marker = &footer_buf[8..24];

    if marker != FOOTER_MARKER {
        eprintln!("Invalid resource archive marker.");
        return;
    }

    // Locate and read the appended archive data
    let archive_start = file_size as i64 - (archive_data_length as i64 + FOOTER_SIZE as i64);
    if archive_start < 0 {
        eprintln!("Invalid archive start.");
        return;
    }
    file.seek(SeekFrom::Start(archive_start as u64))
        .expect("Failed to seek to archive start");

    let mut archive_data = vec![0u8; archive_data_length];
    file.read_exact(&mut archive_data)
        .expect("Failed to read archive data");

    if header_length > archive_data.len() {
        eprintln!("Invalid header length.");
        return;
    }
    let header_json = &archive_data[0..header_length];
    let resource_bytes = &archive_data[header_length..];

    // Deserialize the header JSON
    let header: ArchiveHeader = serde_json::from_slice(header_json)
        .expect("Failed to parse header JSON");

    // Check if admin rights are required and if we have them
    if header.run_as_admin {
        match is_elevated() {
            Ok(elevated) => {
                if !elevated {
                    show_message_box("Please run as administrator.");
                    return;
                }
            }
            Err(err) => {
                eprintln!("Failed to check admin rights: {}", err);
                show_message_box("Failed to check admin rights. Please run as administrator.");
                return;
            }
        }
    }

    // Create the extraction directory
    fs::create_dir_all(&header.extraction_path)
        .expect("Failed to create extraction directory");

    // Extract each resource
    let mut offset = 0;
    
    // Decompress the resource data if needed
    let mut decompressed_resource_bytes: Vec<u8>;  // Added 'mut' keyword here
    let final_resource_bytes = if header.is_compressed {
        let mut decompressor = GzDecoder::new(Cursor::new(resource_bytes));
        decompressed_resource_bytes = Vec::new();
        decompressor.read_to_end(&mut decompressed_resource_bytes)
            .expect("Failed to decompress resource data");
        &decompressed_resource_bytes
    } else {
        resource_bytes
    };
    
    for resource in &header.resources {
        let file_path = Path::new(&header.extraction_path).join(&resource.filename);
        let size = resource.size as usize;
        if offset + size > final_resource_bytes.len() {
            eprintln!("Resource data is incomplete.");
            return;
        }
        let data = &final_resource_bytes[offset..offset + size];
        fs::write(&file_path, data)
            .expect(&format!("Failed to write file {:?}", file_path));
        offset += size;
    }

    // Determine the SHOW_WINDOW_CMD value
    let show_cmd = match header.execution_style.to_lowercase().as_str() {
        "no-window"   => SW_HIDE,
        "minimized"   => SW_SHOWMINIMIZED,
        "normal"      => SW_SHOWNORMAL,
        "maximized"   => SW_SHOWMAXIMIZED,
        _             => SW_SHOWNORMAL,
    };

    // Launch the "main" file
    let main_file_path = Path::new(&header.extraction_path).join(&header.main_file);
    println!("Launching main file: {:?}", main_file_path);

    // Choose the operation verb: "runas" if elevation is requested, otherwise "open"
    let operation = if header.run_as_admin { "open" } else { "open" };

    // If the file is a batch file, run it via cmd /c
    let file_extension = main_file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    if file_extension.eq_ignore_ascii_case("bat") || file_extension.eq_ignore_ascii_case("cmd") {
        let cmd = "cmd";
        let parameters = format!("/c \"{}\"", main_file_path.to_str().unwrap());
        launch_process(operation, cmd, &parameters, show_cmd);
    } else {
        launch_process(operation, main_file_path.to_str().unwrap(), "", show_cmd);
    }
}

/// Launch a process using ShellExecuteW
/// The `show_cmd` parameter is of type SHOW_WINDOW_CMD
fn launch_process(operation: &str, file: &str, parameters: &str, show_cmd: windows::Win32::UI::WindowsAndMessaging::SHOW_WINDOW_CMD) {
    use std::ffi::OsStr;
    use std::iter;
    use std::os::windows::ffi::OsStrExt;

    let wide_operation: Vec<u16> = OsStr::new(operation)
        .encode_wide()
        .chain(iter::once(0))
        .collect();
    let wide_file: Vec<u16> = OsStr::new(file)
        .encode_wide()
        .chain(iter::once(0))
        .collect();
    let wide_parameters: Vec<u16> = OsStr::new(parameters)
        .encode_wide()
        .chain(iter::once(0))
        .collect();

    let result = unsafe {
        ShellExecuteW(
            Some(HWND(std::ptr::null_mut())),
            PCWSTR(wide_operation.as_ptr()),
            PCWSTR(wide_file.as_ptr()),
            if parameters.is_empty() {
                PCWSTR(std::ptr::null())
            } else {
                PCWSTR(wide_parameters.as_ptr())
            },
            PCWSTR(std::ptr::null()),
            show_cmd,
        )
    };

    if result.0 as isize <= 32 {
        eprintln!("ShellExecuteW failed with code: {:?}", result.0);
    }
}