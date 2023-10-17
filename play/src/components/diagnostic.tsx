import { Report } from '@/codemirror/diagnostics'
import { Box, List, ListItem, Text, useOutsideClick } from '@chakra-ui/react'
import { forwardRef, useCallback, useRef, useState } from 'react'

type DiagnosticPanelProps = {
  reports: Array<Report>
}

export function DiagnosticPanel(props: DiagnosticPanelProps) {
  const { reports } = props

  const [selectedIndex, setSelectedIndex] = useState<number | null>(null)

  const listRef = useRef(null)
  useOutsideClick({
    ref: listRef,
    handler: () => setSelectedIndex(null),
  })

  return (
    <Box>
      <DiagnosticList ref={listRef} selectedIndex={selectedIndex}>
        {reports.map((report, index) => (
          <DiagnosticReport
            key={index}
            report={report}
            index={index}
            isSelected={index === selectedIndex}
            handleSelect={() => setSelectedIndex(index)}
          />
        ))}
      </DiagnosticList>
    </Box>
  )
}

type DiagnosticListProps = {
  children: React.ReactNode
  selectedIndex: number | null
}

const DiagnosticList = forwardRef<HTMLUListElement, DiagnosticListProps>(
  function DiagnosticList(props, ref) {
    const { children, selectedIndex } = props
    const activeId = selectedIndex == null ? undefined : `diagnostic-list-item-${selectedIndex}`
    return (
      <List ref={ref} role="listbox" aria-activedescendant={activeId}>
        {children}
      </List>
    )
  },
)

type DiagnosticReportProps = {
  report: Report
  index: number
  isSelected: boolean
  handleSelect: () => void
}

function DiagnosticReport(props: DiagnosticReportProps) {
  const { report, index, isSelected, handleSelect } = props
  const { message } = report

  return (
    <ListItem
      id={`diagnostic-list-item-${index}`}
      role="option"
      aria-selected={isSelected}
      onClick={handleSelect}
    >
      <Text>{message}</Text>
    </ListItem>
  )
}

/*
function renderReport(view: EditorView, report: Report, inPanel: boolean) {
  let keys = inPanel ? assignKeys(report.actions) : []
  return elt(
    'li',
    { class: 'cm-report cm-report-' + report.severity },
    elt('span', { class: 'cm-reportText' }, report.message),
    report.actions?.map((action, i) => {
      let fired = false,
        click = (e: Event) => {
          e.preventDefault()
          if (fired) return
          fired = true
          let found = findReport(view.state.field(diagnosticState).reports, report)
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
    report.span.sourceId && elt('div', { class: 'cm-reportSource' }, report.span.sourceId),
  )
}
*/

/*

class PanelItem {
  id = 'item_' + Math.floor(Math.random() * 0xffffffff).toString(16)
  dom: HTMLElement

  constructor(
    view: EditorView,
    readonly report: Report,
  ) {
    this.dom = renderReport(view, report, true)
    this.dom.id = this.id
    this.dom.setAttribute('role', 'option')
  }
}
class DiagnosticPanel implements Panel {
  items: PanelItem[] = []
  dom: HTMLElement
  list: HTMLElement

  constructor(readonly view: EditorView) {
    let onkeydown = (event: KeyboardEvent) => {
      if (event.keyCode == 27) {
        // Escape
        closeDiagnosticPanel(this.view)
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
    this.dom = elt('div', { class: 'cm-panel-diagnostic' }, this.list)
    this.update()
  }

  get selectedIndex() {
    let selected = this.view.state.field(diagnosticState).selected
    if (!selected) return -1
    for (let i = 0; i < this.items.length; i++)
      if (this.items[i].report == selected.report) return i
    return -1
  }

  update() {
    let { reports, selected } = this.view.state.field(diagnosticState)
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
    while (
      i < this.items.length &&
      !(this.items.length == 1 && this.items[0].report.span.from < 0)
    ) {
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
    let field = this.view.state.field(diagnosticState)
    let selection = findReport(field.reports, this.items[selectedIndex].report)
    if (!selection) return
    this.view.dispatch({
      selection: { anchor: selection.from, head: selection.to },
      scrollIntoView: true,
      effects: movePanelSelection.of(selection),
    })
  }

  static open(view: EditorView) {
    return new DiagnosticPanel(view)
  }
}
*/
