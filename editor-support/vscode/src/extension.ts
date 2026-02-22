import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const workspaceFolder = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  const serverOptions: ServerOptions = {
    command: serverPath(),
    args: ["lsp", "--transport", transportMode()],
    options: workspaceFolder ? { cwd: workspaceFolder } : undefined,
  };
  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "vibelang" }],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher("**/*.{yb,vibe}"),
    },
  };

  client = new LanguageClient(
    "vibelang",
    "VibeLang Language Server",
    serverOptions,
    clientOptions
  );

  client.start();
  context.subscriptions.push(client);

  context.subscriptions.push(
    vscode.workspace.onWillSaveTextDocument((event) => {
      if (event.document.languageId !== "vibelang") {
        return;
      }
      const enabled = vscode.workspace
        .getConfiguration("vibelang")
        .get<boolean>("formatOnSave", true);
      if (!enabled) {
        return;
      }
      event.waitUntil(
        vscode.commands
          .executeCommand<vscode.TextEdit[]>(
            "vscode.executeFormatDocumentProvider",
            event.document.uri
          )
          .then((edits) => edits ?? [])
      );
    })
  );
}

export async function deactivate(): Promise<void> {
  if (client) {
    await client.stop();
    client = undefined;
  }
}

function serverPath(): string {
  return (
    vscode.workspace
      .getConfiguration("vibelang")
      .get<string>("server.path") ?? "vibe"
  );
}

function transportMode(): string {
  return (
    vscode.workspace
      .getConfiguration("vibelang")
      .get<string>("lsp.transport") ?? "jsonrpc"
  );
}

