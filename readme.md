<h1 align="center">Resource Compiler</h1>
<p align="center">
  Package as many files/resources into a single executable as you want!
</p>

<p align="center">
  <a href="https://github.com/B0N3head/resource_compiler/releases/latest"><img src="https://github.com/user-attachments/assets/95bea422-5268-41bb-b2e9-e038ce5ac511" alt="Download Resource Compiler" width="500" /></a>
  <h6 align="center">Click the above image to download</h6>
</p>

### Features
<ul>
  <li><strong>File Packaging</strong>: Bundle multiple files into a single executable</li>
  <li><strong>Auto-extraction</strong>: Resources are automatically extracted when the executable is run</li>
  <li><strong>Auto-execution</strong>: Automatically run a specified main file after extraction</li>
  <li><strong>Custom Extraction Path</strong>: Specify where resources should be extracted
   <ol>
      <li>Supports environmental variables <code>%USERPROFILE%</code>, <code>%APPDATA%</code> etc</li>
      <li>Direct paths <code>C:\cool_extraction_folder</code></li>
      <li>Relative paths <code>extraction_folder</code> would be created at the same dir as the .exe</li>
    </ol>
  </li>
  <li><strong>Compression</strong>: Optional compression to reduce output file size

|Original|Non compressed|Compressed|
|-|-|-|
|384,271KB webm file|384,615KB exe file|379,456KB exe file|
  </li>
  <li><strong>Execution Options</strong>: Run the main file in different window states
  <ol>
      <li>Normal</li>
      <li>Maximized</li>
      <li>Minimized</li>
      <li>Hidden (no window)</li>
    </ol>
  </li>
  <li><strong>Administrator Rights</strong>: Option to request elevated privileges  </li>
</ul> 
</br>

### User Interface

<p align="right">
<img align="right" src="https://github.com/user-attachments/assets/a1fcb68b-09ba-4462-8776-56587eca424d" width="300" />
</p>

GUI makes it a little easier than forging some 300 character cli argument
- Drag & drop support for resources (or via file explorer)
- Resource management: Add, remove, and reorder resources
- Search functionality (no fuzzy search D:)
- Save and load project configs

</br>

### Getting Started
- Launch main_gui.exe
- Add resources using the "Add Resource" button or drag and drop
- Select your main executable from the added resources
- Configure extraction path and execution options
- Click "Compile EXE" to generate your packaged application

</br>

<h2 align="center">How It Works</h1>
The project consists of two main components:

#### Compiler GUI
A graphical tool that packages resources into a new executable

- Reads a stub executable
- Appends resource data along with metadata
- Creates a new standalone executable

#### Resource Stub
The executable template that extracts and launches resources

- Reads its own executable file to extract resources
- Creates the extraction directory
- Extracts all files while maintaining their filenames
- Launches the designated main file with specified window state

#### Project Structure
```
resource_compiler/
├── compiler_gui/          # GUI application source
│   ├── assets/            # Application assets (icons)
│   └── src/               # GUI source code
├── resource_stub/         # Stub executable source
│   └── src/               # Stub source code
├── main_gui.exe           # Compiled GUI application
└── stub.exe               # Compiled stub executable
```

###### This was made as a test/first rust project... rust is awesome 🦀
