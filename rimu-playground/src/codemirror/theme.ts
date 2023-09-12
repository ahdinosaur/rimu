import { variants } from '@catppuccin/palette'
import { Variant } from 'codemirror-theme-catppuccin'
import { EditorView } from 'codemirror'

type Palette = (typeof variants)[Variant]

export const createDiagnosticTheme = (palette: Palette) => {
  const error = palette.red.hex
  const warning = palette.yellow.hex
  const hint = palette.peach.hex
  const info = palette.teal.hex

  const panelBg = palette.surface2.hex
  const panelFg = palette.text.hex
  const panelFocusedBg = palette.surface1.hex

  return EditorView.baseTheme({
    '.cm-report': {
      padding: '3px 6px 3px 8px',
      marginLeft: '-1px',
      display: 'block',
      whiteSpace: 'pre-wrap',
    },
    '.cm-report-error': { borderLeft: `5px solid ${error}` },
    '.cm-report-warning': { borderLeft: `5px solid ${warning}` },
    '.cm-report-info': { borderLeft: `5px solid ${info}` },
    '.cm-report-hint': { borderLeft: `5px solid ${hint}` },

    '.cm-reportAction': {
      font: 'inherit',
      border: 'none',
      padding: '2px 4px',
      backgroundColor: '#444',
      color: 'white',
      borderRadius: '3px',
      marginLeft: '8px',
      cursor: 'pointer',
    },

    '.cm-reportSource': {
      fontSize: '70%',
      opacity: 0.7,
    },

    '.cm-diagnosticRange': {
      backgroundPosition: 'left bottom',
      backgroundRepeat: 'repeat-x',
      paddingBottom: '0.7px',
    },

    '.cm-diagnosticRange-error': { backgroundImage: underline(error) },
    '.cm-diagnosticRange-warning': { backgroundImage: underline(warning) },
    '.cm-diagnosticRange-info': { backgroundImage: underline(info) },
    '.cm-diagnosticRange-hint': { backgroundImage: underline(hint) },
    '.cm-diagnosticRange-active': { backgroundColor: '#ffdd9980' },

    '.cm-tooltip-diagnostic': {
      padding: 0,
      margin: 0,
    },

    '.cm-diagnosticPoint': {
      position: 'relative',

      '&:after': {
        content: '""',
        position: 'absolute',
        bottom: 0,
        left: '-2px',
        borderLeft: '3px solid transparent',
        borderRight: '3px solid transparent',
        borderBottom: '4px solid #d11',
      },
    },

    '.cm-diagnosticPoint-warning': {
      '&:after': { borderBottomColor: warning },
    },
    '.cm-diagnosticPoint-info': {
      '&:after': { borderBottomColor: info },
    },
    '.cm-diagnosticPoint-hint': {
      '&:after': { borderBottomColor: hint },
    },

    '.cm-panel.cm-panel-diagnostic': {
      position: 'relative',
      '& ul': {
        maxHeight: '100px',
        overflowY: 'auto',
        '& [aria-selected]': {
          backgroundColor: panelBg,
          color: panelFg,
          '& u': { textDecoration: 'underline' },
        },
        '&:focus [aria-selected]': {
          backgroundColor: panelFocusedBg,
        },
        '& u': { textDecoration: 'none' },
        padding: 0,
        margin: 0,
      },
      '& [name=close]': {
        position: 'absolute',
        top: '0',
        right: '2px',
        background: 'inherit',
        border: 'none',
        font: 'inherit',
        padding: 0,
        margin: 0,
      },
    },
  })
}

export const createDiagnosticGutterTheme = (palette: Palette) =>
  EditorView.baseTheme({
    '.cm-gutter-diagnostic': {
      width: '1.4em',
      '& .cm-gutterElement': {
        padding: '.2em',
      },
    },
    '.cm-diagnostic-marker': {
      width: '1em',
      height: '1em',
    },
    '.cm-diagnostic-marker-info': {
      content: svg(
        `<path fill="#aaf" stroke="#77e" stroke-width="6" stroke-linejoin="round" d="M5 5L35 5L35 35L5 35Z"/>`,
      ),
    },
    '.cm-diagnostic-marker-warning': {
      content: svg(
        `<path fill="#fe8" stroke="#fd7" stroke-width="6" stroke-linejoin="round" d="M20 6L37 35L3 35Z"/>`,
      ),
    },
    '.cm-diagnostic-marker-error': {
      content: svg(`<circle cx="20" cy="20" r="15" fill="#f87" stroke="#f43" stroke-width="6"/>`),
    },
  })

function svg(content: string, attrs = `viewBox="0 0 40 40"`) {
  return `url('data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" ${attrs}>${encodeURIComponent(
    content,
  )}</svg>')`
}

function underline(color: string) {
  return svg(
    `<path d="m0 2.5 l2 -1.5 l1 0 l2 1.5 l1 0" stroke="${color}" fill="none" stroke-width=".7"/>`,
    `width="6" height="3"`,
  )
}
