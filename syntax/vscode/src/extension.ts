import * as vscode from "vscode";
import * as prettier from "prettier";

/* eslint-disable */ // <-- Before function
export function activate(context: vscode.ExtensionContext) {
  vscode.languages.registerDocumentFormattingEditProvider("sailfish", {
    async provideDocumentFormattingEdits(
      document: vscode.TextDocument
    ): Promise<vscode.TextEdit[]> {
      const edits: vscode.TextEdit[] = [];
      const entireRange = new vscode.Range(
        document.positionAt(0),
        document.positionAt(document.getText().length)
      );

      const formatted = await formatSailfishHTML(document.getText());
      edits.push(vscode.TextEdit.replace(entireRange, formatted));

      return edits;
    },
  });
}
/* eslint-enable */  // <-- After function

async function formatSailfishHTML(text: string): Promise<string> {
  const placeholders: { [key: string]: string } = {};
  let placeholderCounter = 0;

  const textWithPlaceholders = text.replace(/<%.*?%>/g, (match) => {
    const placeholder = `__SAILFISH_PLACEHOLDER_${placeholderCounter++}__`;
    placeholders[placeholder] = match;
    return placeholder;
  });

  const formattedText = await prettier.format(textWithPlaceholders, {
    parser: "html",
  });

  // Replace placeholders back with the original template tags
  const finalFormattedText = formattedText.replace(
    /__SAILFISH_PLACEHOLDER_\d+__/g,
    (match) => {
      return placeholders[match];
    }
  );

  return finalFormattedText;
}
