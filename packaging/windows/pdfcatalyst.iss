; Script generated for PDFCatalyst Windows Installer
; Designed for SarthAyush

#define MyAppName "PDFCatalyst"
#ifndef MyAppVersion
  #define MyAppVersion "1.0.0"
#endif
#define MyAppPublisher "Sarthak"
#define MyAppURL "https://github.com/SarthAyush/PDFcatalyst-App"
#define MyAppExeName "PDFCatalyst.exe"

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
OutputBaseFilename=PDFCatalyst-Setup-v{#MyAppVersion}-x64
SetupIconFile=..\..\assets\app-icon\pdfcatalyst.ico
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequiredOverridesAllowed=commandline dialog
UninstallDisplayIcon={app}\{#MyAppExeName}
CloseApplications=yes
RestartApplications=yes
VersionInfoVersion={#MyAppVersion}
VersionInfoCompany={#MyAppPublisher}
VersionInfoDescription={#MyAppName} Setup Wizard
VersionInfoCopyright=Copyright (C) 2026 {#MyAppPublisher}
VersionInfoProductName={#MyAppName}
VersionInfoProductVersion={#MyAppVersion}

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"
Name: "contextmenu"; Description: "Add 'Edit with PDFCatalyst' to PDF right-click context menu"; GroupDescription: "File Explorer Integration:"; Flags: unchecked
Name: "fileassoc"; Description: "Register PDFCatalyst as a PDF reader"; GroupDescription: "File Associations:"

[Files]
Source: "..\..\target\release\PDFCatalyst.exe"; DestDir: "{app}"; DestName: "{#MyAppExeName}"; Flags: ignoreversion
Source: "..\..\assets\app-icon\pdfcatalyst.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\assets\app-icon\pdfcatalyst-document.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#MyAppName}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\pdfcatalyst.ico"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\pdfcatalyst.ico"; Tasks: desktopicon

[Registry]
; Register .pdf file handler
Root: HKA; Subkey: "Software\Classes\.pdf\OpenWithProgids"; ValueType: string; ValueName: "PDFCatalyst.Document"; Tasks: fileassoc; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\PDFCatalyst.Document"; ValueType: string; ValueData: "PDF Document"; Tasks: fileassoc; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\PDFCatalyst.Document\DefaultIcon"; ValueType: string; ValueData: "{app}\pdfcatalyst-document.ico,0"; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\PDFCatalyst.Document\shell\open\command"; ValueType: string; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: fileassoc

; Context menu item: "Edit with PDFCatalyst"
Root: HKA; Subkey: "Software\Classes\SystemFileAssociations\.pdf\shell\PDFCatalyst"; ValueType: string; ValueData: "Edit with PDFCatalyst"; Tasks: contextmenu; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\SystemFileAssociations\.pdf\shell\PDFCatalyst"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\pdfcatalyst.ico"; Tasks: contextmenu
Root: HKA; Subkey: "Software\Classes\SystemFileAssociations\.pdf\shell\PDFCatalyst\command"; ValueType: string; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: contextmenu

; Capabilities registration for Windows Settings -> Default Apps
Root: HKA; Subkey: "Software\PDFCatalyst\Capabilities"; ValueType: string; ValueName: "ApplicationName"; ValueData: "{#MyAppName}"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\PDFCatalyst\Capabilities"; ValueType: string; ValueName: "ApplicationDescription"; ValueData: "PDFCatalyst PDF Workbench (Configured by Sarthak)"
Root: HKA; Subkey: "Software\PDFCatalyst\Capabilities"; ValueType: string; ValueName: "ApplicationIcon"; ValueData: "{app}\pdfcatalyst.ico,0"
Root: HKA; Subkey: "Software\PDFCatalyst\Capabilities\FileAssociations"; ValueType: string; ValueName: ".pdf"; ValueData: "PDFCatalyst.Document"
Root: HKA; Subkey: "Software\RegisteredApplications"; ValueType: string; ValueName: "{#MyAppName}"; ValueData: "Software\PDFCatalyst\Capabilities"; Flags: uninsdeletevalue

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
