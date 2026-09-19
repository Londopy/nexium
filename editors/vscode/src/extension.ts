// Nexium for VS Code: starts `nx lsp` (diagnostics and hover) and adds a
// "Run Current File" command. Highlighting comes from the TextMate grammar.
import * as vscode from "vscode";
import { LanguageClient, LanguageClientOptions, ServerOptions } from "vscode-languageclient/node";

let client: LanguageClient | undefined;

function nxPath(): string {
  return vscode.workspace.getConfiguration("nexium").get<string>("nxPath", "nx");
}

async function startServer(context: vscode.ExtensionContext): Promise<void> {
  const config = vscode.workspace.getConfiguration("nexium");
  if (!config.get<boolean>("enableLanguageServer", true)) {
    return;
  }
  const serverOptions: ServerOptions = { command: nxPath(), args: ["lsp"] };
  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "nexium" }],
    synchronize: { fileEvents: vscode.workspace.createFileSystemWatcher("**/*.nx") },
  };
  client = new LanguageClient("nexium", "Nexium Language Server", serverOptions, clientOptions);
  try {
    await client.start();
    context.subscriptions.push(client);
  } catch (err) {
    client = undefined;
    const choice = await vscode.window.showWarningMessage(
      `Nexium: could not start \`${nxPath()} lsp\`. Install nx and put it on PATH, or set nexium.nxPath.`,
      "Open Settings"
    );
    if (choice === "Open Settings") {
      vscode.commands.executeCommand("workbench.action.openSettings", "nexium.nxPath");
    }
  }
}

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  await startServer(context);

  context.subscriptions.push(
    vscode.commands.registerCommand("nexium.restartServer", async () => {
      if (client) {
        await client.stop();
        client = undefined;
      }
      await startServer(context);
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("nexium.run", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor || editor.document.languageId !== "nexium") {
        vscode.window.showInformationMessage("Nexium: open a .nx file first.");
        return;
      }
      await editor.document.save();
      const terminal = vscode.window.terminals.find((t) => t.name === "Nexium") ?? vscode.window.createTerminal("Nexium");
      terminal.show();
      terminal.sendText(`${nxPath()} run "${editor.document.fileName}"`);
    })
  );

  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration(async (e) => {
      if (e.affectsConfiguration("nexium")) {
        await vscode.commands.executeCommand("nexium.restartServer");
      }
    })
  );
}

export async function deactivate(): Promise<void> {
  if (client) {
    await client.stop();
    client = undefined;
  }
}
