import {
  EditorView,
  ViewPlugin,
  Decoration,
  DecorationSet,
  WidgetType,
  ViewUpdate,
  Command,
  logException,
  KeyBinding,
  hoverTooltip,
  Tooltip,
  showTooltip,
  gutter,
  GutterMarker,
  PanelConstructor,
  Panel,
  showPanel,
  getPanel,
} from '@codemirror/view'
import {
  Text,
  StateEffect,
  StateField,
  Extension,
  TransactionSpec,
  Transaction,
  EditorState,
  Facet,
  combineConfig,
  RangeSet,
  Range,
} from '@codemirror/state'
import elt from 'crelt'
import { color } from '@codemirror/theme-one-dark'

type Severity = 'hint' | 'info' | 'warning' | 'error'

export interface Label {
  message: string
  from: number
  to: number
}

export interface Note {
  message: string
}

export interface Report {
  message: string
  from: number
  to: number
  sourceId: string
  severity: Severity
  markClass?: string
  // labels: Array<Label>
  // notes: Array<Note>
  renderMessage?: () => Node
  actions?: readonly Action[]
}

/// An action associated with a diagnostic.
export interface Action {
  /// The label to show to the user. Should be relatively short.
  name: string
  /// The function to call when the user activates this action. Is
  /// given the diagnostic's _current_ position, which may have
  /// changed since the creation of the diagnostic, due to editing.
  apply: (view: EditorView, from: number, to: number) => void
}

type ReportFilter = (reports: readonly Report[]) => Report[]

interface EvalConfig {
  delay?: number
  needsRefresh?: null | ((update: ViewUpdate) => boolean)
  markerFilter?: null | ReportFilter
  tooltipFilter?: null | ReportFilter
}

interface EvalGutterConfig {
  hoverTime?: number
  markerFilter?: null | ReportFilter
  tooltipFilter?: null | ReportFilter
}

class SelectedReport {
  constructor(readonly from: number, readonly to: number, readonly report: Report) {}
}

class EvalState {
  constructor(
    readonly reports: DecorationSet,
    readonly panel: PanelConstructor | null,
    readonly selected: SelectedReport | null,
  ) {}

  static init(reports: readonly Report[], panel: PanelConstructor | null, state: EditorState) {
    // Filter the list of reports for which to create markers
    let markedReports = reports
    let reportFilter = state.facet(evalConfig).markerFilter
    if (reportFilter) markedReports = reportFilter(markedReports)

    let ranges = Decoration.set(
      markedReports.map((d: Report) => {
        // For zero-length ranges or ranges covering only a line break, create a widget
        return d.from == d.to || (d.from == d.to - 1 && state.doc.lineAt(d.from).to == d.from)
          ? Decoration.widget({
              widget: new ReportWidget(d),
              report: d,
            }).range(d.from)
          : Decoration.mark({
              attributes: {
                class:
                  'cm-evalRange cm-evalRange-' +
                  d.severity +
                  (d.markClass ? ' ' + d.markClass : ''),
              },
              report: d,
            }).range(d.from, d.to)
      }),
      true,
    )
    return new EvalState(ranges, panel, findReport(ranges))
  }
}

function findReport(
  reports: DecorationSet,
  report: Report | null = null,
  after = 0,
): SelectedReport | null {
  let found: SelectedReport | null = null
  reports.between(after, 1e9, (from, to, { spec }) => {
    if (report && spec.report != report) return
    found = new SelectedReport(from, to, spec.report)
    return false
  })
  return found
}

function hideTooltip(tr: Transaction, tooltip: Tooltip) {
  let line = tr.startState.doc.lineAt(tooltip.pos)
  return !!(
    tr.effects.some((e) => e.is(setReportsEffect)) || tr.changes.touchesRange(line.from, line.to)
  )
}

function maybeEnableEval(state: EditorState, effects: readonly StateEffect<unknown>[]) {
  return state.field(evalState, false)
    ? effects
    : effects.concat(StateEffect.appendConfig.of(evalExtensions))
}

/// Returns a transaction spec which updates the current set of
/// reports, and enables the eval extension if if wasn't already
/// active.
export function setReports(state: EditorState, reports: readonly Report[]): TransactionSpec {
  return {
    effects: maybeEnableEval(state, [setReportsEffect.of(reports)]),
  }
}

