import { FaChevronDown, FaMoon, FaSun } from 'react-icons/fa'
import {
  Box,
  Button,
  Flex,
  HStack,
  Heading,
  IconButton,
  Menu,
  MenuButton,
  MenuItem,
  MenuList,
  Modal,
  ModalOverlay,
  ModalContent,
  ModalHeader,
  ModalFooter,
  ModalBody,
  ModalCloseButton,
  useColorMode,
  useColorModeValue,
  useDisclosure,
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
        alignItems: 'center',
        width: '100%',
        backgroundColor: 'rimu.header.background',
        paddingX: 1,
      }}
    >
      <HStack spacing={4} sx={{ alignItems: 'center' }}>
        <Heading as="h1" size="lg" sx={{ lineHeight: 'normal' }}>
          Rimu
        </Heading>
        <ExamplesMenu setCodeToLoad={setCodeToLoad} />
        <HelpButton />
      </HStack>

      <Box sx={{ flexGrow: 1 }} />

      <Flex sx={{ flexDirection: 'row', alignItems: 'center' }}>
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
      <MenuButton as={Button} size="sm" rightIcon={<FaChevronDown />}>
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

function HelpButton() {
  const { isOpen, onOpen, onClose } = useDisclosure()

  return (
    <>
      <Button size="sm" onClick={onOpen}>
        Help
      </Button>

      <Modal isOpen={isOpen} onClose={onClose}>
        <ModalOverlay />
        <ModalContent>
          <ModalHeader>How to use the Rimu Playground</ModalHeader>
          <ModalCloseButton />
          <ModalBody>TODO</ModalBody>

          <ModalFooter>
            <Button colorScheme="teal" mr={3} onClick={onClose}>
              Close
            </Button>
          </ModalFooter>
        </ModalContent>
      </Modal>
    </>
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
