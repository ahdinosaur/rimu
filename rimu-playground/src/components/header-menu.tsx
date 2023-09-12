import { FaChevronDown, FaMoon, FaSun } from 'react-icons/fa'
import {
  Box,
  Button,
  Flex,
  Heading,
  IconButton,
  Menu,
  MenuButton,
  MenuItem,
  MenuList,
  useColorMode,
  useColorModeValue,
} from '@chakra-ui/react'

export type HeaderMenuProps = {
  height: string
  setCodeToLoad: (code: string) => void
}

export function HeaderMenu(props: HeaderMenuProps) {
  const { height, setCodeToLoad } = props

  return (
    <Flex
      sx={{
        height,
        flexDirection: 'row',
        alignItems: 'baseline',
        width: '100%',
        backgroundColor: 'rimu.header.background',
      }}
    >
      <Flex sx={{ flexDirection: 'row', alignItems: 'baseline' }}>
        <Heading
          as="h1"
          size="lg"
          sx={{ alignSelf: 'start', paddingLeft: 4, lineHeight: 'normal' }}
        >
          Rimu
        </Heading>
        <ExamplesMenu setCodeToLoad={setCodeToLoad} />
      </Flex>

      <Box sx={{ flexGrow: 1 }} />

      <Flex sx={{ flexDirection: 'row', alignItems: 'baseline' }}>
        <ColorModeSwitch />
      </Flex>
    </Flex>
  )
}

type Example = {
  name: string
  code: string
}
const examples: Array<Example> = [
  {
    name: 'Hello world',
    code: 'hello: "world"',
  },
]

type ExamplesMenuProps = {
  setCodeToLoad: (code: string) => void
}

function ExamplesMenu(props: ExamplesMenuProps) {
  const { setCodeToLoad } = props

  return (
    <Menu>
      <MenuButton as={Button} rightIcon={<FaChevronDown />}>
        Examples
      </MenuButton>
      <MenuList>
        {examples.map((example, index) => {
          const { name, code } = example
          return (
            <MenuItem key={index} onClick={() => setCodeToLoad(code)}>
              {name}
            </MenuItem>
          )
        })}
      </MenuList>
    </Menu>
  )
}

function ColorModeSwitch() {
  const { toggleColorMode } = useColorMode()

  const text = useColorModeValue('dark', 'light')
  const SwitchIcon = useColorModeValue(FaMoon, FaSun)

  return (
    <IconButton
      size="sm"
      fontSize="lg"
      aria-label={`Switch to ${text} mode`}
      variant="ghost"
      color="current"
      onClick={toggleColorMode}
      icon={<SwitchIcon />}
    />
  )
}
