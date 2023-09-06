import { init, render } from "rimu-wasm";
import { basicSetup, EditorView } from "codemirror";
import * as yamlMode from "@codemirror/legacy-modes/mode/yaml";
import { StreamLanguage, LanguageSupport } from "@codemirror/language";

const yaml = new LanguageSupport(StreamLanguage.define(yamlMode.yaml));

init();

const sourceId = "playground";

const inputEl = document.getElementById("input");
const renderButtonEl = document.getElementById("render");
const outputEl = document.getElementById("output");

const inputView = new EditorView({
  doc: `
hello:
  world: 10 + 2
`,
  extensions: [basicSetup, yaml],
  parent: inputEl,
});

renderButtonEl.addEventListener("click", () => {
  const input = inputView.state.sliceDoc();
  console.log("input", input);
  let output;
  try {
    output = render(input, sourceId);
  } catch (err) {
    const { reports } = err;
    if (reports == null) throw err;
    const message = reports[0].message;
    outputEl.innerText = JSON.stringify(message, null, 2);
    return;
  }
  console.log("value", output);
  outputEl.innerText = JSON.stringify(output, null, 2);
});