/// The state effect that updates the set of active reports. Can
/// be useful when writing an extension that needs to track these.
export const setReportsEffect = StateEffect.define<readonly Report[]>()

const togglePanel = StateEffect.define<boolean>()

const movePanelSelection = StateEffect.define<SelectedReport>()

const evalState = StateField.define<EvalState>({
  create() {
    return new EvalState(Decoration.none, EvalPanel.open, null)
  },
  update(value, tr) {
    if (tr.docChanged) {
      let mapped = value.reports.map(tr.changes),
        selected = null
      if (value.selected) {
        let selPos = tr.changes.mapPos(value.selected.from, 1)
        selected =
          findReport(mapped, value.selected.report, selPos) || findReport(mapped, null, selPos)
      }
      value = new EvalState(mapped, value.panel, selected)
    }

    for (let effect of tr.effects) {
      if (effect.is(setReportsEffect)) {
        value = EvalState.init(effect.value, value.panel, tr.state)
      } else if (effect.is(togglePanel)) {
        value = new EvalState(value.reports, effect.value ? EvalPanel.open : null, value.selected)
      } else if (effect.is(movePanelSelection)) {
        value = new EvalState(value.reports, value.panel, effect.value)
      }
    }

    return value
  },
  provide: (f) => [
    showPanel.from(f, (val) => val.panel),
    EditorView.decorations.from(f, (s) => s.reports),
  ],
})

/// Returns the number of active eval reports in the given state.
export function reportCount(state: EditorState) {
  const eval_ = state.field(evalState, false)
  return eval_ ? eval_.reports.size : 0
}

const activeMark = Decoration.mark({ class: 'cm-evalRange cm-evalRange-active' })

function evalTooltip(view: EditorView, pos: number, side: -1 | 1) {
  let { reports } = view.state.field(evalState)
  let found: Report[] = [],
    stackStart = 2e8,
    stackEnd = 0
  reports.between(pos - (side < 0 ? 1 : 0), pos + (side > 0 ? 1 : 0), (from, to, { spec }) => {
    if (
      pos >= from &&
      pos <= to &&
      (from == to || ((pos > from || side > 0) && (pos < to || side < 0)))
    ) {
      found.push(spec.report)
      stackStart = Math.min(from, stackStart)
      stackEnd = Math.max(to, stackEnd)
    }
  })

  let reportFilter = view.state.facet(evalConfig).tooltipFilter
  if (reportFilter) found = reportFilter(found)

  if (!found.length) return null

  return {
    pos: stackStart,
    end: stackEnd,
    above: view.state.doc.lineAt(stackStart).to < stackEnd,
    create() {
      return { dom: reportsTooltip(view, found) }
    },
  }
}

function reportsTooltip(view: EditorView, reports: readonly Report[]) {
  return elt(
    'ul',
    { class: 'cm-tooltip-eval' },
    reports.map((d) => renderReport(view, d, false)),
  )
}

/// Command to open and focus the eval panel.
export const openEvalPanel: Command = (view: EditorView) => {
  let field = view.state.field(evalState, false)
  if (!field || !field.panel)
    view.dispatch({ effects: maybeEnableEval(view.state, [togglePanel.of(true)]) })
  let panel = getPanel(view, EvalPanel.open)
  if (panel) (panel.dom.querySelector('.cm-panel-eval ul') as HTMLElement).focus()
  return true
}

/// Command to close the eval panel, when open.
export const closeEvalPanel: Command = (view: EditorView) => {
  let field = view.state.field(evalState, false)
  if (!field || !field.panel) return false
  view.dispatch({ effects: togglePanel.of(false) })
  return true
}

/// Move the selection to the next report.
export const nextReport: Command = (view: EditorView) => {
  let field = view.state.field(evalState, false)
  if (!field) return false
  let sel = view.state.selection.main,
    next = field.reports.iter(sel.to + 1)
  if (!next.value) {
    next = field.reports.iter(0)
    if (!next.value || (next.from == sel.from && next.to == sel.to)) return false
  }
  view.dispatch({ selection: { anchor: next.from, head: next.to }, scrollIntoView: true })
  return true
}

