"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
const vscode = __importStar(require("vscode"));
const prettier = __importStar(require("prettier"));
/* eslint-disable */ // <-- Before function
function activate(context) {
    vscode.languages.registerDocumentFormattingEditProvider("sailfish", {
        async provideDocumentFormattingEdits(document) {
            const edits = [];
            const entireRange = new vscode.Range(document.positionAt(0), document.positionAt(document.getText().length));
            const formatted = await formatSailfishHTML(document.getText());
            edits.push(vscode.TextEdit.replace(entireRange, formatted));
            return edits;
        },
    });
}
/* eslint-enable */ // <-- After function
async function formatSailfishHTML(text) {
    const placeholders = {};
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
    const finalFormattedText = formattedText.replace(/__SAILFISH_PLACEHOLDER_\d+__/g, (match) => {
        return placeholders[match];
    });
    return finalFormattedText;
}
//# sourceMappingURL=extension.js.map