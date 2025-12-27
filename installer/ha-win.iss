#define AppName "HA Win"
#define AppPublisher "derDeno"
#define AppExeName "HaWin.exe"
#define AppId "{{B5A68C5E-1C5D-4C0D-A3F6-8F4C3B2C6E1A}}"

#ifndef AppVersion
  #if FileExists("..\\artifacts\\publish\\HaWin.exe")
    #define AppVersion GetVersionNumbersString("..\\artifacts\\publish\\HaWin.exe")
  #else
    #define AppVersion "0.0.0"
  #endif
#endif

[Setup]
AppId={#AppId}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#AppPublisher}
DefaultDirName={autopf}\HA Win
DefaultGroupName={#AppName}
UninstallDisplayIcon={app}\{#AppExeName}
SetupIconFile=..\icon.ico
OutputDir=..\artifacts\installer
OutputBaseFilename=HA-Win-Setup
Compression=lzma2
SolidCompression=yes
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
WizardStyle=modern
UsePreviousAppDir=yes
DisableDirPage=auto
VersionInfoVersion={#AppVersion}
VersionInfoProductVersion={#AppVersion}

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "startmenuicon"; Description: "Create a &Start Menu shortcut"; GroupDescription: "Shortcuts:"; Flags: checkedonce
Name: "desktopicon"; Description: "Create a &desktop icon"; GroupDescription: "Additional icons:"; Flags: unchecked

[Files]
Source: "..\\artifacts\\publish\\*"; DestDir: "{app}"; Flags: recursesubdirs createallsubdirs ignoreversion

[Icons]
Name: "{group}\\{#AppName}"; Filename: "{app}\\{#AppExeName}"; Tasks: startmenuicon
Name: "{autodesktop}\\{#AppName}"; Filename: "{app}\\{#AppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\\{#AppExeName}"; Description: "Launch {#AppName}"; Flags: nowait postinstall skipifsilent
