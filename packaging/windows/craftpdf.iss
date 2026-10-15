; Script generated for CraftPDF Windows Installer
; Designed for SarthAyush

#define MyAppName "PDFix"
#ifndef MyAppVersion
  #define MyAppVersion "1.0.0"
#endif
#define MyAppPublisher "Sarthak"
#define MyAppURL "https://github.com/SarthAyush/PDFix-App"
#define MyAppExeName "PDFix.exe"

[Setup]
AppId={{D37E6144-8456-4BD3-958B-8D7DF389A9B4}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
OutputDir=..\..\dist
OutputBaseFilename=PDFix-Setup-v{#MyAppVersion}-x64
SetupIconFile=..\..\assets\app-icon\craftpdf.ico
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequiredOverridesAllowed=commandline dialog
UninstallDisplayIcon={app}\{#MyAppExeName}

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"
Name: "contextmenu"; Description: "Add 'Edit with PDFix' to PDF right-click context menu"; GroupDescription: "File Explorer Integration:"; Flags: unchecked
Name: "fileassoc"; Description: "Register PDFix as a PDF reader"; GroupDescription: "File Associations:"

[Files]
Source: "..\..\target\release\pdfcraft.exe"; DestDir: "{app}"; DestName: "{#MyAppExeName}"; Flags: ignoreversion
Source: "..\..\assets\app-icon\craftpdf.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\assets\app-icon\craftpdf-document.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#MyAppName}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\craftpdf.ico"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\craftpdf.ico"; Tasks: desktopicon

[Registry]
; Register .pdf file handler
Root: HKA; Subkey: "Software\Classes\.pdf\OpenWithProgids"; ValueType: string; ValueName: "PDFix.Document"; Tasks: fileassoc; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\PDFix.Document"; ValueType: string; ValueData: "PDF Document"; Tasks: fileassoc; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\PDFix.Document\DefaultIcon"; ValueType: string; ValueData: "{app}\craftpdf-document.ico,0"; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\PDFix.Document\shell\open\command"; ValueType: string; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: fileassoc

; Context menu item: "Edit with PDFix"
Root: HKA; Subkey: "Software\Classes\SystemFileAssociations\.pdf\shell\PDFix"; ValueType: string; ValueData: "Edit with PDFix"; Tasks: contextmenu; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\SystemFileAssociations\.pdf\shell\PDFix"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\craftpdf.ico"; Tasks: contextmenu
Root: HKA; Subkey: "Software\Classes\SystemFileAssociations\.pdf\shell\PDFix\command"; ValueType: string; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: contextmenu

; Capabilities registration for Windows Settings -> Default Apps
Root: HKA; Subkey: "Software\PDFix\Capabilities"; ValueType: string; ValueName: "ApplicationName"; ValueData: "{#MyAppName}"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\PDFix\Capabilities"; ValueType: string; ValueName: "ApplicationDescription"; ValueData: "PDFix PDF Workbench (Configured by Sarthak)"
Root: HKA; Subkey: "Software\PDFix\Capabilities"; ValueType: string; ValueName: "ApplicationIcon"; ValueData: "{app}\craftpdf.ico,0"
Root: HKA; Subkey: "Software\PDFix\Capabilities\FileAssociations"; ValueType: string; ValueName: ".pdf"; ValueData: "PDFix.Document"
Root: HKA; Subkey: "Software\RegisteredApplications"; ValueType: string; ValueName: "{#MyAppName}"; ValueData: "Software\PDFix\Capabilities"; Flags: uninsdeletevalue

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
