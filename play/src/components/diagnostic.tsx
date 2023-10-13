import { Report } from '@/codemirror/diagnostics'
import { Box, List, ListItem, Text } from '@chakra-ui/react'

type DiagnosticPanelProps = {
  reports: Array<Report>
}

export function DiagnosticPanel(props: DiagnosticPanelProps) {
  const { reports } = props
  return (
    <Box>
      <DiagnosticList>
        {reports.map((report, index) => (
          <DiagnosticReport key={index} report={report} />
        ))}
      </DiagnosticList>
    </Box>
  )
}

type DiagnosticListProps = {
  children: React.ReactNode
}

function DiagnosticList(props: DiagnosticListProps) {
  const { children } = props
  return <List>{children}</List>
}

type DiagnosticReportProps = {
  report: Report
}

function DiagnosticReport(props: DiagnosticReportProps) {
  const { report } = props
  const { message } = report
  return (
    <ListItem>
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
