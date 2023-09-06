import { init, render } from "rimu-wasm";
import { basicSetup, EditorView } from "codemirror";
import * as yamlMode from "@codemirror/legacy-modes/mode/yaml";
import { StreamLanguage, LanguageSupport } from "@codemirror/language";
import {
  linter as createLinter,
  lintGutter,
  openLintPanel,
} from "@codemirror/lint";

const yaml = new LanguageSupport(StreamLanguage.define(yamlMode.yaml));

init();

const sourceId = "playground";

const inputEl = document.getElementById("input");
const renderButtonEl = document.getElementById("render");
const outputEl = document.getElementById("output");

const linter = createLinter(
  (view) => {
    const diagnostics = [];

    const input = inputView.state.sliceDoc();
    console.log("input", input);

    let output;
    try {
      output = render(input, sourceId);
    } catch (err) {
      const { reports } = err;
      if (reports == null) throw err;

      for (const report of reports) {
        const { span } = report;
        let message = report.message;
        for (const [_span, label] of report.labels) {
          message += "\n\n" + label;
        }
        diagnostics.push({
          from: span.start,
          to: span.end,
          severity: "error",
          message,
        });
      }
    }

    if (output !== undefined) {
      console.log("value", output);
      outputEl.innerText = JSON.stringify(output, null, 2);
    }

    return diagnostics;
  },
  {
    delay: 250,
  }
);

const inputView = new EditorView({
  doc: `
hello:
  world: 10 + 2
`,
  extensions: [basicSetup, yaml, linter, lintGutter()],
  parent: inputEl,
});

openLintPanel(inputView);