/// Move the selection to the previous report.
export const previousReport: Command = (view: EditorView) => {
  let { state } = view,
    field = state.field(evalState, false)
  if (!field) return false
  let sel = state.selection.main
  let prevFrom: number | undefined,
    prevTo: number | undefined,
    lastFrom: number | undefined,
    lastTo: number | undefined
  field.reports.between(0, state.doc.length, (from, to) => {
    if (to < sel.to && (prevFrom == null || prevFrom < from)) {
      prevFrom = from
      prevTo = to
    }
    if (lastFrom == null || from > lastFrom) {
      lastFrom = from
      lastTo = to
    }
  })
  if (lastFrom == null || (prevFrom == null && lastFrom == sel.from)) return false
  view.dispatch({
    selection: { anchor: prevFrom ?? lastFrom, head: prevTo ?? lastTo },
    scrollIntoView: true,
  })
  return true
}

/// A set of default key bindings for the eval functionality.
///
/// - Ctrl-Shift-m (Cmd-Shift-m on macOS): [`openEvalPanel`](#eval.openEvalPanel)
/// - F8: [`nextReport`](#eval.nextReport)
export const evalKeymap: readonly KeyBinding[] = [
  { key: 'Mod-Shift-m', run: openEvalPanel, preventDefault: true },
  { key: 'F8', run: nextReport },
]

/// The type of a function that produces reports.
export type EvalSource = (view: EditorView) => readonly Report[] | Promise<readonly Report[]>

const evalPlugin = ViewPlugin.fromClass(
  class {
    evalTime: number
    timeout: ReturnType<typeof setTimeout> | undefined = undefined
    set = true

    constructor(readonly view: EditorView) {
      let { delay } = view.state.facet(evalConfig)
      this.evalTime = Date.now() + delay
      this.run = this.run.bind(this)
      this.timeout = setTimeout(this.run, delay)
    }

    run() {
      let now = Date.now()
      if (now < this.evalTime - 10) {
        this.timeout = setTimeout(this.run, this.evalTime - now)
      } else {
        this.set = false
        let { state } = this.view,
          { sources } = state.facet(evalConfig)
        Promise.all(sources.map((source) => Promise.resolve(source(this.view)))).then(
          (annotations) => {
            let all = annotations.reduce((a, b) => a.concat(b))
            if (this.view.state.doc == state.doc)
              this.view.dispatch(setReports(this.view.state, all))
          },
          (error) => {
            logException(this.view.state, error)
          },
        )
      }
    }

    update(update: ViewUpdate) {
      let config = update.state.facet(evalConfig)
      if (
        update.docChanged ||
        config != update.startState.facet(evalConfig) ||
        (config.needsRefresh && config.needsRefresh(update))
      ) {
        this.evalTime = Date.now() + config.delay
        if (!this.set) {
          this.set = true
          this.timeout = setTimeout(this.run, config.delay)
        }
      }
    }

    force() {
      if (this.set) {
        this.evalTime = Date.now()
        this.run()
      }
    }

    destroy() {
      clearTimeout(this.timeout)
    }
  },
)

const evalConfig = Facet.define<
  { source: EvalSource; config: EvalConfig },
  Required<EvalConfig> & { sources: readonly EvalSource[] }
>({
  combine(input) {
    return {
      sources: input.map((i) => i.source),
      ...combineConfig(
        input.map((i) => i.config),
        {
          delay: 750,
          markerFilter: null,
          tooltipFilter: null,
          needsRefresh: null,
        },
        {
          needsRefresh: (a, b) => (!a ? b : !b ? a : (u) => a(u) || b(u)),
        },
      ),
    }
  },
})

/// Given a report source, this function returns an extension that
/// enables evaling with that source. It will be called whenever the
/// editor is idle (after its content changed).
export function evaler(source: EvalSource, config: EvalConfig = {}): Extension {
  return [evalConfig.of({ source, config }), evalPlugin, evalExtensions]
}

/// Forces any evalers [configured](#eval.evaler) to run when the
/// editor is idle to run right away.
export function forceEvaling(view: EditorView) {
  let plugin = view.plugin(evalPlugin)
  if (plugin) plugin.force()
}

