import * as vscode from 'vscode';
import * as path from 'path';

let diagnosticCollection: vscode.DiagnosticCollection;
let outputChannel: vscode.OutputChannel;

export async function activate(context: vscode.ExtensionContext) {
    outputChannel = vscode.window.createOutputChannel("RAPID");
    context.subscriptions.push(outputChannel);

    outputChannel.appendLine('RAPID extension is now active!');

    diagnosticCollection = vscode.languages.createDiagnosticCollection('rapid');
    context.subscriptions.push(diagnosticCollection);

    const wasmModule = await import(path.join(__dirname, '..', 'pkg', 'rapid_wasm.js'));
    let disposable = vscode.workspace.onDidChangeTextDocument(event => {
        if (event.document.languageId === 'rapid') {
            updateDiagnostics(event.document, wasmModule);
        }
    });

    context.subscriptions.push(disposable);

    // Run once for the current file
    if (vscode.window.activeTextEditor) {
        updateDiagnostics(vscode.window.activeTextEditor.document, wasmModule);
    }
}

function updateDiagnostics(document: vscode.TextDocument, wasmModule: any) {
    const text = document.getText();
    try {
        const result = JSON.parse(wasmModule.parse_rapid(text));
        if (result.success) {
            diagnosticCollection.delete(document.uri);
        } else {
            const diagnostics: vscode.Diagnostic[] = [];
            for (const error of result.errors) {
                if (error.error_position) {
                    const [startPos, endPos] = error.error_position;
                    const range = new vscode.Range(
                        document.positionAt(startPos),
                        document.positionAt(endPos)
                    );
                    const diagnostic = new vscode.Diagnostic(range, error.message, vscode.DiagnosticSeverity.Error);
                    diagnostics.push(diagnostic);
                } else {
                    // If no specific position is available, mark the entire document
                    const range = new vscode.Range(
                        document.positionAt(0),
                        document.positionAt(document.getText().length)
                    );
                    const diagnostic = new vscode.Diagnostic(range, error.message, vscode.DiagnosticSeverity.Error);
                    diagnostics.push(diagnostic);
                }
            }
            diagnosticCollection.set(document.uri, diagnostics);
        }
    } catch (error) {
        outputChannel.appendLine(`Error parsing RAPID code: ${error}`);
    }
}

export function deactivate() {}