; Nexium for Windows: an Inno Setup script.
;
; Built by the release workflow with
;   ISCC.exe /DAppVersion=0.1.0 /DSourceDir=C:\path\to\dist-win installers\windows\nexium.iss
; where SourceDir holds: nx.exe, README.md, LICENSE, CHANGELOG.md, docs\,
; examples\, std\, zig\ (the bundled Zig toolchain), editors\nexium.vsix.
;
; What the wizard offers: license, an overview page, install for this user or
; all users, components (compiler, bundled Zig, examples, docs, VS Code
; extension), tasks (add to PATH, register .nx, install the extension), and a
; finish page that can open the README or a console running `nx doctor`.

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif
#ifndef SourceDir
  #define SourceDir "..\..\dist-win"
#endif
#define AppName "Nexium"
#define AppPublisher "Londopy"
#define AppURL "https://github.com/Londopy/nexium"

[Setup]
AppId={{B7E4C2F1-7A5D-4B7E-9D1C-3E2A9F0C5E11}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher={#AppPublisher}
AppPublisherURL={#AppURL}
AppSupportURL={#AppURL}/issues
AppUpdatesURL={#AppURL}/releases
DefaultDirName={autopf}\Nexium
DefaultGroupName=Nexium
DisableProgramGroupPage=yes
LicenseFile={#SourceDir}\LICENSE
InfoBeforeFile={#SourcePath}\BEFORE.txt
OutputDir={#SourcePath}\Output
OutputBaseFilename=nexium-{#AppVersion}-setup-x64
SetupIconFile={#SourcePath}\..\..\assets\icon.ico
UninstallDisplayIcon={app}\nx.exe
UninstallDisplayName={#AppName} {#AppVersion}
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
ChangesEnvironment=yes
ChangesAssociations=yes
MinVersion=10.0

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Types]
Name: "full"; Description: "Full installation (recommended)"
Name: "compact"; Description: "Compiler only"
Name: "custom"; Description: "Custom installation"; Flags: iscustom

[Components]
Name: "core"; Description: "Nexium compiler (nx.exe)"; Types: full compact custom; Flags: fixed
Name: "zig"; Description: "Bundled Zig toolchain (the C compiler nx uses; recommended)"; Types: full custom
Name: "std"; Description: "Standard library sources and examples"; Types: full custom
Name: "docs"; Description: "Documentation (language reference, guides)"; Types: full custom
Name: "vscode"; Description: "VS Code extension (.vsix file)"; Types: full custom

[Tasks]
Name: "addtopath"; Description: "Add nx to the PATH"; GroupDescription: "Environment:"
Name: "assoc"; Description: "Register the .nx file type (icon and ""Open with"")"; GroupDescription: "File types:"; Flags: unchecked
Name: "installvsix"; Description: "Install the VS Code extension now (needs 'code' on the PATH)"; GroupDescription: "Editors:"; Components: vscode; Flags: unchecked

[Files]
Source: "{#SourceDir}\nx.exe"; DestDir: "{app}"; Components: core; Flags: ignoreversion
Source: "{#SourceDir}\README.md"; DestDir: "{app}"; Components: core; Flags: ignoreversion
Source: "{#SourceDir}\LICENSE"; DestDir: "{app}"; Components: core; Flags: ignoreversion
Source: "{#SourceDir}\CHANGELOG.md"; DestDir: "{app}"; Components: core; Flags: ignoreversion
Source: "{#SourceDir}\zig\*"; DestDir: "{app}\zig"; Components: zig; Flags: recursesubdirs createallsubdirs ignoreversion skipifsourcedoesntexist
Source: "{#SourceDir}\examples\*"; DestDir: "{app}\examples"; Components: std; Flags: recursesubdirs ignoreversion skipifsourcedoesntexist
Source: "{#SourceDir}\std\*"; DestDir: "{app}\std"; Components: std; Flags: recursesubdirs ignoreversion skipifsourcedoesntexist
Source: "{#SourceDir}\docs\*"; DestDir: "{app}\docs"; Components: docs; Flags: recursesubdirs ignoreversion skipifsourcedoesntexist
Source: "{#SourceDir}\editors\nexium.vsix"; DestDir: "{app}\editors"; Components: vscode; Flags: ignoreversion skipifsourcedoesntexist

[Registry]
; .nx file type: icon and a right-click "Open with Nexium (nx run)"
Root: HKA; Subkey: "Software\Classes\.nx"; ValueType: string; ValueName: ""; ValueData: "Nexium.Source"; Flags: uninsdeletevalue; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Nexium.Source"; ValueType: string; ValueName: ""; ValueData: "Nexium source file"; Flags: uninsdeletekey; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Nexium.Source\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\nx.exe,0"; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Nexium.Source\shell\run"; ValueType: string; ValueName: ""; ValueData: "Run with Nexium"; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Nexium.Source\shell\run\command"; ValueType: string; ValueName: ""; ValueData: "cmd.exe /k """"{app}\nx.exe"" run ""%1"""""; Tasks: assoc

[Icons]
Name: "{group}\Nexium README"; Filename: "{app}\README.md"
Name: "{group}\Nexium language reference"; Filename: "{app}\docs\language.md"; Components: docs
Name: "{group}\Nexium examples"; Filename: "{app}\examples"; Components: std
Name: "{group}\Nexium console"; Filename: "{cmd}"; Parameters: "/k ""{app}\nx.exe"" doctor"; WorkingDir: "{app}\examples"
Name: "{group}\Uninstall Nexium"; Filename: "{uninstallexe}"

[Run]
Filename: "{cmd}"; Parameters: "/c code --install-extension ""{app}\editors\nexium.vsix"""; StatusMsg: "Installing the VS Code extension..."; Tasks: installvsix; Flags: runhidden waituntilterminated
Filename: "{app}\README.md"; Description: "Open the README"; Flags: postinstall shellexec skipifsilent unchecked
Filename: "{cmd}"; Parameters: "/k ""{app}\nx.exe"" doctor"; Description: "Open a console and check the installation (nx doctor)"; Flags: postinstall skipifsilent nowait

[UninstallDelete]
Type: filesandordirs; Name: "{app}\nx-out"

[Code]
// PATH handling: append or remove {app} in the user's or the machine's PATH,
// depending on the install mode chosen in the wizard.

function EnvKey(): string;
begin
  if IsAdminInstallMode then
    Result := 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment'
  else
    Result := 'Environment';
end;

function EnvRoot(): Integer;
begin
  if IsAdminInstallMode then
    Result := HKEY_LOCAL_MACHINE
  else
    Result := HKEY_CURRENT_USER;
end;

procedure EnvAddPath(Dir: string);
var
  Paths: string;
begin
  if not RegQueryStringValue(EnvRoot(), EnvKey(), 'Path', Paths) then
    Paths := '';
  if Pos(';' + Uppercase(Dir) + ';', ';' + Uppercase(Paths) + ';') > 0 then
    exit;
  if (Paths <> '') and (Paths[Length(Paths)] <> ';') then
    Paths := Paths + ';';
  Paths := Paths + Dir;
  RegWriteExpandStringValue(EnvRoot(), EnvKey(), 'Path', Paths);
end;

procedure EnvRemovePath(Dir: string);
var
  Paths: string;
  P: Integer;
begin
  if not RegQueryStringValue(EnvRoot(), EnvKey(), 'Path', Paths) then
    exit;
  P := Pos(';' + Uppercase(Dir) + ';', ';' + Uppercase(Paths) + ';');
  if P = 0 then
    exit;
  Delete(Paths, P - 1, Length(Dir) + 1);
  RegWriteExpandStringValue(EnvRoot(), EnvKey(), 'Path', Paths);
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if (CurStep = ssPostInstall) and WizardIsTaskSelected('addtopath') then
    EnvAddPath(ExpandConstant('{app}'));
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
    EnvRemovePath(ExpandConstant('{app}'));
end;