function assignKeys(actions: readonly Action[] | undefined) {
  let assigned: string[] = []
  if (actions)
    actions: for (let { name } of actions) {
      for (let i = 0; i < name.length; i++) {
        let ch = name[i]
        if (/[a-zA-Z]/.test(ch) && !assigned.some((c) => c.toLowerCase() == ch.toLowerCase())) {
          assigned.push(ch)
          continue actions
        }
      }
      assigned.push('')
    }
  return assigned
}

function renderReport(view: EditorView, report: Report, inPanel: boolean) {
  let keys = inPanel ? assignKeys(report.actions) : []
  return elt(
    'li',
    { class: 'cm-report cm-report-' + report.severity },
    elt(
      'span',
      { class: 'cm-reportText' },
      report.renderMessage ? report.renderMessage() : report.message,
    ),
    report.actions?.map((action, i) => {
      let fired = false,
        click = (e: Event) => {
          e.preventDefault()
          if (fired) return
          fired = true
          let found = findReport(view.state.field(evalState).reports, report)
          if (found) action.apply(view, found.from, found.to)
        }
      let { name } = action,
        keyIndex = keys[i] ? name.indexOf(keys[i]) : -1
      let nameElt =
        keyIndex < 0
          ? name
          : [
              name.slice(0, keyIndex),
              elt('u', name.slice(keyIndex, keyIndex + 1)),
              name.slice(keyIndex + 1),
            ]
      return elt(
        'button',
        {
          type: 'button',
          class: 'cm-reportAction',
          onclick: click,
          onmousedown: click,
          'aria-label': ` Action: ${name}${keyIndex < 0 ? '' : ` (access key "${keys[i]})"`}.`,
        },
        nameElt,
      )
    }),
    report.sourceId && elt('div', { class: 'cm-reportSource' }, report.sourceId),
  )
}

class ReportWidget extends WidgetType {
  constructor(readonly report: Report) {
    super()
  }

  eq(other: ReportWidget) {
    return other.report == this.report
  }

  toDOM() {
    return elt('span', { class: 'cm-evalPoint cm-evalPoint-' + this.report.severity })
  }
}

class PanelItem {
  id = 'item_' + Math.floor(Math.random() * 0xffffffff).toString(16)
  dom: HTMLElement

  constructor(view: EditorView, readonly report: Report) {
    this.dom = renderReport(view, report, true)
    this.dom.id = this.id
    this.dom.setAttribute('role', 'option')
  }
}

class EvalPanel implements Panel {
  items: PanelItem[] = []
  dom: HTMLElement
  list: HTMLElement

  constructor(readonly view: EditorView) {
    let onkeydown = (event: KeyboardEvent) => {
      if (event.keyCode == 27) {
        // Escape
        closeEvalPanel(this.view)
        this.view.focus()
      } else if (event.keyCode == 38 || event.keyCode == 33) {
        // ArrowUp, PageUp
        this.moveSelection((this.selectedIndex - 1 + this.items.length) % this.items.length)
      } else if (event.keyCode == 40 || event.keyCode == 34) {
        // ArrowDown, PageDown
        this.moveSelection((this.selectedIndex + 1) % this.items.length)
      } else if (event.keyCode == 36) {
        // Home
        this.moveSelection(0)
      } else if (event.keyCode == 35) {
        // End
        this.moveSelection(this.items.length - 1)
      } else if (event.keyCode == 13) {
        // Enter
        this.view.focus()
      } else if (event.keyCode >= 65 && event.keyCode <= 90 && this.selectedIndex >= 0) {
        // A-Z
        let { report } = this.items[this.selectedIndex],
          keys = assignKeys(report.actions)
        for (let i = 0; i < keys.length; i++)
          if (keys[i].toUpperCase().charCodeAt(0) == event.keyCode) {
            let found = findReport(this.view.state.field(evalState).reports, report)
            if (found) report.actions![i].apply(view, found.from, found.to)
          }
      } else {
        return
      }
      event.preventDefault()
    }
    let onclick = (event: MouseEvent) => {
      for (let i = 0; i < this.items.length; i++) {
        if (this.items[i].dom.contains(event.target as HTMLElement)) this.moveSelection(i)
      }
    }

    this.list = elt('ul', {
      tabIndex: 0,
      role: 'listbox',
      'aria-label': this.view.state.phrase('Reports'),
      onkeydown,
      onclick,
    })
    this.dom = elt('div', { class: 'cm-panel-eval' }, this.list)
    this.update()
  }

  get selectedIndex() {
    let selected = this.view.state.field(evalState).selected
    if (!selected) return -1
    for (let i = 0; i < this.items.length; i++)
      if (this.items[i].report == selected.report) return i
    return -1
  }

  update() {
    let { reports, selected } = this.view.state.field(evalState)
    let i = 0,
      needsSync = false,
      newSelectedItem: PanelItem | null = null
    reports.between(0, this.view.state.doc.length, (_start, _end, { spec }) => {
      let found = -1,
        item
      for (let j = i; j < this.items.length; j++)
        if (this.items[j].report == spec.report) {
          found = j
          break
        }
      if (found < 0) {
        item = new PanelItem(this.view, spec.report)
        this.items.splice(i, 0, item)
        needsSync = true
      } else {
        item = this.items[found]
        if (found > i) {
          this.items.splice(i, found - i)
          needsSync = true
        }
      }
      if (selected && item.report == selected.report) {
        if (!item.dom.hasAttribute('aria-selected')) {
          item.dom.setAttribute('aria-selected', 'true')
          newSelectedItem = item
        }
      } else if (item.dom.hasAttribute('aria-selected')) {
        item.dom.removeAttribute('aria-selected')
      }
      i++
    })
    while (i < this.items.length && !(this.items.length == 1 && this.items[0].report.from < 0)) {
      needsSync = true
      this.items.pop()
    }
    if (newSelectedItem) {
      this.list.setAttribute('aria-activedescendant', newSelectedItem!.id)
      this.view.requestMeasure({
        key: this,
        read: () => ({
          sel: newSelectedItem!.dom.getBoundingClientRect(),
          panel: this.list.getBoundingClientRect(),
        }),
        write: ({ sel, panel }) => {
          let scaleY = panel.height / this.list.offsetHeight
          if (sel.top < panel.top) this.list.scrollTop -= (panel.top - sel.top) / scaleY
          else if (sel.bottom > panel.bottom)
            this.list.scrollTop += (sel.bottom - panel.bottom) / scaleY
        },
      })
    } else if (this.selectedIndex < 0) {
      this.list.removeAttribute('aria-activedescendant')
    }
    if (needsSync) this.sync()
  }

  sync() {
    let domPos: ChildNode | null = this.list.firstChild
    function rm() {
      let prev = domPos!
      domPos = prev.nextSibling
      prev.remove()
    }

    for (let item of this.items) {
      if (item.dom.parentNode == this.list) {
        while (domPos != item.dom) rm()
        domPos = item.dom.nextSibling
      } else {
        this.list.insertBefore(item.dom, domPos)
      }
    }
    while (domPos) rm()
  }

  moveSelection(selectedIndex: number) {
    if (this.selectedIndex < 0) return
    let field = this.view.state.field(evalState)
    let selection = findReport(field.reports, this.items[selectedIndex].report)
    if (!selection) return
    this.view.dispatch({
      selection: { anchor: selection.from, head: selection.to },
      scrollIntoView: true,
      effects: movePanelSelection.of(selection),
    })
  }

  static open(view: EditorView) {
    return new EvalPanel(view)
  }
}

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

const baseTheme = EditorView.baseTheme({
  '.cm-report': {
    padding: '3px 6px 3px 8px',
    marginLeft: '-1px',
    display: 'block',
    whiteSpace: 'pre-wrap',
  },
  '.cm-report-error': { borderLeft: '5px solid #d11' },
  '.cm-report-warning': { borderLeft: '5px solid orange' },
  '.cm-report-info': { borderLeft: '5px solid #999' },
  '.cm-report-hint': { borderLeft: '5px solid #66d' },

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

  '.cm-evalRange': {
    backgroundPosition: 'left bottom',
    backgroundRepeat: 'repeat-x',
    paddingBottom: '0.7px',
  },

  '.cm-evalRange-error': { backgroundImage: underline('#d11') },
  '.cm-evalRange-warning': { backgroundImage: underline('orange') },
  '.cm-evalRange-info': { backgroundImage: underline('#999') },
  '.cm-evalRange-hint': { backgroundImage: underline('#66d') },
  '.cm-evalRange-active': { backgroundColor: '#ffdd9980' },

  '.cm-tooltip-eval': {
    padding: 0,
    margin: 0,
  },

  '.cm-evalPoint': {
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

  '.cm-evalPoint-warning': {
    '&:after': { borderBottomColor: 'orange' },
  },
  '.cm-evalPoint-info': {
    '&:after': { borderBottomColor: '#999' },
  },
  '.cm-evalPoint-hint': {
    '&:after': { borderBottomColor: '#66d' },
  },

  '.cm-panel.cm-panel-eval': {
    position: 'relative',
    '& ul': {
      maxHeight: '100px',
      overflowY: 'auto',
      '& [aria-selected]': {
        backgroundColor: color.darkBackground,
        '& u': { textDecoration: 'underline' },
      },
      '&:focus [aria-selected]': {
        backgroundColor: color.highlightBackground,
        color: color.ivory,
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

function severityWeight(sev: Severity) {
  return sev == 'error' ? 4 : sev == 'warning' ? 3 : sev == 'info' ? 2 : 1
}

class EvalGutterMarker extends GutterMarker {
  severity: Severity
  constructor(readonly reports: readonly Report[]) {
    super()
    this.severity = reports.reduce(
      (max, d) => (severityWeight(max) < severityWeight(d.severity) ? d.severity : max),
      'hint' as Severity,
    )
  }

  toDOM(view: EditorView) {
    let elt = document.createElement('div')
    elt.className = 'cm-eval-marker cm-eval-marker-' + this.severity

    let reports = this.reports
    let reportsFilter = view.state.facet(evalGutterConfig).tooltipFilter
    if (reportsFilter) reports = reportsFilter(reports)

    if (reports.length) elt.onmouseover = () => gutterMarkerMouseOver(view, elt, reports)

    return elt
  }
}

const enum Hover {
  Time = 300,
  Margin = 10,
}

function trackHoverOn(view: EditorView, marker: HTMLElement) {
  let mousemove = (event: MouseEvent) => {
    let rect = marker.getBoundingClientRect()
    if (
      event.clientX > rect.left - Hover.Margin &&
      event.clientX < rect.right + Hover.Margin &&
      event.clientY > rect.top - Hover.Margin &&
      event.clientY < rect.bottom + Hover.Margin
    )
      return
    for (let target = event.target as Node | null; target; target = target.parentNode) {
      if (target.nodeType == 1 && (target as HTMLElement).classList.contains('cm-tooltip-eval'))
        return
    }
    window.removeEventListener('mousemove', mousemove)
    if (view.state.field(evalGutterTooltip))
      view.dispatch({ effects: setEvalGutterTooltip.of(null) })
  }
  window.addEventListener('mousemove', mousemove)
}

function gutterMarkerMouseOver(view: EditorView, marker: HTMLElement, reports: readonly Report[]) {
  function hovered() {
    let line = view.elementAtHeight(marker.getBoundingClientRect().top + 5 - view.documentTop)
    const linePos = view.coordsAtPos(line.from)
    if (linePos) {
      view.dispatch({
        effects: setEvalGutterTooltip.of({
          pos: line.from,
          above: false,
          create() {
            return {
              dom: reportsTooltip(view, reports),
              getCoords: () => marker.getBoundingClientRect(),
            }
          },
        }),
      })
    }
    marker.onmouseout = marker.onmousemove = null
    trackHoverOn(view, marker)
  }

  let { hoverTime } = view.state.facet(evalGutterConfig)

  let hoverTimeout = setTimeout(hovered, hoverTime)
  marker.onmouseout = () => {
    clearTimeout(hoverTimeout)
    marker.onmouseout = marker.onmousemove = null
  }
  marker.onmousemove = () => {
    clearTimeout(hoverTimeout)
    hoverTimeout = setTimeout(hovered, hoverTime)
  }
}

function markersForReports(doc: Text, reports: readonly Report[]) {
  let byLine: { [line: number]: Report[] } = Object.create(null)
  for (let report of reports) {
    let line = doc.lineAt(report.from)
    ;(byLine[line.from] || (byLine[line.from] = [])).push(report)
  }
  let markers: Range<GutterMarker>[] = []
  for (let line in byLine) {
    markers.push(new EvalGutterMarker(byLine[line]).range(+line))
  }
  return RangeSet.of(markers, true)
}

const evalGutterExtension = gutter({
  class: 'cm-gutter-eval',
  markers: (view) => view.state.field(evalGutterMarkers),
})

const evalGutterMarkers = StateField.define<RangeSet<GutterMarker>>({
  create() {
    return RangeSet.empty
  },
  update(markers, tr) {
    markers = markers.map(tr.changes)
    let reportFilter = tr.state.facet(evalGutterConfig).markerFilter
    for (let effect of tr.effects) {
      if (effect.is(setReportsEffect)) {
        let reports = effect.value
        if (reportFilter) reports = reportFilter(reports || [])
        markers = markersForReports(tr.state.doc, reports.slice(0))
      }
    }
    return markers
  },
})

const setEvalGutterTooltip = StateEffect.define<Tooltip | null>()

const evalGutterTooltip = StateField.define<Tooltip | null>({
  create() {
    return null
  },
  update(tooltip, tr) {
    if (tooltip && tr.docChanged)
      tooltip = hideTooltip(tr, tooltip)
        ? null
        : { ...tooltip, pos: tr.changes.mapPos(tooltip.pos) }
    return tr.effects.reduce((t, e) => (e.is(setEvalGutterTooltip) ? e.value : t), tooltip)
  },
  provide: (field) => showTooltip.from(field),
})

const evalGutterTheme = EditorView.baseTheme({
  '.cm-gutter-eval': {
    width: '1.4em',
    '& .cm-gutterElement': {
      padding: '.2em',
    },
  },
  '.cm-eval-marker': {
    width: '1em',
    height: '1em',
  },
  '.cm-eval-marker-info': {
    content: svg(
      `<path fill="#aaf" stroke="#77e" stroke-width="6" stroke-linejoin="round" d="M5 5L35 5L35 35L5 35Z"/>`,
    ),
  },
  '.cm-eval-marker-warning': {
    content: svg(
      `<path fill="#fe8" stroke="#fd7" stroke-width="6" stroke-linejoin="round" d="M20 6L37 35L3 35Z"/>`,
    ),
  },
  '.cm-eval-marker-error': {
    content: svg(`<circle cx="20" cy="20" r="15" fill="#f87" stroke="#f43" stroke-width="6"/>`),
  },
})

const evalExtensions = [
  evalState,
  EditorView.decorations.compute([evalState], (state) => {
    let { selected, panel } = state.field(evalState)
    return !selected || !panel || selected.from == selected.to
      ? Decoration.none
      : Decoration.set([activeMark.range(selected.from, selected.to)])
  }),
  hoverTooltip(evalTooltip, { hideOn: hideTooltip }),
  baseTheme,
]

const evalGutterConfig = Facet.define<EvalGutterConfig, Required<EvalGutterConfig>>({
  combine(configs) {
    return combineConfig(configs, {
      hoverTime: Hover.Time,
      markerFilter: null,
      tooltipFilter: null,
    })
  },
})

/// Returns an extension that installs a gutter showing markers for
/// each line that has reports, which can be hovered over to see
/// the reports.
export function evalGutter(config: EvalGutterConfig = {}): Extension {
  return [
    evalGutterConfig.of(config),
    evalGutterMarkers,
    evalGutterExtension,
    evalGutterTheme,
    evalGutterTooltip,
  ]
}

/// Iterate over the marked reports for the given editor state,
/// calling `f` for each of them. Note that, if the document changed
/// since the reports were created, the `Report` object will
/// hold the original outdated position, whereas the `to` and `from`
/// arguments hold the report's current position.
export function forEachReport(
  state: EditorState,
  f: (d: Report, from: number, to: number) => void,
) {
  let lState = state.field(evalState, false)
  if (lState && lState.reports.size)
    for (let iter = RangeSet.iter([lState.reports]); iter.value; iter.next())
      f(iter.value.spec.report, iter.from, iter.to)
}
