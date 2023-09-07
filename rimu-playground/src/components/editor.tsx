'use client'

import './editor.css'

import React, { useRef, useEffect } from 'react'
import { clsx } from 'clsx'

import { EditorState } from '@codemirror/state'
import { EditorView, basicSetup } from 'codemirror'
import * as yamlMode from '@codemirror/legacy-modes/mode/yaml'
import { StreamLanguage, LanguageSupport } from '@codemirror/language'
import { oneDark } from '@codemirror/theme-one-dark'

const sourceId = 'playground'

export type EditorProps = {
  className: string
  rimu: typeof import('rimu-wasm')
  initialCode: string
  setCode: (code: string) => void
  setOutput: (output: any) => void
}

export function Editor(props: EditorProps) {
  const { className, rimu, initialCode, setCode, setOutput } = props

  const editorRef = useRef(null)

  useEffect(() => rimu.init(), [rimu])

  useEffect(() => {
    if (editorRef.current == null) return

    const yaml = new LanguageSupport(StreamLanguage.define(yamlMode.yaml))

    const onUpdate = EditorView.updateListener.of((v) => {
      const code = v.state.doc.toString()

      let output
      try {
        output = rimu.render(code, sourceId)
      } catch (err) {
        console.error(err)
        return
      }

      setOutput(output)
      setCode(code)
    })

    const startState = EditorState.create({
      doc: initialCode,
      extensions: [basicSetup, oneDark, yaml, onUpdate],
    })

    const view = new EditorView({ state: startState, parent: editorRef.current })

    return () => {
      view.destroy()
    }
  }, [editorRef, rimu, initialCode, setCode, setOutput])

  return <div className={className} ref={editorRef}></div>
}

/*
import { init, render } from "rimu-wasm";
import { basicSetup, EditorView } from "codemirror";
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
*/
