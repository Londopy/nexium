; Nexium for Windows: an Inno Setup script.
;
; Built by the release workflow with
;   ISCC.exe /DAppVersion=0.1.0 /DSourceDir=C:\path\to\dist-win installers\windows\nexium.iss
; where SourceDir holds: nx.exe, README.md, LICENSE, CHANGELOG.md, docs\,
; examples\, std\, zig\ (the bundled Zig toolchain), editors\nexium.vsix.
;
; What the wizard offers: a welcome page, license, an overview page, install
; for this user or all users, components (compiler, bundled Zig, examples,
; docs, VS Code extension), tasks (add to PATH, register .nx, a console entry
; in Explorer's folder menu, a desktop shortcut, install the extension), a
; page presenting the publisher's other projects with open and download
; buttons, and a finish page that can open the README, the changelog, or a
; console running `nx doctor`.
;
; The wizard images are drawn by scripts/make_wizard_images.py.

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
AppComments=The Nexium language: compiler, bundled C toolchain, standard library and docs
AppContact={#AppURL}/issues
VersionInfoVersion={#AppVersion}
VersionInfoDescription=Nexium {#AppVersion} setup
VersionInfoProductName=Nexium
DefaultDirName={autopf}\Nexium
DefaultGroupName=Nexium
DisableProgramGroupPage=yes
DisableWelcomePage=no
LicenseFile={#SourceDir}\LICENSE
InfoBeforeFile={#SourcePath}\BEFORE.txt
OutputDir={#SourcePath}\Output
OutputBaseFilename=nexium-{#AppVersion}-setup-x64
SetupIconFile={#SourcePath}\..\..\assets\icon.ico
WizardImageFile={#SourcePath}\wizard-large.bmp
WizardSmallImageFile={#SourcePath}\wizard-small.bmp
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
; an install over an install: the previous directory, type, components,
; tasks, language and privileges are the defaults (the page after the
; welcome, under [Code], offers the upgrade, the repair or the removal)
UsePreviousAppDir=yes
UsePreviousSetupType=yes
UsePreviousTasks=yes
UsePreviousLanguage=yes
UsePreviousPrivileges=yes
; one installer at a time: Setup and Uninstall refuse to start while another
; holds this mutex, and say so (the message is under [Messages])
SetupMutex=NexiumSetupMutex
; a running nx.exe holds the files being replaced: Restart Manager closes it
; first, and does not start it again (a console program has nothing to resume)
CloseApplications=yes
CloseApplicationsFilter=*.exe,*.dll
RestartApplications=no
; every step of the install, kept next to the program as install.log
; (see CurStepChanged); the uninstaller logs with /LOG="file"
SetupLogging=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Messages]
SetupAppRunningError=Another Nexium installer (%1) is already open.%n%nFinish or close it, then click OK to try again, or Cancel to exit.
UninstallAppRunningError=Another Nexium installer or uninstaller (%1) is already open.%n%nFinish or close it, then click OK to try again, or Cancel to exit.

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
Name: "foldermenu"; Description: "Add ""Open Nexium console here"" and ""Open Nexium REPL here"" to the folder right-click menu"; GroupDescription: "Explorer:"; Flags: unchecked
Name: "wtprofile"; Description: "Add a ""Nexium REPL"" profile to Windows Terminal"; GroupDescription: "Windows Terminal:"; Flags: unchecked
Name: "desktopicon"; Description: "Create a desktop shortcut to the Nexium console"; GroupDescription: "Shortcuts:"; Flags: unchecked
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
Source: "{#SourceDir}\nexium.ico"; DestDir: "{app}"; Components: core; Flags: ignoreversion skipifsourcedoesntexist

[Registry]
; .nx file type: icon and a right-click "Open with Nexium (nx run)"
Root: HKA; Subkey: "Software\Classes\.nx"; ValueType: string; ValueName: ""; ValueData: "Nexium.Source"; Flags: uninsdeletevalue; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Nexium.Source"; ValueType: string; ValueName: ""; ValueData: "Nexium source file"; Flags: uninsdeletekey; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Nexium.Source\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\nx.exe,0"; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Nexium.Source\shell\run"; ValueType: string; ValueName: ""; ValueData: "Run with Nexium"; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Nexium.Source\shell\run\command"; ValueType: string; ValueName: ""; ValueData: "cmd.exe /k """"{app}\nx.exe"" run ""%1"""""; Tasks: assoc
; right-click a folder's background: a console with nx on the PATH, in that folder
Root: HKA; Subkey: "Software\Classes\Directory\Background\shell\NexiumConsole"; ValueType: string; ValueName: ""; ValueData: "Open Nexium console here"; Flags: uninsdeletekey; Tasks: foldermenu
Root: HKA; Subkey: "Software\Classes\Directory\Background\shell\NexiumConsole"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\nx.exe,0"; Tasks: foldermenu
Root: HKA; Subkey: "Software\Classes\Directory\Background\shell\NexiumConsole\command"; ValueType: string; ValueName: ""; ValueData: "cmd.exe /k ""set PATH={app};%PATH% && ""{app}\nx.exe"" version"""; Tasks: foldermenu
; right-click a folder's background: the interactive session, in that folder
Root: HKA; Subkey: "Software\Classes\Directory\Background\shell\NexiumRepl"; ValueType: string; ValueName: ""; ValueData: "Open Nexium REPL here"; Flags: uninsdeletekey; Tasks: foldermenu
Root: HKA; Subkey: "Software\Classes\Directory\Background\shell\NexiumRepl"; ValueType: string; ValueName: "Icon"; ValueData: "{app}\nx.exe,0"; Tasks: foldermenu
Root: HKA; Subkey: "Software\Classes\Directory\Background\shell\NexiumRepl\command"; ValueType: string; ValueName: ""; ValueData: "cmd.exe /c ""cd /d ""%V"" && set PATH={app};%PATH% && ""{app}\nx.exe"" repl"""; Tasks: foldermenu

[Icons]
Name: "{group}\Nexium {#AppVersion} (64-bit)"; Filename: "{app}\nx.exe"; IconFilename: "{app}\nexium.ico"; WorkingDir: "{userdocs}"; Comment: "The Nexium interactive session (nx repl)"
Name: "{group}\Nexium README"; Filename: "{app}\README.md"
Name: "{group}\Nexium language reference"; Filename: "{app}\docs\language.md"; Components: docs
Name: "{group}\Nexium examples"; Filename: "{app}\examples"; Components: std
Name: "{group}\Nexium console"; Filename: "{cmd}"; Parameters: "/k ""{app}\nx.exe"" doctor"; WorkingDir: "{app}\examples"
Name: "{group}\What's new in Nexium"; Filename: "{app}\CHANGELOG.md"
Name: "{group}\Nexium on GitHub"; Filename: "{#AppURL}"
Name: "{group}\Uninstall Nexium"; Filename: "{uninstallexe}"
Name: "{autodesktop}\Nexium console"; Filename: "{cmd}"; Parameters: "/k ""{app}\nx.exe"" doctor"; WorkingDir: "{userdocs}"; IconFilename: "{app}\nexium.ico"; Tasks: desktopicon

[Run]
Filename: "{cmd}"; Parameters: "/c code --install-extension ""{app}\editors\nexium.vsix"""; StatusMsg: "Installing the VS Code extension..."; Tasks: installvsix; Flags: runhidden waituntilterminated
Filename: "{app}\nx.exe"; Description: "Launch the Nexium interactive session"; Flags: postinstall skipifsilent nowait
Filename: "{cmd}"; Parameters: "/k ""{app}\nx.exe"" doctor"; Description: "Open a console and check the installation (nx doctor)"; Flags: postinstall skipifsilent nowait unchecked
Filename: "{app}\README.md"; Description: "Open the README"; Flags: postinstall shellexec skipifsilent unchecked
Filename: "{app}\CHANGELOG.md"; Description: "Show what's new in {#AppVersion}"; Flags: postinstall shellexec skipifsilent unchecked

[UninstallDelete]
Type: filesandordirs; Name: "{app}\nx-out"
Type: files; Name: "{app}\install.log"
; the Windows Terminal profile (whichever install mode wrote it)
Type: files; Name: "{localappdata}\Microsoft\Windows Terminal\Fragments\Nexium\nexium.json"
Type: dirifempty; Name: "{localappdata}\Microsoft\Windows Terminal\Fragments\Nexium"
Type: files; Name: "{commonappdata}\Microsoft\Windows Terminal\Fragments\Nexium\nexium.json"
Type: dirifempty; Name: "{commonappdata}\Microsoft\Windows Terminal\Fragments\Nexium"

[Code]
// ---- an installed version: when Nexium is already here, the page after the
// welcome says which version and where, and offers the upgrade (a reinstall
// when it is the same version, a replacement when it is newer) with the
// previous choices as defaults, or to remove it and exit.

procedure ExitProcess(Code: Integer); external 'ExitProcess@kernel32.dll stdcall';

var
  InstalledPage: TInputOptionWizardPage;
  InstalledVersion, InstalledDir, UninstallCmd: string;

function UninstallKey(): string;
begin
  Result := 'Software\Microsoft\Windows\CurrentVersion\Uninstall\{B7E4C2F1-7A5D-4B7E-9D1C-3E2A9F0C5E11}_is1';
end;

function FindInstalled(): Boolean;
var
  Root: Integer;
begin
  Result := False;
  Root := HKEY_CURRENT_USER;
  if not RegQueryStringValue(Root, UninstallKey(), 'DisplayVersion', InstalledVersion) then
  begin
    Root := HKEY_LOCAL_MACHINE;
    if not RegQueryStringValue(Root, UninstallKey(), 'DisplayVersion', InstalledVersion) then
      exit;
  end;
  RegQueryStringValue(Root, UninstallKey(), 'InstallLocation', InstalledDir);
  RegQueryStringValue(Root, UninstallKey(), 'UninstallString', UninstallCmd);
  Result := UninstallCmd <> '';
end;

// the numeric part of a dotted version, one component at a time
function VersionPart(var S: string): Integer;
var
  P: Integer;
begin
  P := Pos('.', S);
  if P = 0 then
  begin
    Result := StrToIntDef(S, 0);
    S := '';
  end else begin
    Result := StrToIntDef(Copy(S, 1, P - 1), 0);
    Delete(S, 1, P);
  end;
end;

// negative when A is older than B, zero when equal, positive when newer
function CompareVersions(A, B: string): Integer;
var
  I, X, Y: Integer;
begin
  Result := 0;
  for I := 1 to 3 do
  begin
    X := VersionPart(A);
    Y := VersionPart(B);
    if X <> Y then
    begin
      Result := X - Y;
      exit;
    end;
  end;
end;

function UpgradeCaption(): string;
var
  C: Integer;
begin
  C := CompareVersions('{#AppVersion}', InstalledVersion);
  if C > 0 then
    Result := 'Upgrade to {#AppVersion} (your choices from the last install are the defaults)'
  else if C = 0 then
    Result := 'Reinstall {#AppVersion} (a repair: every file is written again)'
  else
    Result := 'Replace it with {#AppVersion}, an older version';
end;

procedure OfferInstalledPage();
begin
  if not FindInstalled() then
    exit;
  InstalledPage := CreateInputOptionPage(wpWelcome, 'Nexium is already installed',
    'Version ' + InstalledVersion + ' is in ' + InstalledDir,
    'This setup is version {#AppVersion}. What would you like to do?', True, False);
  InstalledPage.Add(UpgradeCaption());
  InstalledPage.Add('Remove the installed version and exit');
  InstalledPage.Values[0] := True;
end;

function NextButtonClick(CurPageID: Integer): Boolean;
var
  Code: Integer;
begin
  Result := True;
  if (InstalledPage <> nil) and (CurPageID = InstalledPage.ID) and InstalledPage.Values[1] then
  begin
    if Exec(RemoveQuotes(UninstallCmd), '', '', SW_SHOW, ewWaitUntilTerminated, Code) then
      MsgBox('Nexium ' + InstalledVersion + ' was removed. Run this setup again to install {#AppVersion}.', mbInformation, MB_OK)
    else
      MsgBox('The uninstaller could not be started: ' + SysErrorMessage(Code), mbError, MB_OK);
    ExitProcess(0);
  end;
end;

// ---- "More from Londopy": a page after the tasks presenting the publisher's
// other projects, each with a button that opens its GitHub page and one that
// opens its latest release. Nothing is downloaded by the installer itself.

const
  ProjectCount = 3;

var
  MorePage: TWizardPage;
  ProjectUrls: array[0..ProjectCount - 1] of string;

procedure OpenProjectClick(Sender: TObject);
var
  Url: string;
  Code: Integer;
begin
  Url := ProjectUrls[TNewButton(Sender).Tag div 2];
  if TNewButton(Sender).Tag mod 2 = 1 then
    Url := Url + '/releases/latest';
  ShellExec('open', Url, '', '', SW_SHOWNORMAL, ewNoWait, Code);
end;

procedure AddProject(Index: Integer; Top: Integer; const Name, Blurb, Url: string);
var
  Title, Text: TNewStaticText;
  OpenBtn, DownloadBtn: TNewButton;
begin
  ProjectUrls[Index] := Url;
  Title := TNewStaticText.Create(MorePage);
  Title.Parent := MorePage.Surface;
  Title.Left := 0;
  Title.Top := Top;
  Title.Caption := Name;
  Title.Font.Style := [fsBold];
  Text := TNewStaticText.Create(MorePage);
  Text.Parent := MorePage.Surface;
  Text.Left := 0;
  Text.Top := Top + ScaleY(18);
  Text.Width := MorePage.SurfaceWidth - ScaleX(220);
  Text.WordWrap := True;
  Text.AutoSize := False;
  Text.Height := ScaleY(48);
  Text.Caption := Blurb;
  OpenBtn := TNewButton.Create(MorePage);
  OpenBtn.Parent := MorePage.Surface;
  OpenBtn.Left := MorePage.SurfaceWidth - ScaleX(210);
  OpenBtn.Top := Top + ScaleY(14);
  OpenBtn.Width := ScaleX(100);
  OpenBtn.Height := ScaleY(23);
  OpenBtn.Caption := 'Open on GitHub';
  OpenBtn.Tag := Index * 2;
  OpenBtn.OnClick := @OpenProjectClick;
  DownloadBtn := TNewButton.Create(MorePage);
  DownloadBtn.Parent := MorePage.Surface;
  DownloadBtn.Left := MorePage.SurfaceWidth - ScaleX(104);
  DownloadBtn.Top := Top + ScaleY(14);
  DownloadBtn.Width := ScaleX(104);
  DownloadBtn.Height := ScaleY(23);
  DownloadBtn.Caption := 'Download';
  DownloadBtn.Tag := Index * 2 + 1;
  DownloadBtn.OnClick := @OpenProjectClick;
end;

procedure InitializeWizard();
var
  Intro: TNewStaticText;
begin
  OfferInstalledPage();
  MorePage := CreateCustomPage(wpSelectTasks, 'More from Londopy',
    'Other free tools by the author of Nexium. Nothing here is installed unless you download it yourself.');
  Intro := TNewStaticText.Create(MorePage);
  Intro.Parent := MorePage.Surface;
  Intro.Left := 0;
  Intro.Top := 0;
  Intro.Width := MorePage.SurfaceWidth;
  Intro.WordWrap := True;
  Intro.AutoSize := False;
  Intro.Height := ScaleY(28);
  Intro.Caption := 'Each button opens a page in your browser. "Download" goes to the latest release.';
  AddProject(0, ScaleY(40), 'HideDesktopApps',
    'A lightweight system-tray app that hides and shows desktop icons, the taskbar and all windows with hotkeys. For ricing, streaming and focus.',
    'https://github.com/Londopy/HideDesktopApps');
  AddProject(1, ScaleY(116), 'capture-bypass',
    'A DLL injection tool that bypasses screen-capture protection on Windows 10 and 11, so protected windows show up in recordings.',
    'https://github.com/Londopy/capture-bypass');
  AddProject(2, ScaleY(192), 'gesture-synth',
    'A chord instrument you play with your hands in front of a camera: hand tracking drives a polyphonic synth in the browser.',
    'https://github.com/Londopy/gesture-synth');
end;

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

// Windows Terminal: a profile "Nexium REPL", as a JSON fragment in the
// per-user or the all-users fragments directory (Terminal reads both and
// shows the profile in its menu; no settings.json is touched).

function JsonPath(const Path: string): string;
begin
  Result := Path;
  StringChangeEx(Result, '\', '\\', True);
end;

procedure WriteTerminalProfile();
var
  Dir, Text: string;
begin
  if IsAdminInstallMode then
    Dir := ExpandConstant('{commonappdata}\Microsoft\Windows Terminal\Fragments\Nexium')
  else
    Dir := ExpandConstant('{localappdata}\Microsoft\Windows Terminal\Fragments\Nexium');
  ForceDirectories(Dir);
  Text := '{' + #13#10 +
    '  "$help": "https://aka.ms/terminal-documentation",' + #13#10 +
    '  "profiles": [' + #13#10 +
    '    {' + #13#10 +
    '      "name": "Nexium REPL",' + #13#10 +
    '      "commandline": "\"' + JsonPath(ExpandConstant('{app}\nx.exe')) + '\" repl",' + #13#10 +
    '      "icon": "' + JsonPath(ExpandConstant('{app}\nexium.ico')) + '",' + #13#10 +
    '      "startingDirectory": "%USERPROFILE%"' + #13#10 +
    '    }' + #13#10 +
    '  ]' + #13#10 +
    '}' + #13#10;
  SaveStringToFile(Dir + '\nexium.json', Text, False);
end;

var
  TerminalOffered: Boolean;

procedure CurPageChanged(CurPageID: Integer);
begin
  // the profile task is checked once, when Windows Terminal is installed
  if (CurPageID = wpSelectTasks) and not TerminalOffered then
  begin
    TerminalOffered := True;
    if FileExists(ExpandConstant('{localappdata}\Microsoft\WindowsApps\wt.exe')) then
      WizardSelectTasks('wtprofile');
  end;
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if (CurStep = ssPostInstall) and WizardIsTaskSelected('addtopath') then
    EnvAddPath(ExpandConstant('{app}'));
  if (CurStep = ssPostInstall) and WizardIsTaskSelected('wtprofile') then
    WriteTerminalProfile();
  // the log of this run, next to the program (SetupLogging=yes writes it)
  if CurStep = ssDone then
    FileCopy(ExpandConstant('{log}'), ExpandConstant('{app}\install.log'), False);
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
    EnvRemovePath(ExpandConstant('{app}'));
end;
